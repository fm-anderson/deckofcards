use std::fmt;

use crate::cards::card::{Card, Rank};
use crate::cards::deck::Deck;

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

    pub fn is_blackjack(&self) -> bool {
        self.cards.len() == 2 && self.score() == 21
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, card) in self.cards.iter().enumerate() {
            if index > 0 {
                write!(f, " ")?;
            }

            write!(f, "{card}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    PlayerTurn,
    PlayerBust,
    PlayerWin,
    DealerWin,
    Push,
}

pub struct BlackjackGame {
    deck: Deck,
    player: Hand,
    dealer: Hand,
    state: GameState,
}

impl BlackjackGame {
    pub fn new() -> Self {
        let mut deck = Deck::new();
        deck.shuffle();

        let mut player = Hand::default();
        let mut dealer = Hand::default();

        let cards = deck.draw(4).expect("A new deck has 52 cards");

        for pair in cards.as_chunks::<2>().0 {
            player.add_card(pair[0]);
            dealer.add_card(pair[1]);
        }

        let mut game = Self {
            deck,
            player,
            dealer,
            state: GameState::PlayerTurn,
        };

        game.resolve_naturals();
        game
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
            return Err("Round is already finished");
        }

        let card = self.draw_card()?;
        self.player.add_card(card);

        let score = self.player.score();

        if score > 21 {
            self.state = GameState::PlayerBust;
        } else if score == 21 {
            self.stand()?;
        }

        Ok(card)
    }

    pub fn stand(&mut self) -> Result<GameState, &'static str> {
        if self.state != GameState::PlayerTurn {
            return Err("Round is already finished");
        }

        while self.dealer.score() < 17 {
            let card = self.draw_card()?;
            self.dealer.add_card(card);
        }

        let player_score = self.player.score();
        let dealer_score = self.dealer.score();

        self.state = if dealer_score > 21 || player_score > dealer_score {
            GameState::PlayerWin
        } else if player_score < dealer_score {
            GameState::DealerWin
        } else {
            GameState::Push
        };

        Ok(self.state)
    }

    fn draw_card(&mut self) -> Result<Card, &'static str> {
        self.deck
            .draw(1)
            .and_then(|cards| cards.into_iter().next())
            .ok_or("No cards remaining")
    }

    fn resolve_naturals(&mut self) {
        self.state = match (self.player.is_blackjack(), self.dealer.is_blackjack()) {
            (true, true) => GameState::Push,
            (true, false) => GameState::PlayerWin,
            (false, true) => GameState::DealerWin,
            (false, false) => GameState::PlayerTurn,
        };
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
    use crate::cards::card::Suit;
    use std::collections::HashSet;

    fn hand(ranks: &[Rank]) -> Hand {
        Hand {
            cards: ranks
                .iter()
                .map(|&rank| Card {
                    rank,
                    suit: Suit::Spades,
                })
                .collect(),
        }
    }

    #[test]
    fn scores_aces_and_detects_busts() {
        let mut hand = hand(&[Rank::Ace, Rank::Ace, Rank::Nine]);

        assert_eq!(hand.score(), 21);
        assert!(!hand.is_blackjack());

        hand.add_card(Card {
            rank: Rank::Five,
            suit: Suit::Hearts,
        });

        assert_eq!(hand.score(), 16);

        hand.add_card(Card {
            rank: Rank::King,
            suit: Suit::Clubs,
        });

        assert_eq!(hand.score(), 26);
        assert!(hand.is_bust());
    }

    #[test]
    fn deals_new_games_and_resolves_natural_blackjacks() {
        let mut game = BlackjackGame::new();

        assert_eq!(game.player_hand().cards().len(), 2);
        assert_eq!(game.dealer_hand().cards().len(), 2);
        assert_eq!(game.remaining(), 48);

        let cards: HashSet<Card> = game
            .player_hand()
            .cards()
            .iter()
            .chain(game.dealer_hand().cards())
            .copied()
            .collect();

        assert_eq!(cards.len(), 4);

        // player blackjack.
        game.player = hand(&[Rank::Ace, Rank::King]);
        game.dealer = hand(&[Rank::Ten, Rank::Nine]);
        game.resolve_naturals();

        assert_eq!(game.state(), GameState::PlayerWin);
        assert!(game.hit().is_err());

        // both players have blackjack.
        game.dealer = hand(&[Rank::Ace, Rank::Queen]);
        game.resolve_naturals();

        assert_eq!(game.state(), GameState::Push);

        // dealer blackjack.
        game.player = hand(&[Rank::Nine, Rank::Ten]);
        game.resolve_naturals();

        assert_eq!(game.state(), GameState::DealerWin);
    }

    #[test]
    fn hitting_updates_hand_and_finishes_round() {
        let mut game = BlackjackGame::new();

        // unshuffled deck lets us control the next card.
        game.deck = Deck::new();
        game.player = hand(&[Rank::King, Rank::Nine]);
        game.state = GameState::PlayerTurn;

        let remaining = game.remaining();

        game.hit().unwrap();

        assert_eq!(game.player.score(), 29);
        assert_eq!(game.remaining(), remaining - 1);
        assert_eq!(game.state(), GameState::PlayerBust);
        assert!(game.hit().is_err());

        // hitting exactly 21 automatically finishes the round.
        game.deck = Deck::new();
        game.player = hand(&[Rank::Six, Rank::Five]);
        game.dealer = hand(&[Rank::Ten, Rank::Eight]);
        game.state = GameState::PlayerTurn;

        game.hit().unwrap();

        assert_eq!(game.player.score(), 21);
        assert!(!game.player.is_blackjack());
        assert_eq!(game.state(), GameState::PlayerWin);
        assert!(game.hit().is_err());
        assert!(game.stand().is_err());
    }

    #[test]
    fn standing_finishes_round_and_prevents_more_actions() {
        let mut game = BlackjackGame::new();

        game.player = hand(&[Rank::King, Rank::Queen]);
        game.dealer = hand(&[Rank::Ten, Rank::Eight]);
        game.state = GameState::PlayerTurn;

        let remaining = game.remaining();

        assert_eq!(game.stand(), Ok(GameState::PlayerWin));
        assert_eq!(game.remaining(), remaining);

        assert!(game.hit().is_err());
        assert!(game.stand().is_err());
        assert_eq!(game.remaining(), remaining);
    }
}
