use crate::card::{Card, Rank};
use crate::deck::Deck;

#[derive(Default)]
pub struct Hand {
    cards: Vec<Card>,
}

impl Hand {
    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn score(&self) -> u16 {
        let mut total = 0;
        let mut aces = 0;

        for card in &self.cards {
            total += match card.rank {
                Rank::Ace => {
                    aces += 1;
                    11
                }
                Rank::Two => 2,
                Rank::Three => 3,
                Rank::Four => 4,
                Rank::Five => 5,
                Rank::Six => 6,
                Rank::Seven => 7,
                Rank::Eight => 8,
                Rank::Nine => 9,
                Rank::Ten | Rank::Jack | Rank::Queen | Rank::King => 10,
            };
        }

        while total > 21 && aces > 0 {
            total -= 10;
            aces -= 1;
        }

        total
    }

    pub fn is_bust(&self) -> bool {
        self.score() > 21
    }
}

pub struct BlackjackGame {
    deck: Deck,
    player: Hand,
    dealer: Hand,
    state: GameState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    PlayerTurn,
    PlayerBust,
    PlayerWin,
    DealerWin,
    Push,
}

impl BlackjackGame {
    pub fn new() -> Self {
        let mut deck = Deck::new();
        deck.shuffle();

        let mut player = Hand::default();
        let mut dealer = Hand::default();
        let cards = deck.draw(4).expect("A new deck has 52 cards");

        for (index, card) in cards.into_iter().enumerate() {
            if index % 2 == 0 {
                player.add_card(card);
            } else {
                dealer.add_card(card);
            }
        }

        Self {
            deck,
            player,
            dealer,
            state: GameState::PlayerTurn,
        }
    }

    pub fn player_hand(&self) -> &Hand {
        &self.player
    }

    pub fn dealer_hand(&self) -> &Hand {
        &self.dealer
    }

    pub fn remaining(&self) -> usize {
        self.deck.remaining()
    }

    pub fn state(&self) -> GameState {
        self.state
    }

    pub fn hit(&mut self) -> Result<Card, &'static str> {
        if self.state != GameState::PlayerTurn {
            return Err("Cannot hit after busting");
        }

        let card = self
            .deck
            .draw(1)
            .and_then(|cards| cards.into_iter().next())
            .ok_or("No cards remaining")?;

        self.player.add_card(card);

        if self.player.is_bust() {
            self.state = GameState::PlayerBust;
        }

        Ok(card)
    }

    pub fn stand(&mut self) -> Result<GameState, &'static str> {
        if self.state != GameState::PlayerTurn {
            return Err("Round is already finished");
        }

        while self.dealer.score() < 17 {
            let card = self
                .deck
                .draw(1)
                .and_then(|cards| cards.into_iter().next())
                .ok_or("No cards remaining")?;

            self.dealer.add_card(card);
        }

        self.state = if self.dealer.is_bust() || self.player.score() > self.dealer.score() {
            GameState::PlayerWin
        } else if self.player.score() < self.dealer.score() {
            GameState::DealerWin
        } else {
            GameState::Push
        };

        Ok(self.state)
    }
}

impl Default for BlackjackGame {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::Suit;

    #[test]
    fn handles_multiple_aces_and_busts() {
        let mut hand = Hand::default();

        hand.add_card(Card {
            rank: Rank::Ace,
            suit: Suit::Spades,
        });
        hand.add_card(Card {
            rank: Rank::Ace,
            suit: Suit::Hearts,
        });
        hand.add_card(Card {
            rank: Rank::Nine,
            suit: Suit::Clubs,
        });
        assert_eq!(hand.score(), 21);
        assert!(!hand.is_bust());

        hand.add_card(Card {
            rank: Rank::Five,
            suit: Suit::Diamonds,
        });
        assert_eq!(hand.score(), 16);

        hand.add_card(Card {
            rank: Rank::King,
            suit: Suit::Spades,
        });
        assert_eq!(hand.score(), 26);
        assert!(hand.is_bust());
    }

    #[test]
    fn new_game_deals_four_unique_cards() {
        use std::collections::HashSet;

        let game = BlackjackGame::new();
        assert_eq!(game.player_hand().cards().len(), 2);
        assert_eq!(game.dealer_hand().cards().len(), 2);
        assert_eq!(game.remaining(), 48);

        let dealt_cards: HashSet<Card> = game
            .player_hand()
            .cards()
            .iter()
            .chain(game.dealer_hand().cards())
            .copied()
            .collect();

        assert_eq!(dealt_cards.len(), 4);
    }

    #[test]
    fn hitting_draws_cards_and_stops_after_bust() {
        let mut game = BlackjackGame::new();
        assert_eq!(game.state(), GameState::PlayerTurn);

        let initial_cards = game.player_hand().cards().len();
        let initial_remaining = game.remaining();
        game.hit().unwrap();
        assert_eq!(game.player_hand().cards().len(), initial_cards + 1);
        assert_eq!(game.remaining(), initial_remaining - 1);

        while game.state() == GameState::PlayerTurn {
            game.hit().unwrap();
        }
        assert_eq!(game.state(), GameState::PlayerBust);

        let remaining = game.remaining();
        assert!(game.hit().is_err());
        assert_eq!(game.remaining(), remaining);
    }

    #[test]
    fn standing_finishes_round_and_prevents_more_actions() {
        let mut game = BlackjackGame::new();

        game.player = Hand::default();
        game.dealer = Hand::default();

        game.player.add_card(Card {
            rank: Rank::King,
            suit: Suit::Spades,
        });
        game.player.add_card(Card {
            rank: Rank::Queen,
            suit: Suit::Hearts,
        });
        game.dealer.add_card(Card {
            rank: Rank::Ten,
            suit: Suit::Clubs,
        });
        game.dealer.add_card(Card {
            rank: Rank::Eight,
            suit: Suit::Diamonds,
        });

        let remaining = game.remaining();

        assert_eq!(game.stand(), Ok(GameState::PlayerWin));
        assert_eq!(game.state(), GameState::PlayerWin);
        assert_eq!(game.remaining(), remaining);

        assert!(game.hit().is_err());
        assert!(game.stand().is_err());
        assert_eq!(game.remaining(), remaining);
    }
}
