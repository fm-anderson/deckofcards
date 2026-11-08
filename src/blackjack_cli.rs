use std::io::{self, Write};

use deckofcards::blackjack::{BlackjackGame, GameState};

pub fn play() -> io::Result<()> {
    println!("Welcome to Blackjack!\n");

    loop {
        let mut game = BlackjackGame::new();

        println!("Dealer: {} [Hidden]", game.dealer_hand().cards()[0]);

        println!(
            "You: {} (Score: {})",
            game.player_hand(),
            game.player_hand().score()
        );

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
                        println!("\nYou drew: {card}");
                        println!(
                            "Your hand: {} (Score: {})",
                            game.player_hand(),
                            game.player_hand().score()
                        );
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

        println!("\n--- Round Complete ---");

        println!(
            "Dealer: {} (Score: {})",
            game.dealer_hand(),
            game.dealer_hand().score()
        );

        println!(
            "You: {} (Score: {})",
            game.player_hand(),
            game.player_hand().score()
        );

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
