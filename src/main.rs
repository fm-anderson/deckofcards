mod card;
mod deck;
mod deck_store;

use deck_store::DeckStore;

fn main() {
    let mut store = DeckStore::new();

    let first_id = store.create(true);
    let second_id = store.create(false);

    println!("First deck ID: {first_id}");
    println!("Second deck ID: {second_id}");

    // Draw two cards from the first deck.
    let first_deck = store.get_mut(&first_id).unwrap();
    let drawn = first_deck.draw(2).unwrap();

    println!("Drawn cards: {drawn:?}");

    println!(
        "First deck remaining: {}",
        store.get(&first_id).unwrap().remaining()
    );

    println!(
        "Second deck remaining: {}",
        store.get(&second_id).unwrap().remaining()
    );
}
