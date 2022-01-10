#![feature(int_abs_diff)]

mod card;
mod solver;

use arrayvec::ArrayVec;
use card::*;
use rand::{seq::SliceRandom, thread_rng};
use solver::*;
use std::{cmp::Ordering, collections::HashSet, fmt, time::Instant, ops::Range};

#[derive(Debug, Hash, PartialEq, Eq)]
enum CardPosition {
    Stock,
    Waste,
    Foundation(u8),
    // tableau_idx, card_idx
    Tableau((u8, u8)),
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct Move {
    from: CardPosition,
    to: CardPosition,
}

impl Move {
    fn pretty_string(&self, game: &Game) -> String {
        let from_card = match self.from {
            CardPosition::Stock => game.data[game.stock.clone()].last().unwrap(),
            CardPosition::Waste => game.data[game.waste.clone()].last().unwrap(),
            CardPosition::Foundation(idx) => game.foundations[idx as usize].as_ref().unwrap(),
            CardPosition::Tableau((tableau_idx, card_idx)) => &game.data[game.tableaus[tableau_idx as usize].clone()][card_idx as usize],
        };
        let to_card = match self.to {
            CardPosition::Stock => game.data[game.stock.clone()].last(),
            CardPosition::Waste => game.data[game.waste.clone()].last(),
            CardPosition::Foundation(idx) => game.foundations[idx as usize].as_ref(),
            CardPosition::Tableau((tableau_idx, _)) => game.data[game.tableaus[tableau_idx as usize].clone()].last(),
        };
        format!("From: {:?} - {}\tTo: {:?} - {}", self.from,  pretty_string(*from_card), self.to, to_card.map_or_else(|| " ".to_string(), |card| pretty_string(*card)))
    }
}

// TODO: Make sure this is all stack-allocated, and uses as few bytes as possible
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Game {
    data: ArrayVec<Card, 52>,
    // Foundations are not a slice but just the top card
    // I think that makes things a bit easier
    foundations: [Option<u8>; 4],
    tableaus: [Range<usize>; 7],
    stock: Range<usize>,
    waste: Range<usize>,
}

impl Game {
    fn new() -> Self {
        let mut data: ArrayVec<Card, 52> = (0..NUM_CARDS_DECK).collect();
        println!("Initial Deal: {:?}", data);
        data.shuffle(&mut thread_rng());
        Self {
            data,
            tableaus: [(0..0), (0..0), (0..0), (0..0), (0..0), (0..0), (0..0)],
            foundations: [None; 4],
            stock: (0..NUM_CARDS_DECK as usize),
            waste: (0..0),
        }
    }

    fn sort_tableaus(&mut self) {
        // This trick helps reduce the problem space by eliminating symmetrical setups
        // TODO: Figure out how to do that with the slices
    }

    #[rustfmt::skip]
    fn initial_deal(&mut self) {
        // TODO: We will re-arrange the last 28 members of the stock, and mark those as tableaus.
        const CARDS_LEFT_IN_STOCK: usize = 52 - 28;
        let tableaus = 
        [self.data[27],
        self.data[26], self.data[20],
        self.data[25], self.data[19], self.data[14],
        self.data[24], self.data[18], self.data[13], self.data[9],
        self.data[23], self.data[17], self.data[12], self.data[8], self.data[5],
        self.data[22], self.data[16], self.data[11], self.data[7], self.data[4], self.data[2],
        self.data[21], self.data[15], self.data[10], self.data[6], self.data[3], self.data[1], self.data[0]];
        // TODO: Could be optimized to just one memory move (maybe the compiler already does that here)
        for (i, card) in tableaus.into_iter().enumerate() {
            self.data[CARDS_LEFT_IN_STOCK as usize + i] = card;
        }

        self.stock = 0..CARDS_LEFT_IN_STOCK;

        self.tableaus[0] = CARDS_LEFT_IN_STOCK..CARDS_LEFT_IN_STOCK + 1;
        self.tableaus[1] = CARDS_LEFT_IN_STOCK + 1..CARDS_LEFT_IN_STOCK + 3;
        self.tableaus[2] = CARDS_LEFT_IN_STOCK + 3..CARDS_LEFT_IN_STOCK + 6;
        self.tableaus[3] = CARDS_LEFT_IN_STOCK + 6..CARDS_LEFT_IN_STOCK + 10;
        self.tableaus[4] = CARDS_LEFT_IN_STOCK + 10..CARDS_LEFT_IN_STOCK + 15;
        self.tableaus[5] = CARDS_LEFT_IN_STOCK + 15..CARDS_LEFT_IN_STOCK + 21;
        self.tableaus[6] = CARDS_LEFT_IN_STOCK + 21..CARDS_LEFT_IN_STOCK + 28;

        self.sort_tableaus();
    }

