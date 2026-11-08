mod api;
mod cli;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("blackjack") => cli::blackjack::play(),
        None | Some("serve") => api::serve().await,
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Usage: deckofcards [serve|blackjack]",
        )),
    }
}
