use std::sync::{Arc, Mutex, MutexGuard};

use axum::{
    Json, Router,
    extract::{Path, Query, State, rejection::QueryRejection},
    http::{Method, StatusCode, header},
    routing::{get, post},
};
use deckofcards::cards::{card::Card, deck_store::DeckStore};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

type SharedStore = Arc<Mutex<DeckStore>>;
type ApiError = (StatusCode, Json<ErrorResponse>);
type ApiResult<T> = Result<Json<T>, ApiError>;

#[derive(Deserialize)]
struct DrawQuery {
    count: Option<usize>,
}

#[derive(Serialize)]
struct DeckResponse {
    deck_id: String,
    remaining: usize,
}

#[derive(Serialize)]
struct DrawResponse {
    deck_id: String,
    cards: Vec<Card>,
    remaining: usize,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
}

fn json_error(status: StatusCode, message: &'static str) -> ApiError {
    (status, Json(ErrorResponse { error: message }))
}

fn parse_deck_id(id: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(id).map_err(|_| json_error(StatusCode::BAD_REQUEST, "Invalid deck ID"))
}

fn lock_store(store: &SharedStore) -> Result<MutexGuard<'_, DeckStore>, ApiError> {
    store
        .lock()
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"))
}

async fn create_deck(State(store): State<SharedStore>) -> ApiResult<DeckResponse> {
    let mut store = lock_store(&store)?;

    let id = store
        .create(true)
        .ok_or_else(|| json_error(StatusCode::SERVICE_UNAVAILABLE, "Deck capacity reached"))?;

    let remaining = store
        .get(&id)
        .ok_or_else(|| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"))?
        .remaining();

    Ok(Json(DeckResponse {
        deck_id: id.to_string(),
        remaining,
    }))
}

async fn get_deck(
    State(store): State<SharedStore>,
    Path(id): Path<String>,
) -> ApiResult<DeckResponse> {
    let id = parse_deck_id(&id)?;
    let mut store = lock_store(&store)?;

    let deck = store
        .get(&id)
        .ok_or_else(|| json_error(StatusCode::NOT_FOUND, "Deck not found"))?;

    Ok(Json(DeckResponse {
        deck_id: id.to_string(),
        remaining: deck.remaining(),
    }))
}

async fn draw_cards(
    State(store): State<SharedStore>,
    Path(id): Path<String>,
    query: Result<Query<DrawQuery>, QueryRejection>,
) -> ApiResult<DrawResponse> {
    let id = parse_deck_id(&id)?;

    let Query(query) =
        query.map_err(|_| json_error(StatusCode::BAD_REQUEST, "Invalid draw count"))?;

    let count = query.count.unwrap_or(1);

    if count == 0 {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            "Count must be at least 1",
        ));
    }

    let mut store = lock_store(&store)?;

    let deck = store
        .get_mut(&id)
        .ok_or_else(|| json_error(StatusCode::NOT_FOUND, "Deck not found"))?;

    let cards = deck
        .draw(count)
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "Not enough cards remaining"))?;

    Ok(Json(DrawResponse {
        deck_id: id.to_string(),
        cards,
        remaining: deck.remaining(),
    }))
}

async fn reset_deck(
    State(store): State<SharedStore>,
    Path(id): Path<String>,
) -> ApiResult<DeckResponse> {
    let id = parse_deck_id(&id)?;
    let mut store = lock_store(&store)?;

    let deck = store
        .get_mut(&id)
        .ok_or_else(|| json_error(StatusCode::NOT_FOUND, "Deck not found"))?;

    deck.reset_and_shuffle();

    Ok(Json(DeckResponse {
        deck_id: id.to_string(),
        remaining: deck.remaining(),
    }))
}

fn build_app(store: SharedStore) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    Router::new()
        .route("/decks", post(create_deck))
        .route("/decks/{id}", get(get_deck))
        .route("/decks/{id}/draw", post(draw_cards))
        .route("/decks/{id}/reset", post(reset_deck))
        .layer(cors)
        .with_state(store)
}