    fn valid_moves(&self) -> HashSet<Move> {
        // TODO: This is inneficient.
        // Ideally, we should be able to determine a subset of possible moves to check according to the last move made,
        // And what previously valid moves have been invalidated by that same previous move

        // ANOTHER TODO: We could parallelize some of this, sorta annoying tho.

        let mut valid_moves = HashSet::new();

        // Valid moves from stock
        if !self.data[self.stock.clone()].is_empty() {
            valid_moves.insert(Move{
                from: CardPosition::Stock,
                to: CardPosition::Waste
            });
        } else {
            // Restock
            valid_moves.insert(Move{
                from: CardPosition::Waste,
                to: CardPosition::Stock
            });
        }

        // Valid moves from waste
        if !self.data[self.waste.clone()].is_empty() {
            let card = self.data[self.waste.clone()].last().unwrap();

            if self.can_move_card_to_foundation(*card) {
                valid_moves.insert(Move{
                    from: CardPosition::Waste,
                    to: CardPosition::Foundation(suit_rank(*card))
                });
            }

            self.tableaus.iter().enumerate().for_each(|(tableau_idx, tableau)| {
                if let Some(tableau_card) = self.data[tableau.clone()].last() {
                    if Self::can_be_placed_on_top_of(*tableau_card, *card) {
                        valid_moves.insert(Move{
                            from: CardPosition::Waste,
                            to: CardPosition::Tableau((tableau_idx as u8, self.data[self.tableaus[tableau_idx].clone()].len() as u8))
                        });
                    }
                } else if is_king(*card) {
                    valid_moves.insert(Move{
                        from: CardPosition::Waste,
                        to: CardPosition::Tableau((tableau_idx as u8, self.data[self.tableaus[tableau_idx].clone()].len() as u8))
                    });
                }
            });
        }

        // Valid moves from tableau
        for (from_tableau_idx, from_tableau) in self.tableaus.iter().enumerate() {
            // Move from tableau to foundation
            if let Some(from_tableau_card) = self.data[from_tableau.clone()].last() {
                if self.can_move_card_to_foundation(*from_tableau_card) {
                    valid_moves.insert(Move{
                        from: CardPosition::Tableau((from_tableau_idx as u8, (self.data[self.tableaus[from_tableau_idx].clone()].len() - 1) as u8)),
                        to: CardPosition::Foundation(suit_rank(*from_tableau_card))
                    });
                }
            }

            // Move from tableau to tableau
            for (card_idx, card) in self.data[from_tableau.clone()].iter().enumerate().rev() {
                if self.is_card_unlocked(from_tableau_idx, card_idx) {
                    self.tableaus.iter().enumerate().for_each(|(to_tableau_idx, to_tableau)| {
                        if from_tableau_idx != to_tableau_idx {
                            if let Some(to_tableau_card) = self.data[to_tableau.clone()].last() {
                                if Self::can_be_placed_on_top_of(*to_tableau_card, *card) {
                                    valid_moves.insert(Move{
                                        from: CardPosition::Tableau((from_tableau_idx as u8, card_idx as u8)),
                                        to: CardPosition::Tableau((to_tableau_idx as u8, self.data[self.tableaus[to_tableau_idx].clone()].len() as u8))
                                    });
                                }
                            } else if is_king(*card) {
                                valid_moves.insert(Move{
                                    from: CardPosition::Tableau((from_tableau_idx as u8, card_idx as u8)),
                                    to: CardPosition::Tableau((to_tableau_idx as u8, self.data[self.tableaus[to_tableau_idx].clone()].len() as u8))
                                });
                            }
                        }
                    });
                } else {
                    break;
                }
            }
        };

        valid_moves
    }

