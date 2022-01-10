use colored::*;

pub const NUM_CARDS_DECK: u8 = 52;
pub const NUM_CARDS_SUIT: u8 = 13;

pub type Card = u8;

const CLUBS_ACE: Card = 0;
const CLUBS_2: Card = 1;
const CLUBS_3: Card = 2;
const CLUBS_4: Card = 3;
const CLUBS_5: Card = 4;
const CLUBS_6: Card = 5;
const CLUBS_7: Card = 6;
const CLUBS_8: Card = 7;
const CLUBS_9: Card = 8;
const CLUBS_10: Card = 9;
const CLUBS_JACK: Card = 10;
const CLUBS_QUEEN: Card = 11;
const CLUBS_KING: Card = 12;

const DIAMONDS_ACE: Card = 13;
const DIAMONDS_2: Card = 14;
const DIAMONDS_3: Card = 15;
const DIAMONDS_4: Card = 16;
const DIAMONDS_5: Card = 17;
const DIAMONDS_6: Card = 18;
const DIAMONDS_7: Card = 19;
const DIAMONDS_8: Card = 20;
const DIAMONDS_9: Card = 21;
const DIAMONDS_10: Card = 22;
const DIAMONDS_JACK: Card = 23;
const DIAMONDS_QUEEN: Card = 24;
const DIAMONDS_KING: Card = 25;

const SPADES_ACE: Card = 26;
const SPADES_2: Card = 27;
const SPADES_3: Card = 28;
const SPADES_4: Card = 29;
const SPADES_5: Card = 30;
const SPADES_6: Card = 31;
const SPADES_7: Card = 32;
const SPADES_8: Card = 33;
const SPADES_9: Card = 34;
const SPADES_10: Card = 35;
const SPADES_JACK: Card = 36;
const SPADES_QUEEN: Card = 37;
const SPADES_KING: Card = 38;

const HEARTS_ACE: Card = 39;
const HEARTS_2: Card = 40;
const HEARTS_3: Card = 41;
const HEARTS_4: Card = 42;
const HEARTS_5: Card = 43;
const HEARTS_6: Card = 44;
const HEARTS_7: Card = 45;
const HEARTS_8: Card = 46;
const HEARTS_9: Card = 47;
const HEARTS_10: Card = 48;
const HEARTS_JACK: Card = 49;
const HEARTS_QUEEN: Card = 50;
const HEARTS_KING: Card = 51;

pub fn is_clubs(card: Card) -> bool {
    card <= CLUBS_KING
}

pub fn is_diamonds(card: Card) -> bool {
    card >= DIAMONDS_ACE && card <= DIAMONDS_KING
}

pub fn is_hearts(card: Card) -> bool {
    card >= HEARTS_ACE && card <= HEARTS_KING
}

pub fn is_spades(card: Card) -> bool {
    card >= SPADES_ACE && card <= SPADES_KING
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
        match card {
            CLUBS_ACE..=CLUBS_KING => card - CLUBS_ACE + 1,
            DIAMONDS_ACE..=DIAMONDS_KING => card - DIAMONDS_ACE + 1,
            HEARTS_ACE..=HEARTS_KING => card - HEARTS_ACE + 1,
            SPADES_ACE..=SPADES_KING => card - SPADES_ACE + 1,
            _ => unreachable!(),
        }
    }
}

pub fn are_card_ranks_sequential(bottom: Card, top: Card) -> bool {
    card_rank(top) > 1 && card_rank(bottom) == (card_rank(top) - 1)
}

pub fn are_card_colors_different(card1: Card, card2: Card) -> bool {
    is_red(card1) != is_red(card2)
}

pub fn are_card_suits_the_same(card1: Card, card2: Card) -> bool {
    let card_rank_1 = card_rank(card1);
    let card_rank_2 = card_rank(card2);
    card_rank_1.abs_diff(card_rank_2) <= 12 && !are_card_colors_different(card1, card2)
}

pub fn suit_rank(card: Card) -> u8 {
    card / NUM_CARDS_SUIT
}

pub fn is_king(card: Card) -> bool {
    card % 13 == 12
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
        if is_red(card) {
            ret_string.red().to_string()
        } else {
            ret_string.yellow().to_string()
        }
    }
}
