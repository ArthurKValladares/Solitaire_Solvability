#![feature(int_abs_diff)]

mod card;
mod solver;

use card::*;
use rand::{seq::SliceRandom, thread_rng};
use std::{collections::HashSet, fmt};

#[derive(Debug, Hash, PartialEq, Eq)]
enum CardPosition {
    Stock,
    Waste,
    Foundation(u8),
    // tableau_idx, card_idx
    Tableau((u8, u8))
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct Move {
    from: CardPosition,
    to: CardPosition
}

impl Move {
    fn pretty_string(&self, game: &Game) -> String {
        let from_card = match self.from {
            CardPosition::Stock => game.stock.last().unwrap(),
            CardPosition::Waste => game.waste.last().unwrap(),
            CardPosition::Foundation(idx) => game.foundations[idx as usize].last().unwrap(),
            CardPosition::Tableau((tableau_idx, card_idx)) => &game.tableaus[tableau_idx as usize][card_idx as usize],
        };
        let to_card = match self.to {
            CardPosition::Stock => game.stock.last(),
            CardPosition::Waste => game.waste.last(),
            CardPosition::Foundation(idx) => game.foundations[idx as usize].last(),
            CardPosition::Tableau((tableau_idx, _)) => game.tableaus[tableau_idx as usize].last(),
        };
        format!("From: {:?} - {}\tTo: {:?} - {}", self.from,  pretty_string(*from_card), self.to, to_card.map_or_else(|| " ".to_string(), |card| pretty_string(*card)))
    }
}
#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
struct Game {
    tableaus: [Vec<Card>; 7],
    foundations: [Vec<Card>; 4],
    stock: Vec<Card>,
    waste: Vec<Card>,
}

impl Game {
    fn set_stock(&mut self) {
        self.stock = (0..NUM_CARDS_DECK).collect::<Vec<Card>>();
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

    fn valid_moves(&self) -> HashSet<Move> {
        // TODO: This is inneficient.
        // Ideally, we should be able to determine a subset of possible moves to check according to the last move made,
        // And what previously valid moves have been invalidated by that same previous move

        // ANOTHER TODO: We could parallelize some of this, sorta annoying tho.

        let mut valid_moves = HashSet::new();

        // Valid moves from stock
        if !self.stock.is_empty() {
            valid_moves.insert(Move{
                from: CardPosition::Stock,
                to: CardPosition::Waste
            });
        }

        // Valid moves from waste
        if !self.waste.is_empty() {
            let card = self.waste.last().unwrap();

            if self.can_move_card_to_foundation(*card) {
                valid_moves.insert(Move{
                    from: CardPosition::Waste,
                    to: CardPosition::Foundation(suit_rank(*card))
                });
            }

            self.tableaus.iter().enumerate().for_each(|(tableau_idx, tableau)| {
                if let Some(tableau_card) = tableau.last() {
                    if Self::can_be_placed_on_top_of(*tableau_card, *card) {
                        valid_moves.insert(Move{
                            from: CardPosition::Waste,
                            to: CardPosition::Tableau((tableau_idx as u8, self.tableaus[tableau_idx].len() as u8))
                        });
                    }
                }
            });
        }

        // Valid moves from foundations
        self.foundations.iter().enumerate().for_each(|(foundation_idx, foundation)| {
            if let Some(foundation_card) = foundation.last() {
                self.tableaus.iter().enumerate().for_each(|(tableau_idx, tableau)| {
                    if let Some(tableau_card) = tableau.last() {
                        if Self::can_be_placed_on_top_of(*tableau_card, *foundation_card) {
                            valid_moves.insert(Move{
                                from: CardPosition::Foundation(foundation_idx as u8),
                                to: CardPosition::Tableau((tableau_idx as u8, self.tableaus[tableau_idx].len() as u8))
                            });
                        }
                    }
                });
            }
        });

        // Valid moves from tableau
        for (from_tableau_idx, from_tableau) in self.tableaus.iter().enumerate() {
            // Move from tableau to foundation
            if let Some(from_tableau_card) = from_tableau.last() {
                if self.can_move_card_to_foundation(*from_tableau_card) {
                    valid_moves.insert(Move{
                        from: CardPosition::Tableau((from_tableau_idx as u8, (self.tableaus[from_tableau_idx].len() - 1) as u8)),
                        to: CardPosition::Foundation(suit_rank(*from_tableau_card))
                    });
                }
            }

            // Move from tableau to tableau
            for (card_idx, card) in from_tableau.iter().enumerate().rev() {
                if self.is_card_unlocked(from_tableau_idx, card_idx) {
                    self.tableaus.iter().enumerate().for_each(|(to_tableau_idx, to_tableau)| {
                        if from_tableau_idx != to_tableau_idx {
                            if let Some(to_tableau_card) = to_tableau.last() {
                                if Self::can_be_placed_on_top_of(*to_tableau_card, *card) {
                                    valid_moves.insert(Move{
                                        from: CardPosition::Tableau((from_tableau_idx as u8, card_idx as u8)),
                                        to: CardPosition::Tableau((to_tableau_idx as u8, self.tableaus[to_tableau_idx].len() as u8))
                                    });
                                }
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

    fn is_card_unlocked(&self, tableau_idx: usize, card_idx: usize) -> bool {
        if card_idx == self.tableaus[tableau_idx].len() - 1 {
            true
        } else {
            self.tableaus[tableau_idx][card_idx..].iter().fold((true, None), |(result, prev_card) : (bool, Option<&Card>), card| {
                (result && if let Some(prev_card) = prev_card {
                    Self::can_be_placed_on_top_of(*prev_card, *card)
                } else {
                    true
                }, Some(card))
            }).0
        }
    }

    fn can_draw_from_stock(&self, count: usize) -> bool {
        self.stock.len() >= count
    }

    fn is_game_lost(&self) -> bool {
        self.valid_moves().is_empty()
    }

    //
    // Actions
    //
    fn handle_move(&self, mv: &Move) -> Self {
        let Move {
            from,
            to,
        } = mv;
        match (to, from) {
            (CardPosition::Stock, CardPosition::Waste) => self.draw_from_stock(1),
            _ => unreachable!()
        }
    }

    fn restock(&self) -> Self {
        let mut new_game = self.clone();
        new_game.waste.reverse();
        std::mem::swap(&mut new_game.stock, &mut new_game.waste);
        new_game
    }

    fn draw_from_stock(&self, count: usize) -> Self {
        let mut new_game = self.clone();
        (0..count).for_each(|_| {
            new_game.waste.push(new_game.stock.pop().expect("Popped empty stock"));
        });
        new_game
    }

    fn move_from_waste_to_foundation(&self) -> Self {
        let mut new_game = self.clone();
        let card = new_game.waste.pop().expect("Popped empty waste");
        new_game.foundations[suit_rank(card) as usize].push(card);
        new_game
    }

    fn move_from_tableau_to_foundation(&self, tableau_idx: usize) -> Self {
        let mut new_game = self.clone();
        let card =  new_game.tableaus[tableau_idx].pop().expect("Popped empty tableau");
        new_game.foundations[suit_rank(card) as usize].push(card);
        new_game
    }

    fn move_from_waste_to_tableau(&self, tableau_idx: usize) -> Self {
        let mut new_game = self.clone();
        new_game.tableaus[tableau_idx].push(new_game.waste.pop().expect("Popped empty waste"));
        new_game
    }

    fn move_stack_between_tableaus(&self, from_index: usize, to_index: usize, index_from_bottom: usize) -> Self {
        let mut new_game = self.clone();
        let from_len = new_game.tableaus[from_index].len();
        let start_index = from_len - (1 + index_from_bottom);
        let drain_iter = new_game.tableaus[from_index].drain(start_index..).collect::<Vec<_>>();
        new_game.tableaus[to_index].extend(drain_iter);
        new_game.tableaus[from_index].truncate(start_index);
        new_game
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\n--------- Foundations ---------")?;
        self.foundations.iter().try_for_each(|foundation|  write!(f, "[{}]\t", foundation.last().map_or_else(|| " ".to_string(), |u| format!("{:X}", u))))?;
        writeln!(f)?;
        writeln!(f, "--------- Tableaus ------------")?;
        self.tableaus.iter().enumerate().try_for_each(|(idx, tableau)| {
            write!(f, "{}:\t", idx)?;
            tableau.iter().try_for_each(|card|  write!(f, "{}\t", pretty_string(*card)))?;
            writeln!(f)
        })?;        
        writeln!(f, "--------- Stock ---------------")?;
        self.stock.iter().try_for_each(|card|  write!(f, "{} ", pretty_string(*card)))?;
        writeln!(f)?;
        writeln!(f, "--------- Waste ---------------")?;
        self.waste.iter().try_for_each(|card|  write!(f, "{} ", pretty_string(*card)))?;
        writeln!(f, "--------- Valid Moves ---------")?;
        self.valid_moves().iter().try_for_each(|mv| {
            writeln!(f, "{}", mv.pretty_string(self))
        })    
    }
}

fn main() {
    let mut game = Game::default();
    game.set_stock();
    game.initial_deal();
    println!("Game: {}", game);
}
