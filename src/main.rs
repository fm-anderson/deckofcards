mod blackjack_cli;

use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use deckofcards::{card::Card, deck_store::DeckStore};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

type SharedStore = Arc<Mutex<DeckStore>>;

#[derive(Serialize)]
struct DeckResponse {
    deck_id: String,
    remaining: usize,
}

#[derive(Deserialize)]
struct DrawQuery {
    count: Option<usize>,
}

#[derive(Serialize)]
struct DrawResponse {
    deck_id: String,
    cards: Vec<Card>,
    remaining: usize,
}

async fn create_deck(State(store): State<SharedStore>) -> Result<Json<DeckResponse>, StatusCode> {
    let mut store = store
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let id = store.create(true);

    let remaining = store
        .get(&id)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .remaining();

    Ok(Json(DeckResponse {
        deck_id: id.to_string(),
        remaining,
    }))
}

async fn get_deck(
    State(store): State<SharedStore>,
    Path(id): Path<String>,
) -> Result<Json<DeckResponse>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let store = store
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let deck = store.get(&id).ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(DeckResponse {
        deck_id: id.to_string(),
        remaining: deck.remaining(),
    }))
}

async fn draw_cards(
    State(store): State<SharedStore>,
    Path(id): Path<String>,
    Query(query): Query<DrawQuery>,
) -> Result<Json<DrawResponse>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let count = query.count.unwrap_or(1);

    if count == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut store = store
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let deck = store.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;

    let cards = deck.draw(count).ok_or(StatusCode::BAD_REQUEST)?;

    Ok(Json(DrawResponse {
        deck_id: id.to_string(),
        cards,
        remaining: deck.remaining(),
    }))
}

async fn reset_deck(
    State(store): State<SharedStore>,
    Path(id): Path<String>,
) -> Result<Json<DeckResponse>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut store = store
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let deck = store.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;

    deck.reset_and_shuffle();

    Ok(Json(DeckResponse {
        deck_id: id.to_string(),
        remaining: deck.remaining(),
    }))
}

fn build_app(store: SharedStore) -> Router {
    Router::new()
        .route("/decks", post(create_deck))
        .route("/decks/{id}", get(get_deck))
        .route("/decks/{id}/draw", post(draw_cards))
        .route("/decks/{id}/reset", post(reset_deck))
        .with_state(store)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("blackjack") => blackjack_cli::play(),

        None | Some("serve") => {
            let store = Arc::new(Mutex::new(DeckStore::new()));

            let app = build_app(store);

            let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

            println!("Listening on http://127.0.0.1:3000");

            axum::serve(listener, app).await
        }

        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Usage: deckofcards [serve|blackjack]",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request},
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

    #[tokio::test]
    async fn complete_deck_workflow() {
        let store = Arc::new(Mutex::new(DeckStore::new()));
        let app = build_app(store);
        let (status, created) = request(&app, Method::POST, "/decks").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(created["remaining"], 52);

        let id = created["deck_id"].as_str().unwrap();
        let draw_url = format!("/decks/{id}/draw?count=2");
        let (status, drawn) = request(&app, Method::POST, &draw_url).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(drawn["cards"].as_array().unwrap().len(), 2);
        assert_eq!(drawn["remaining"], 50);

        let deck_url = format!("/decks/{id}");
        let (status, deck) = request(&app, Method::GET, &deck_url).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(deck["remaining"], 50);

        let reset_url = format!("/decks/{id}/reset");
        let (status, reset) = request(&app, Method::POST, &reset_url).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(reset["remaining"], 52);
    }
}
