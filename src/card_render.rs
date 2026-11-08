use crate::card::{Card, Suit};

pub fn render_card(card: Card) -> [String; 7] {
    let label = card.to_string();

    let suit = match card.suit {
        Suit::Clubs => "♣",
        Suit::Diamonds => "♦",
        Suit::Hearts => "♥",
        Suit::Spades => "♠",
    };

    let is_red = matches!(card.suit, Suit::Diamonds | Suit::Hearts);

    let colorize = |text: &str| {
        if is_red {
            format!("\x1b[91m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    };

    let padding = " ".repeat(7 - label.chars().count());

    [
        "┌───────┐".to_string(),
        format!("│{}{}│", colorize(&label), padding),
        "│       │".to_string(),
        format!("│   {}   │", colorize(suit)),
        "│       │".to_string(),
        format!("│{}{}│", padding, colorize(&label)),
        "└───────┘".to_string(),
    ]
}

pub fn render_cards(cards: &[Card]) -> String {
    if cards.is_empty() {
        return String::new();
    }

    let rendered: Vec<[String; 7]> = cards.iter().copied().map(render_card).collect();

    let mut lines = Vec::with_capacity(7);

    for row in 0..7 {
        let line = rendered
            .iter()
            .map(|card| card[row].as_str())
            .collect::<Vec<_>>()
            .join(" ");

        lines.push(line);
    }

    lines.join("\n")
}

pub fn render_dealer_opening(visible: Card) -> String {
    let front = render_card(visible);

    let back = [
        "┌───────┐",
        "│░░░░░░░│",
        "│░┌───┐░│",
        "│░│ ■ │░│",
        "│░└───┘░│",
        "│░░░░░░░│",
        "└───────┘",
    ];

    front
        .iter()
        .zip(back.iter())
        .map(|(front_row, back_row)| format!("{front_row} {back_row}"))
        .collect::<Vec<_>>()
        .join("\n")
}
