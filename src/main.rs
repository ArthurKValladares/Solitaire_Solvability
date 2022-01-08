#![feature(int_abs_diff)]

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
        self.stock.resize(51 - 28, 255);
    }

    fn are_card_ranks_sequential(bottom: Card, top: Card) -> bool {
        card_rank(bottom) == card_rank(top) - 1
    }

    fn are_card_colors_different(card1: Card, card2: Card) -> bool {
        is_red(card1) != is_red(card2)
    }

    fn are_card_suits_the_same(card1: Card, card2: Card) -> bool {
        let card_rank_1 = card_rank(card1);
        let card_rank_2 = card_rank(card2);
        card_rank_1.abs_diff(card_rank_2) <= 12 && !Self::are_card_colors_different(card1, card2)
    }

    fn can_be_placed_on_top_of(bottom: Card, top: Card) -> bool {
        Self::are_card_ranks_sequential(bottom, top) && Self::are_card_colors_different(bottom, top)
    }

    fn can_move_card_to_tableau(&self, card: Card, tableau_idx: usize) -> bool {
        if let Some(top_tableau_card) = self.tableaus[tableau_idx].last() {
            Self::can_be_placed_on_top_of(*top_tableau_card, card)
        } else {
            false
        }
    }

    fn can_move_card_to_foundation(&self, card: Card) -> bool {
        self.foundations.iter().fold(true, |acc, foundation| {
            acc | if let Some(top_foundation_card) = foundation.last() {
                Self::are_card_ranks_sequential(*top_foundation_card, card) && Self::are_card_suits_the_same(*top_foundation_card, card)
            } else {
                card_rank(card) == 0
            }
        })
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