pub async fn serve() -> std::io::Result<()> {
    let store = Arc::new(Mutex::new(DeckStore::new()));
    let app = build_app(store);

    let address = std::env::var("CARDINAL_BIND").unwrap_or_else(|_| "127.0.0.1:3000".to_string());

    let listener = tokio::net::TcpListener::bind(address.as_str()).await?;

    println!("Listening on http://{address}");

    axum::serve(listener, app).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use serde_json::Value;
    use tower::ServiceExt;

    async fn request(app: &Router, method: Method, uri: &str) -> (StatusCode, Value) {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();

        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();

        let json = serde_json::from_slice(&body).unwrap();

        (status, json)
    }

    async fn assert_error(
        app: &Router,
        method: Method,
        uri: &str,
        expected_status: StatusCode,
        expected_message: &str,
    ) {
        let (status, body) = request(app, method, uri).await;

        assert_eq!(status, expected_status, "Unexpected status for {uri}");
        assert_eq!(
            body["error"], expected_message,
            "Unexpected error for {uri}"
        );
    }

    #[tokio::test]
    async fn complete_deck_workflow() {
        let app = build_app(Arc::new(Mutex::new(DeckStore::new())));

        let (status, created) = request(&app, Method::POST, "/decks").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(created["remaining"], 52);

        let id = created["deck_id"].as_str().unwrap();

        let url = format!("/decks/{id}/draw?count=2");
        let (status, drawn) = request(&app, Method::POST, &url).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(drawn["cards"].as_array().unwrap().len(), 2);
        assert_eq!(drawn["remaining"], 50);

        let url = format!("/decks/{id}");
        let (status, deck) = request(&app, Method::GET, &url).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(deck["remaining"], 50);

        let url = format!("/decks/{id}/reset");
        let (status, reset) = request(&app, Method::POST, &url).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(reset["remaining"], 52);
    }

    #[tokio::test]
    async fn rejects_invalid_requests() {
        let app = build_app(Arc::new(Mutex::new(DeckStore::new())));

        let (status, created) = request(&app, Method::POST, "/decks").await;
        assert_eq!(status, StatusCode::OK);

        let id = created["deck_id"].as_str().unwrap();
        let missing_id = "00000000-0000-0000-0000-000000000000";

        assert_error(
            &app,
            Method::GET,
            "/decks/not-a-uuid",
            StatusCode::BAD_REQUEST,
            "Invalid deck ID",
        )
        .await;

        assert_error(
            &app,
            Method::GET,
            &format!("/decks/{missing_id}"),
            StatusCode::NOT_FOUND,
            "Deck not found",
        )
        .await;

        for (count, message) in [
            ("abc", "Invalid draw count"),
            ("0", "Count must be at least 1"),
            ("53", "Not enough cards remaining"),
        ] {
            let url = format!("/decks/{id}/draw?count={count}");

            assert_error(&app, Method::POST, &url, StatusCode::BAD_REQUEST, message).await;
        }

        assert_error(
            &app,
            Method::POST,
            "/decks/not-a-uuid/reset",
            StatusCode::BAD_REQUEST,
            "Invalid deck ID",
        )
        .await;

        let url = format!("/decks/{id}");
        let (status, deck) = request(&app, Method::GET, &url).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(deck["remaining"], 52);
    }

    #[tokio::test]
    async fn rejects_creation_at_capacity() {
        let mut store = DeckStore::new();

        for _ in 0..1_000 {
            store.create(false).unwrap();
        }

        let app = build_app(Arc::new(Mutex::new(store)));

        assert_error(
            &app,
            Method::POST,
            "/decks",
            StatusCode::SERVICE_UNAVAILABLE,
            "Deck capacity reached",
        )
        .await;
    }
}
