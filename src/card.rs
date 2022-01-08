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

const HEARTS_ACE: Card = 26;
const HEARTS_2: Card = 27;
const HEARTS_3: Card = 28;
const HEARTS_4: Card = 29;
const HEARTS_5: Card = 30;
const HEARTS_6: Card = 31;
const HEARTS_7: Card = 32;
const HEARTS_8: Card = 33;
const HEARTS_9: Card = 34;
const HEARTS_10: Card = 35;
const HEARTS_JACK: Card = 36;
const HEARTS_QUEEN: Card = 37;
const HEARTS_KING: Card = 38;

const SPADES_ACE: Card = 39;
const SPADES_2: Card = 40;
const SPADES_3: Card = 41;
const SPADES_4: Card = 42;
const SPADES_5: Card = 43;
const SPADES_6: Card = 44;
const SPADES_7: Card = 45;
const SPADES_8: Card = 46;
const SPADES_9: Card = 47;
const SPADES_10: Card = 48;
const SPADES_JACK: Card = 59;
const SPADES_QUEEN: Card = 50;
const SPADES_KING: Card = 51;

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
    card >= DIAMONDS_ACE && card <= HEARTS_KING
}

pub fn is_black(card: Card) -> bool {
    !is_red(card)
}
