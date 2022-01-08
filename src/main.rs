#![feature(int_abs_diff)]

mod card;

use card::*;
use rand::{seq::SliceRandom, thread_rng};
use std::fmt;
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Game {
    tableaus: [Vec<Card>; 7],
    foundations: [Vec<Card>; 4],
    stock: Vec<Card>,
    waste: Vec<Card>,
}

impl Game {
    fn set_stock(&mut self) {
        self.stock = (0..52).collect::<Vec<Card>>();
        self.stock.shuffle(&mut thread_rng());
    }

    #[rustfmt::skip]
    fn initial_deal(&mut self) {   
        self.tableaus[0] = vec![self.stock[51]];
        self.tableaus[1] = vec![self.stock[51 - 1], self.stock[51 - 7]];
        self.tableaus[2] = vec![self.stock[51 - 2], self.stock[51 - 8], self.stock[51 - 12]];
        self.tableaus[3] = vec![self.stock[51 - 3], self.stock[51 - 9], self.stock[51 - 14], self.stock[51 - 18]];
        self.tableaus[4] = vec![self.stock[51 - 4], self.stock[51 - 10], self.stock[51 - 15], self.stock[51 - 19], self.stock[51 - 22]];
        self.tableaus[5] = vec![self.stock[51 - 5], self.stock[51 - 11], self.stock[51 - 16], self.stock[51 - 20], self.stock[51 - 23], self.stock[51 - 25]];
        self.tableaus[6] = vec![self.stock[51 - 6], self.stock[51 - 12], self.stock[51 - 17], self.stock[51 - 21], self.stock[51 - 24], self.stock[51 - 26], self.stock[51 - 27]];
        self.stock.truncate(51 - 28);
    }

    //
    // Logic Checks
    //

    fn can_be_placed_on_top_of(bottom: Card, top: Card) -> bool {
        are_card_ranks_sequential(bottom, top) && are_card_colors_different(bottom, top)
    }

    fn can_move_card_to_tableau(&self, card: Card, tableau_idx: usize) -> bool {
        if let Some(top_tableau_card) = self.tableaus[tableau_idx].last() {
            Self::can_be_placed_on_top_of(*top_tableau_card, card)
        } else {
            false
        }
    }

    fn can_move_card_to_foundation(&self, card: Card) -> bool {
        if let Some(top_foundation_card) = self.foundations[suit_rank(card) as usize].last() {
            are_card_ranks_sequential(*top_foundation_card, card) && are_card_suits_the_same(*top_foundation_card, card)
        } else {
            card_rank(card) == 0
        }
    }

    fn is_card_unlocked(&self, tableau_idx: usize, stack_idx: usize) -> bool {
        if stack_idx == self.tableaus[tableau_idx].len() - 1 {
            true
        } else {
            self.tableaus[tableau_idx][tableau_idx..].iter().fold((true, None), |(result, prev_card) : (bool, Option<&Card>), card| {
                (result | if let Some(prev_card) = prev_card {
                    Self::can_be_placed_on_top_of(*prev_card, *card)
                } else {
                    true
                }, Some(card))
            }).0
        }
    }

    fn is_game_won(&self) -> bool {
        self.foundations.iter().fold(0, |acc, foundation| acc + foundation.len()) == 52
    }

    fn can_draw_from_stock(&self, count: usize) -> bool {
        self.stock.len() >= count
    }

    //
    // Actions
    //

    fn restock(&mut self) {
        self.waste.reverse();
        std::mem::swap(&mut self.stock, &mut self.waste);
    }

    fn draw_from_stock(&mut self, count: usize) {
        (0..count).for_each(|_| {
            self.waste.push(self.stock.pop().expect("Popped empty stock"));
        })
    }

    fn move_from_waste_to_foundation(&mut self) {
        let card = self.waste.pop().expect("Popped empty waste");
        self.foundations[suit_rank(card) as usize].push(card);
    }

    fn move_from_tableau_to_foundation(&mut self, tableau_idx: usize) {
        let card =  self.tableaus[tableau_idx].pop().expect("Popped empty tableau");
        self.foundations[suit_rank(card) as usize].push(card);
    }

    fn move_from_waste_to_tableau(&mut self, tableau_idx: usize) {
        self.tableaus[tableau_idx].push(self.waste.pop().expect("Popped empty waste"))
    }

    fn move_stack_between_tableaus(&mut self, from_index: usize, to_index: usize, index_from_bottom: usize) {
        let from_len = self.tableaus[from_index].len();
        let start_index = from_len - (1 + index_from_bottom);
        let drain_iter = self.tableaus[from_index].drain(start_index..).collect::<Vec<_>>();
        self.tableaus[to_index].extend(drain_iter);
        self.tableaus[from_index].truncate(start_index);
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\n--------- Foundations ---------")?;
        self.foundations.iter().try_for_each(|foundation|  write!(f, "[{}]\t", foundation.last().map_or_else(|| " ".to_string(), |u| format!("{:X}", u))))?;
        writeln!(f)?;
        writeln!(f, "--------- Tableaus ------------")?;
        self.tableaus.iter().try_for_each(|tableau| {
            tableau.iter().try_for_each(|card|  write!(f, "{:X}\t", card))?;
            writeln!(f)
        })?;        
        writeln!(f, "--------- Stock ---------------")?;
        self.stock.iter().try_for_each(|card|  write!(f, "{:X} ", card))?;
        writeln!(f)?;
        writeln!(f, "--------- Waste ---------------")?;
        self.waste.iter().try_for_each(|card|  write!(f, "{:X} ", card))
    }
}

fn main() {
    let mut game = Game::default();
    game.set_stock();
    game.initial_deal();
    println!("Game: {}", game);
}
