use colored::*;

pub const NUM_CARDS_DECK: u8 = 52;
pub const NUM_CARDS_SUIT: u8 = 13;

pub type Card = u8;

fn abs_diff(a: u8, b: u8) -> u8 {
    if a > b {
        a - b
    } else {
        b - a
    }
}

pub trait IsCard {
    fn index(&self) -> u8;
    fn is_face_up(&self) -> bool;
    fn face_up(&self) -> Self;
    fn face_down(&self) -> Self;
}

impl IsCard for Card {
    fn index(&self) -> u8 {
        self & 0b00111111
    }
    fn is_face_up(&self) -> bool {
        (self & 0b01000000) == 0b00000000
    }
    fn face_up(&self) -> Self {
        *self & 0b10111111
    }
    fn face_down(&self) -> Self {
        *self | 0b01000000
    }
}

pub const CLUBS_ACE: Card = 0;
pub const CLUBS_KING: Card = 12;

pub const DIAMONDS_ACE: Card = 13;
pub const DIAMONDS_KING: Card = 25;

pub const SPADES_ACE: Card = 26;
pub const SPADES_KING: Card = 38;

pub const HEARTS_ACE: Card = 39;
pub const HEARTS_KING: Card = 51;

pub fn is_clubs(card: Card) -> bool {
    card.index() <= CLUBS_KING
}

pub fn is_diamonds(card: Card) -> bool {
    card.index() >= DIAMONDS_ACE && card.index() <= DIAMONDS_KING
}

pub fn is_hearts(card: Card) -> bool {
    card.index() >= HEARTS_ACE && card.index() <= HEARTS_KING
}

pub fn is_spades(card: Card) -> bool {
    card.index() >= SPADES_ACE && card.index() <= SPADES_KING
}

pub fn is_red(card: Card) -> bool {
    is_diamonds(card) || is_hearts(card)
}

pub fn is_yellow(card: Card) -> bool {
    !is_red(card)
}

pub fn card_rank(card: Card) -> u8 {
    if card == u8::MAX {
        0
    } else {
        let card_index = card.index();
        match card_index {
            CLUBS_ACE..=CLUBS_KING => card_index - CLUBS_ACE + 1,
            DIAMONDS_ACE..=DIAMONDS_KING => card_index - DIAMONDS_ACE + 1,
            HEARTS_ACE..=HEARTS_KING => card_index - HEARTS_ACE + 1,
            SPADES_ACE..=SPADES_KING => card_index - SPADES_ACE + 1,
            _ => unreachable!(),
        }
    }
}

pub fn are_card_ranks_ascending(bottom: Card, top: Card) -> bool {
    card_rank(top) != 0 && card_rank(bottom) == (card_rank(top) - 1)
}

pub fn are_card_ranks_descending(bottom: Card, top: Card) -> bool {
    card_rank(bottom) == (card_rank(top) + 1)
}

pub fn are_card_colors_different(card1: Card, card2: Card) -> bool {
    is_red(card1) != is_red(card2)
}

pub fn are_card_suits_the_same(card1: Card, card2: Card) -> bool {
    let card_rank_1 = card_rank(card1);
    let card_rank_2 = card_rank(card2);
    abs_diff(card_rank_1, card_rank_2) <= 12 && !are_card_colors_different(card1, card2)
}

pub fn suit_rank(card: Card) -> u8 {
    card.index() / NUM_CARDS_SUIT
}

pub fn is_king(card: Card) -> bool {
    card.index() % 13 == 12
}

pub fn ranking_of_kings() -> usize {
    13 * 4
}

pub fn pretty_string(card: Card) -> String {
    if card == u8::MAX {
        " ".to_string()
    } else {
        let suit_rank = suit_rank(card);
        let card_rank = card_rank(card);
        let suit_string = match suit_rank {
            0 => "C",
            1 => "D",
            2 => "S",
            3 => "H",
            _ => unreachable!(),
        };
        let rank_string = match card_rank {
            1 => "A",
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            6 => "6",
            7 => "7",
            8 => "8",
            9 => "9",
            10 => "10",
            11 => "J",
            12 => "Q",
            13 => "K",
            _ => unreachable!(),
        };
        let ret_string = format!("{}{}", suit_string, rank_string);
        let mut colored = if is_red(card) {
            ret_string.red()
        } else {
            ret_string.yellow()
        };
        if !card.is_face_up() {
            colored = colored.dimmed();
        }
        colored.to_string()
    }
}
