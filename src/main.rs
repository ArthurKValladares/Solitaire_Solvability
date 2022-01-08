enum Suit {
    Club = 0,
    Spade = 1,
    Heart = 2,
    Diamond = 3,
}

enum Rank {
    Ace,
    Jack,
    Queen,
    Kind,
    Number(u8),
}

struct Card {
    suit: Suit,
    rank: Rank,
}

pub struct Tableau {
    piles: [Vec<Card>; 7],
}

pub struct Foundations {
    piles: [Vec<Card>; 4],
}

pub struct Stock {
    stock: Vec<Card>,
}

pub struct Waste {
    waste: Vec<Card>,
}

struct Game {
    tableau: Tableau,
    foundations: Foundations,
    stock: Stock,
    waste: Waste,
}

fn main() {
    println!("Hello, world!");
}
