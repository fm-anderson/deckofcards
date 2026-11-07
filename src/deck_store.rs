use crate::deck::Deck;
use std::collections::HashMap;
use uuid::Uuid;

pub struct DeckStore {
    decks: HashMap<Uuid, Deck>,
}

impl DeckStore {
    pub fn new() -> Self {
        Self {
            decks: HashMap::new(),
        }
    }

    pub fn create(&mut self, shuffle: bool) -> Uuid {
        let mut deck = Deck::new();

        if shuffle {
            deck.shuffle();
        }

        let id = Uuid::new_v4();
        self.decks.insert(id, deck);

        id
    }

    pub fn get(&self, id: &Uuid) -> Option<&Deck> {
        self.decks.get(id)
    }

    pub fn get_mut(&mut self, id: &Uuid) -> Option<&mut Deck> {
        self.decks.get_mut(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decks_are_independent() {
        let mut store = DeckStore::new();

        let first_id = store.create(false);
        let second_id = store.create(false);
        assert_ne!(first_id, second_id);

        // drawing from one deck should not affect another.
        let drawn = store.get_mut(&first_id).unwrap().draw(3).unwrap();
        assert_eq!(drawn.len(), 3);
        assert_eq!(store.get(&first_id).unwrap().remaining(), 49);
        assert_eq!(store.get(&second_id).unwrap().remaining(), 52);

        // unknown deck ID returns None.
        assert!(store.get(&Uuid::nil()).is_none());
    }
}
