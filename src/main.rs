mod card;
mod deck;

use deck::Deck;

fn main() {
    let mut deck = Deck::new();
    println!("Initial cards: {}", deck.remaining());

    deck.shuffle();

    if let Some(cards) = deck.draw(2) {
        println!("Drawn cards: {cards:?}");
    }

    println!("Remaining cards: {}", deck.remaining());
}
