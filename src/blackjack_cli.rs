use deckofcards::card_render::{render_cards, render_dealer_opening};
use std::io::{self, Write};

use deckofcards::blackjack::{BlackjackGame, GameState};

fn draw_table(game: &BlackjackGame, reveal_dealer: bool) {
    // Clear the terminal and move the cursor to the top-left.
    print!("\x1b[2J\x1b[H");

    println!("♠ CARDINAL — BLACKJACK\n");

    if reveal_dealer {
        println!("DEALER — Score: {}", game.dealer_hand().score());
        println!("{}", render_cards(game.dealer_hand().cards()));
    } else {
        println!("DEALER");
        println!("{}", render_dealer_opening(game.dealer_hand().cards()[0]));
    }

    println!("\nYOUR HAND — Score: {}", game.player_hand().score());
    println!("{}", render_cards(game.player_hand().cards()));
}

pub fn play() -> io::Result<()> {
    loop {
        let mut game = BlackjackGame::new();
        draw_table(&game, false);

        while game.state() == GameState::PlayerTurn {
            print!("\n[h] Hit  [s] Stand  [q] Quit > ");
            io::stdout().flush()?;

            let mut input = String::new();

            if io::stdin().read_line(&mut input)? == 0 {
                return Ok(());
            }

            match input.trim().to_ascii_lowercase().as_str() {
                "h" | "hit" => match game.hit() {
                    Ok(card) => {
                        if game.state() == GameState::PlayerTurn {
                            draw_table(&game, false);
                            println!("\nYou drew: {card}");
                        }
                    }
                    Err(error) => println!("Error: {error}"),
                },

                "s" | "stand" => {
                    if let Err(error) = game.stand() {
                        println!("Error: {error}");
                    }
                }

                "q" | "quit" => {
                    println!("Goodbye!");
                    return Ok(());
                }

                _ => println!("Invalid action. Enter h, s, or q."),
            }
        }

        draw_table(&game, true);
        println!("\n--- Round Complete ---");

        match game.state() {
            GameState::PlayerBust => println!("You busted! Dealer wins."),
            GameState::PlayerWin => println!("You win!"),
            GameState::DealerWin => println!("Dealer wins!"),
            GameState::Push => println!("It's a tie!"),
            GameState::PlayerTurn => {}
        }

        print!("\nPlay again? [y/N] > ");
        io::stdout().flush()?;

        let mut input = String::new();

        if io::stdin().read_line(&mut input)? == 0 {
            break;
        }

        match input.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" => println!(),
            _ => break,
        }
    }

    println!("Goodbye!");
    Ok(())
}
