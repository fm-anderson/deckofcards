use crate::cards::card::{Card, Rank, Suit};
use rand::seq::SliceRandom;

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let suits = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];
        let ranks = [
            Rank::Ace,
            Rank::Two,
            Rank::Three,
            Rank::Four,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
            Rank::Ten,
            Rank::Jack,
            Rank::Queen,
            Rank::King,
        ];

        let mut cards: Vec<Card> = Vec::with_capacity(52);

        for suit in suits {
            for rank in ranks {
                cards.push(Card { rank, suit });
            }
        }

        Self { cards }
    }

    pub fn remaining(&self) -> usize {
        self.cards.len()
    }

    pub fn draw(&mut self, count: usize) -> Option<Vec<Card>> {
        if count > self.cards.len() {
            return None;
        }

        let split_at = self.cards.len() - count;
        Some(self.cards.split_off(split_at))
    }

    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut rand::rng());
    }

    pub fn reset_and_shuffle(&mut self) {
        *self = Self::new();
        self.shuffle();
    }
}

impl Default for Deck {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drawing_cards_updates_deck_correctly() {
        let mut deck = Deck::new();
        assert_eq!(deck.remaining(), 52);

        let drawn = deck.draw(2).expect("Drawing two cards should succeed");
        assert_eq!(drawn.len(), 2);
        assert_eq!(deck.remaining(), 50);
        assert!(deck.draw(51).is_none());
        assert_eq!(deck.remaining(), 50);
    }

    #[test]
    fn shuffle_preserves_all_cards() {
        use std::collections::HashSet;

        let mut original = Deck::new();
        let mut shuffled = Deck::new();

        shuffled.shuffle();

        let expected: HashSet<Card> = original.draw(52).unwrap().into_iter().collect();
        let actual: HashSet<Card> = shuffled.draw(52).unwrap().into_iter().collect();

        assert_eq!(actual.len(), 52);
        assert_eq!(actual, expected);
        assert_eq!(shuffled.remaining(), 0);
    }

    #[test]
    fn reset_restores_full_deck() {
        let mut deck = Deck::new();

        deck.draw(10).unwrap();
        assert_eq!(deck.remaining(), 42);

        deck.reset_and_shuffle();
        assert_eq!(deck.remaining(), 52);

        // restored deck should allow drawing all 52 cards.
        assert_eq!(deck.draw(52).unwrap().len(), 52);
        assert_eq!(deck.remaining(), 0);
    }
}
