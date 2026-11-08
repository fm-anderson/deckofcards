use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use uuid::Uuid;

use crate::cards::deck::Deck;

const DECK_TTL: Duration = Duration::from_secs(60 * 60);
const MAX_DECKS: usize = 1_000;

struct StoredDeck {
    deck: Deck,
    last_used: Instant,
}

#[derive(Default)]
pub struct DeckStore {
    decks: HashMap<Uuid, StoredDeck>,
}

impl DeckStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&mut self, shuffle: bool) -> Option<Uuid> {
        self.remove_expired();

        if self.decks.len() >= MAX_DECKS {
            return None;
        }

        let mut deck = Deck::new();

        if shuffle {
            deck.shuffle();
        }

        let id = Uuid::new_v4();

        self.decks.insert(
            id,
            StoredDeck {
                deck,
                last_used: Instant::now(),
            },
        );

        Some(id)
    }

    pub fn get(&mut self, id: &Uuid) -> Option<&Deck> {
        self.get_mut(id).map(|deck| &*deck)
    }

    pub fn get_mut(&mut self, id: &Uuid) -> Option<&mut Deck> {
        self.remove_expired();

        let stored = self.decks.get_mut(id)?;
        stored.last_used = Instant::now();

        Some(&mut stored.deck)
    }

    fn remove_expired(&mut self) {
        self.decks
            .retain(|_, stored| stored.last_used.elapsed() < DECK_TTL);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decks_are_independent() {
        let mut store = DeckStore::new();

        let first_id = store.create(false).unwrap();
        let second_id = store.create(false).unwrap();
        assert_ne!(first_id, second_id);

        let drawn = store.get_mut(&first_id).unwrap().draw(3).unwrap();
        assert_eq!(drawn.len(), 3);
        assert_eq!(store.get(&first_id).unwrap().remaining(), 49);
        assert_eq!(store.get(&second_id).unwrap().remaining(), 52);
        assert!(store.get(&Uuid::nil()).is_none());
    }

    #[test]
    fn expires_inactive_decks_and_refreshes_active_ones() {
        let mut store = DeckStore::new();

        let expired_id = store.create(false).unwrap();
        let active_id = store.create(false).unwrap();

        // accessing an active deck refreshes its timestamp.
        let old_timestamp = Instant::now() - Duration::from_secs(59 * 60);
        store.decks.get_mut(&active_id).unwrap().last_used = old_timestamp;

        assert!(store.get(&active_id).is_some());
        assert!(store.decks.get(&active_id).unwrap().last_used > old_timestamp);

        // inactive decks expire after one hour.
        store.decks.get_mut(&expired_id).unwrap().last_used =
            Instant::now() - DECK_TTL - Duration::from_secs(1);

        store.remove_expired();

        assert!(store.get(&expired_id).is_none());
        assert!(store.get(&active_id).is_some());
    }

    #[test]
    fn enforces_capacity_and_reuses_expired_space() {
        let mut store = DeckStore::new();

        for _ in 0..MAX_DECKS {
            assert!(store.create(false).is_some());
        }

        assert!(store.create(false).is_none());
        let id = *store.decks.keys().next().unwrap();

        store.decks.get_mut(&id).unwrap().last_used =
            Instant::now() - DECK_TTL - Duration::from_secs(1);

        assert!(store.create(false).is_some());
        assert!(store.create(false).is_none());
        assert_eq!(store.decks.len(), MAX_DECKS);
    }
}