    //
    // Logic Checks
    //

    fn can_be_placed_on_top_of(bottom: Card, top: Card) -> bool {
        are_card_ranks_sequential(bottom, top) && are_card_colors_different(bottom, top)
    }

    fn can_move_card_to_tableau(&self, card: Card, tableau_idx: usize) -> bool {
        if let Some(top_tableau_card) = self.data[self.tableaus[tableau_idx].clone()].last() {
            Self::can_be_placed_on_top_of(*top_tableau_card, card)
        } else {
            false
        }
    }

    fn can_move_card_to_foundation(&self, card: Card) -> bool {
        if let Some(top_foundation_card) = self.foundations[suit_rank(card) as usize] {
            are_card_ranks_sequential(top_foundation_card, card) && are_card_suits_the_same(top_foundation_card, card)
        } else {
            card_rank(card) == 0
        }
    }

    fn is_card_unlocked(&self, tableau_idx: usize, card_idx: usize) -> bool {
        if card_idx == self.data[self.tableaus[tableau_idx].clone()].len() - 1 {
            true
        } else {
            self.data[self.tableaus[tableau_idx].clone()][card_idx..].iter().fold((true, None), |(result, prev_card) : (bool, Option<&Card>), card| {
                (result && if let Some(prev_card) = prev_card {
                    Self::can_be_placed_on_top_of(*prev_card, *card)
                } else {
                    true
                }, Some(card))
            }).0
        }
    }

    fn can_draw_from_stock(&self, count: usize) -> bool {
        self.data[self.stock.clone()].len() >= count
    }

    fn is_game_lost(&self) -> bool {
        self.valid_moves().is_empty()
    }

    fn is_game_won(&self) -> bool {
        self.foundations.iter().fold(0, |acc, foundation|  acc + foundation.unwrap_or(0) as usize) == ranking_of_kings()
    }
    //
    // Actions
    //
    fn handle_move(&self, mv: &Move) -> Self {
        let Move { from, to } = mv;
        match (from, to) {
            (CardPosition::Stock, CardPosition::Waste) => self.draw_from_stock(1),
            (CardPosition::Waste, CardPosition::Stock) => self.restock(),
            (CardPosition::Waste, CardPosition::Foundation(_)) => {
                self.move_from_waste_to_foundation()
            }
            (CardPosition::Waste, CardPosition::Tableau((tableau_idx, _))) => {
                self.move_from_waste_to_tableau(*tableau_idx)
            }
            (CardPosition::Tableau((tableau_idx, _)), CardPosition::Foundation(_)) => {
                self.move_from_tableau_to_foundation(*tableau_idx)
            }
            (
                CardPosition::Tableau((from_tableau_idx, card_idx)),
                CardPosition::Tableau((to_tableau_idx, _)),
            ) => self.move_stack_between_tableaus(*from_tableau_idx, *card_idx, *to_tableau_idx),
            _ => panic!("{:?}", mv),
        }
    }

    fn restock(&self) -> Self {
        let mut new_game = self.clone();
        /*
        new_game.data[new_game.waste.clone()].reverse();
        std::mem::swap(&mut new_game.stock, &mut new_game.waste);
        */
        new_game
    }

