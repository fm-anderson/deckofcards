# Deck of Cards

A Rust application providing a REST API for managing playing-card decks and an interactive terminal Blackjack game.

Both interfaces use the same underlying deck and card implementations.

## Features

- Standard 52-card decks
- Card shuffling and drawing
- Multiple independently managed decks through the REST API
- Interactive terminal Blackjack
- Blackjack scoring with Ace handling (1 or 11)
- Natural Blackjack detection
- Dealer draws until reaching at least 17
- Multiple Blackjack rounds in one session

## Requirements

- Rust and Cargo

## Getting Started

Clone the repository:

```bash
git clone https://github.com/fm-anderson/deckofcards.git
cd deckofcards
```

Build the application:

```bash
cargo build --release
```

The executable will be available at `target/release/deckofcards`.

## Playing Blackjack

Start the terminal game:

```bash
cargo run -- blackjack
```

Or use the release executable:

```bash
./target/release/deckofcards blackjack
```

During a round, enter:

- `h` — Hit (draw another card)
- `s` — Stand (finish your turn)
- `q` — Quit

The game automatically resolves natural Blackjacks and finishes the player's turn when their score reaches exactly 21.

After each round, you can choose to play again.

The game uses simplified Blackjack rules, with no betting, splitting, doubling down, or insurance. Each round starts with a freshly shuffled deck.

## REST API

Start the HTTP server:

```bash
cargo run -- serve
```

The server listens on `http://127.0.0.1:3000`.

### Endpoints

| Method | Endpoint                   | Description                |
| ------ | -------------------------- | -------------------------- |
| POST   | `/decks`                   | Create a new deck          |
| GET    | `/decks/{id}`              | Get deck information       |
| POST   | `/decks/{id}/draw?count=2` | Draw cards from a deck     |
| POST   | `/decks/{id}/reset`        | Restore and shuffle a deck |

### Examples

Create a deck:

```bash
curl -X POST http://127.0.0.1:3000/decks
```

Example response:

```json
{
  "deck_id": "ac79be6e-9e76-47af-b1ff-f364a054086b",
  "remaining": 52
}
```

Use the returned `deck_id` for subsequent requests.

Get deck information:

```bash
curl http://127.0.0.1:3000/decks/YOUR_DECK_ID
```

Draw two cards:

```bash
curl -X POST "http://127.0.0.1:3000/decks/YOUR_DECK_ID/draw?count=2"
```

Reset the deck:

```bash
curl -X POST http://127.0.0.1:3000/decks/YOUR_DECK_ID/reset
```

The server keeps decks in memory. Restarting the server removes previously created decks.

## Development

Run the tests:

```bash
cargo test
```

Check formatting:

```bash
cargo fmt --check
```

Run Clippy:

```bash
cargo clippy --all-targets -- -D warnings
```

Build an optimized executable:

```bash
cargo build --release
```

## Project Structure

```text
src/
├── main.rs           # REST API and command dispatch
├── lib.rs            # Library modules
├── card.rs           # Card, Rank, and Suit
├── deck.rs           # Deck creation, shuffling, and drawing
├── deck_store.rs     # In-memory deck management
├── blackjack.rs      # Blackjack rules and game state
└── blackjack_cli.rs  # Interactive terminal interface
```
