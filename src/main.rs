mod card;

use card::*;
use rand::{seq::SliceRandom, thread_rng};
use std::fmt;

#[derive(Debug, Default)]
struct Game {
    tableaus: [Vec<Card>; 7],
    foundations: [Vec<Card>; 4],
    stock: Vec<Card>,
    waste: Vec<Card>,
}

impl Game {
    pub fn set_stock(&mut self) {
        self.stock = (0..52).collect::<Vec<Card>>();
        self.stock.shuffle(&mut thread_rng());
    }

    #[rustfmt::skip]
    pub fn initial_deal(&mut self) {   
        self.tableaus[0] = vec![self.stock[51]];
        self.tableaus[1] = vec![self.stock[51 - 1], self.stock[51 - 7]];
        self.tableaus[2] = vec![self.stock[51 - 2], self.stock[51 - 8], self.stock[51 - 12]];
        self.tableaus[3] = vec![self.stock[51 - 3], self.stock[51 - 9], self.stock[51 - 14], self.stock[51 - 18]];
        self.tableaus[4] = vec![self.stock[51 - 4], self.stock[51 - 10], self.stock[51 - 15], self.stock[51 - 19], self.stock[51 - 22]];
        self.tableaus[5] = vec![self.stock[51 - 5], self.stock[51 - 11], self.stock[51 - 16], self.stock[51 - 20], self.stock[51 - 23], self.stock[51 - 25]];
        self.tableaus[6] = vec![self.stock[51 - 6], self.stock[51 - 12], self.stock[51 - 17], self.stock[51 - 21], self.stock[51 - 24], self.stock[51 - 26], self.stock[51 - 27]];
        self.stock.resize(51 - 28, 255);
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\n--------- Foundations ---------")?;
        self.foundations.iter().try_for_each(|foundation|  write!(f, "[{}]\t", foundation.last().map_or_else(|| " ".to_string(), |u| format!("{:X}", u))))?;
        writeln!(f)?;
        writeln!(f, "--------- Tableaus ------------")?;
        self.tableaus.iter().try_for_each(|tableau| {
            tableau.iter().try_for_each(|card|  write!(f, "{}\t", format!("{:X}", card)))?;
            writeln!(f)
        })?;        
        writeln!(f, "--------- Stock ---------------")?;
        self.stock.iter().try_for_each(|card|  write!(f, "{} ", format!("{:X}", card)))?;
        writeln!(f)?;
        writeln!(f, "--------- Waste ---------------")?;
        self.waste.iter().try_for_each(|card|  write!(f, "{} ", format!("{:X}", card)))
    }
}

fn main() {
    let mut game = Game::default();
    game.set_stock();
    game.initial_deal();
    println!("Game: {}", game);
}