    fn draw_from_stock(&self, count: usize) -> Self {
        let mut new_game = self.clone();
        /*
        (0..count).for_each(|_| {
            new_game.waste.0.push(new_game.stock.0.pop().expect("Popped empty stock"));
        });
        */
        new_game
    }

    fn move_from_waste_to_foundation(&self) -> Self {
        let mut new_game = self.clone();
        /*
        let card = new_game.waste.0.pop().expect("Popped empty waste");
        new_game.foundations[suit_rank(card) as usize] = Some(card);
        */
        new_game
    }

    fn move_from_tableau_to_foundation(&self, tableau_idx: u8) -> Self {
        let mut new_game = self.clone();
        /*
        let card =  new_game.tableaus[tableau_idx as usize].0.pop().expect("Popped empty tableau");
        new_game.foundations[suit_rank(card) as usize] = Some(card);
        // If we the tableau is now empty, we need to sort the tableaus
        if  new_game.tableaus[tableau_idx as usize].0.is_empty() {
            new_game.sort_tableaus();
        }
        */
        new_game
    }

    fn move_from_waste_to_tableau(&self, tableau_idx: u8) -> Self {
        let mut new_game = self.clone();
        /*
        new_game.tableaus[tableau_idx as usize].0.push(new_game.waste.0.pop().expect("Popped empty waste"));
        // If we the tableau now only has 1 card, we need to sort the tableaus
        if  new_game.tableaus[tableau_idx as usize].0.len() == 1 {
            new_game.sort_tableaus();
        }
        */
        new_game
    }

    fn move_stack_between_tableaus(&self, from_index: u8, card_idx: u8, to_index: u8) -> Self {
        let mut new_game = self.clone();
        /*
        let drain_iter = new_game.tableaus[from_index as usize].0.drain((card_idx as usize)..).collect::<Vec<_>>();
        let to_prev_len =  new_game.tableaus[to_index as usize].0.len();
        new_game.tableaus[to_index as usize].0.extend(drain_iter);
        new_game.tableaus[from_index as usize].0.truncate(card_idx as usize);
        // If we from tableau is empty or the to tableau was empty, we need to sort the tableaus
        if  new_game.tableaus[from_index as usize].0.is_empty() || to_prev_len == 0 {
            new_game.sort_tableaus();
        }
        */
        new_game
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\n--------- Foundations ---------")?;
        self.foundations.iter().try_for_each(|foundation|  write!(f, "[{}]\t", foundation.map_or_else(|| " ".to_string(), pretty_string)))?;
        writeln!(f)?;
        writeln!(f, "--------- Tableaus ------------")?;
        self.tableaus.iter().enumerate().try_for_each(|(idx, range)| {
            write!(f, "{}:\t", idx)?;
            self.data[range.clone()].iter().try_for_each(|card|  write!(f, "{}\t", pretty_string(*card)))?;
            writeln!(f)
        })?;
        writeln!(f, "--------- Stock ---------------")?;
        self.data[self.stock.clone()].iter().try_for_each(|card|  write!(f, "{} ", pretty_string(*card)))?;
        writeln!(f)?;
        writeln!(f, "--------- Waste ---------------")?;
        self.data[self.waste.clone()].iter().try_for_each(|card|  write!(f, "{} ", pretty_string(*card)))?;
        writeln!(f)?;
        writeln!(f, "--------- Valid Moves ---------")?;
        self.valid_moves().iter().try_for_each(|mv| {
            writeln!(f, "{}", mv.pretty_string(self))
        })
    }
}

fn main() {
    let mut game = Game::new();
    game.initial_deal();
    println!("{}", game);
    /*
    let mut solver = Solver::new();
    let timer = Instant::now();
    println!("Game: {:?}", solver.is_solvable());
    println!("Elapsed Time: {}", timer.elapsed().as_millis() as f64);
    */
}
