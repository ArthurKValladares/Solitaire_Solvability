#![feature(int_abs_diff)]

mod card;
mod solver;

use arrayvec::ArrayVec;
use card::*;
use rand::{seq::SliceRandom, thread_rng};
use solver::*;
use std::{cmp::Ordering, collections::HashSet, fmt, time::Instant};

#[derive(Debug, Hash, PartialEq, Eq)]
enum CardPosition {
    Stock,
    Waste,
    Foundation(u8),
    // tableau_idx, card_idx
    Tableau((u8, u8)),
}

const VERBOSE_PRINT: bool = false;

#[derive(Debug, Hash, PartialEq, Eq)]
struct Move {
    from: CardPosition,
    to: CardPosition,
}

impl Move {
    fn pretty_string(&self, game: &Game) -> String {
        let from_card = match self.from {
            CardPosition::Stock => game.stock.0.last().unwrap(),
            CardPosition::Waste => game.waste.0.last().unwrap(),
            CardPosition::Foundation(idx) => game.foundations[idx as usize].as_ref().unwrap(),
            CardPosition::Tableau((tableau_idx, card_idx)) => {
                &game.tableaus[tableau_idx as usize].0[card_idx as usize]
            }
        };
        let to_card = match self.to {
            CardPosition::Stock => game.stock.0.last(),
            CardPosition::Waste => game.waste.0.last(),
            CardPosition::Foundation(idx) => game.foundations[idx as usize].as_ref(),
            CardPosition::Tableau((tableau_idx, _)) => game.tableaus[tableau_idx as usize].0.last(),
        };
        format!(
            "From: {:?} - {}\tTo: {:?} - {}",
            self.from,
            pretty_string(*from_card),
            self.to,
            to_card.map_or_else(|| " ".to_string(), |card| pretty_string(*card))
        )
    }
}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct CardStack<const CAP: usize>(ArrayVec<Card, CAP>);

impl<const CAP: usize> CardStack<CAP> {
    pub fn score(&self) -> u8 {
        *self.0.first().unwrap_or(&u8::MAX)
    }
}

impl<const CAP: usize> PartialOrd for CardStack<CAP> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.score().cmp(&other.score()))
    }
}

impl<const CAP: usize> Ord for CardStack<CAP> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score().cmp(&other.score())
    }
}

// TODO: Make sure this is all stack-allocated, and uses as few bytes as possible
#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Game {
    // TODO: We can optimize this further
    // stock and waste will always add up to 52 at most, so they can share an array
    tableaus: [CardStack<20>; 7],
    foundations: [Option<Card>; 4],
    stock: CardStack<52>,
    waste: CardStack<52>,
}

impl Game {
    fn new() -> Self {
        let mut game = Game::default();
        game.set_stock();
        game.initial_deal();
        game
    }

    fn sort_tableaus(&mut self) {
        // This trick helps reduce the problem space by eliminating symmetrical setups
        self.tableaus.sort()
    }

    fn set_stock(&mut self) {
        self.stock.0 = (0..NUM_CARDS_DECK).collect::<ArrayVec<Card, 52>>();
        self.stock.0.shuffle(&mut thread_rng());
    }

    fn initial_deal(&mut self) {
        self.tableaus[0]
            .0
            .try_extend_from_slice(&[self.stock.0[51]])
            .expect("Could extend tableau");
        self.tableaus[1]
            .0
            .try_extend_from_slice(&[self.stock.0[51 - 1], self.stock.0[51 - 7]])
            .expect("Could extend tableau");
        self.tableaus[2]
            .0
            .try_extend_from_slice(&[
                self.stock.0[51 - 2],
                self.stock.0[51 - 8],
                self.stock.0[51 - 12],
            ])
            .expect("Could extend tableau");
        self.tableaus[3]
            .0
            .try_extend_from_slice(&[
                self.stock.0[51 - 3],
                self.stock.0[51 - 9],
                self.stock.0[51 - 14],
                self.stock.0[51 - 18],
            ])
            .expect("Could extend tableau");
        self.tableaus[4]
            .0
            .try_extend_from_slice(&[
                self.stock.0[51 - 4],
                self.stock.0[51 - 10],
                self.stock.0[51 - 15],
                self.stock.0[51 - 19],
                self.stock.0[51 - 22],
            ])
            .expect("Could extend tableau");
        self.tableaus[5]
            .0
            .try_extend_from_slice(&[
                self.stock.0[51 - 5],
                self.stock.0[51 - 11],
                self.stock.0[51 - 16],
                self.stock.0[51 - 20],
                self.stock.0[51 - 23],
                self.stock.0[51 - 25],
            ])
            .expect("Could extend tableau");
        self.tableaus[6]
            .0
            .try_extend_from_slice(&[
                self.stock.0[51 - 6],
                self.stock.0[51 - 12],
                self.stock.0[51 - 17],
                self.stock.0[51 - 21],
                self.stock.0[51 - 24],
                self.stock.0[51 - 26],
                self.stock.0[51 - 27],
            ])
            .expect("Could extend tableau");
        self.stock.0.truncate(NUM_CARDS_DECK as usize - 28);
        self.sort_tableaus();
    }

    fn get_move_from_stock(&self) -> Move {
        if !self.stock.0.is_empty() {
            Move {
                from: CardPosition::Stock,
                to: CardPosition::Waste,
            }
        } else {
            // Restock
            Move {
                from: CardPosition::Waste,
                to: CardPosition::Stock,
            }
        }
    }

    fn get_move_from_waste_to_tableau(&self, tableau_idx: usize) -> Option<Move> {
        let card = self.waste.0.last().unwrap();
        if let Some(tableau_card) = self.tableaus[tableau_idx].0.last() {
            if Self::can_be_placed_on_top_of(*tableau_card, *card) {
                Some(Move {
                    from: CardPosition::Waste,
                    to: CardPosition::Tableau((
                        tableau_idx as u8,
                        self.tableaus[tableau_idx].0.len() as u8,
                    )),
                })
            } else {
                None
            }
        } else if is_king(*card) {
            Some(Move {
                from: CardPosition::Waste,
                to: CardPosition::Tableau((
                    tableau_idx as u8,
                    self.tableaus[tableau_idx].0.len() as u8,
                )),
            })
        } else {
            None
        }
    }

    fn get_moves_from_waste(&self) -> HashSet<Move> {
        let mut set = HashSet::new();
        if !self.waste.0.is_empty() {
            let card = self.waste.0.last().unwrap();

            if self.can_move_card_to_foundation(*card) {
                set.insert(Move {
                    from: CardPosition::Waste,
                    to: CardPosition::Foundation(suit_rank(*card)),
                });
            }

            self.tableaus
                .iter()
                .enumerate()
                .for_each(|(tableau_idx, _)| {
                    if let Some(mv) = self.get_move_from_waste_to_tableau(tableau_idx) {
                        set.insert(mv);
                    }
                });
        };
        set
    }

    fn get_specific_move_between_tableaus(
        &self,
        from_tableau_idx: usize,
        card_idx: usize,
        to_tableau_idx: usize,
    ) -> Option<Move> {
        let card = self.tableaus[from_tableau_idx].0[card_idx];
        if let Some(to_tableau_card) = self.tableaus[to_tableau_idx].0.last() {
            if Self::can_be_placed_on_top_of(*to_tableau_card, card) {
                Some(Move {
                    from: CardPosition::Tableau((from_tableau_idx as u8, card_idx as u8)),
                    to: CardPosition::Tableau((
                        to_tableau_idx as u8,
                        self.tableaus[to_tableau_idx].0.len() as u8,
                    )),
                })
            } else {
                None
            }
        } else if is_king(card) {
            Some(Move {
                from: CardPosition::Tableau((from_tableau_idx as u8, card_idx as u8)),
                to: CardPosition::Tableau((
                    to_tableau_idx as u8,
                    self.tableaus[to_tableau_idx].0.len() as u8,
                )),
            })
        } else {
            None
        }
    }

    fn get_tableau_moves_from_tableau(&self, from_tableau_idx: usize) -> HashSet<Move> {
        let mut set = HashSet::new();
        // TODO: Cache cards that are unlocked? YES, DO THAT
        for (card_idx, _) in self.tableaus[from_tableau_idx].0.iter().enumerate().rev() {
            if self.is_card_unlocked(from_tableau_idx, card_idx) {
                self.tableaus
                    .iter()
                    .enumerate()
                    .for_each(|(to_tableau_idx, to_tableau)| {
                        if from_tableau_idx != to_tableau_idx {
                            if let Some(mv) = self.get_specific_move_between_tableaus(
                                from_tableau_idx,
                                card_idx,
                                to_tableau_idx,
                            ) {
                                set.insert(mv);
                            }
                        }
                    });
            } else {
                break;
            }
        }
        set
    }

    fn get_tableau_moves_to_tableau(&self, to_tableau_idx: usize) -> HashSet<Move> {
        let mut set = HashSet::new();
        self.tableaus
            .iter()
            .enumerate()
            .for_each(|(from_tableau_idx, from_tableau)| {
                if from_tableau_idx != to_tableau_idx {
                    for (card_idx, card) in
                        self.tableaus[from_tableau_idx].0.iter().enumerate().rev()
                    {
                        if self.is_card_unlocked(from_tableau_idx, card_idx) {
                            if let Some(mv) = self.get_specific_move_between_tableaus(
                                from_tableau_idx,
                                card_idx,
                                to_tableau_idx,
                            ) {
                                set.insert(mv);
                            }
                        }
                    }
                }
            });
        set
    }

    fn get_move_from_tableau_to_foundation(&self, from_tableau_idx: usize) -> Option<Move> {
        if let Some(from_tableau_card) = self.tableaus[from_tableau_idx].0.last() {
            if self.can_move_card_to_foundation(*from_tableau_card) {
                Some(Move {
                    from: CardPosition::Tableau((
                        from_tableau_idx as u8,
                        (self.tableaus[from_tableau_idx].0.len() - 1) as u8,
                    )),
                    to: CardPosition::Foundation(suit_rank(*from_tableau_card)),
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_moves_from_tableau(&self) -> HashSet<Move> {
        let mut set = HashSet::new();
        for (from_tableau_idx, from_tableau) in self.tableaus.iter().enumerate() {
            if let Some(mv) = self.get_move_from_tableau_to_foundation(from_tableau_idx) {
                set.insert(mv);
            }
            set.extend(self.get_tableau_moves_from_tableau(from_tableau_idx));
        }
        set
    }

    fn valid_moves(&self, last_move: Option<Move>) -> HashSet<Move> {
        // ANOTHER TODO: We could parallelize some of this, sorta annoying tho.
        let mut valid_moves = HashSet::new();
        valid_moves.insert(self.get_move_from_stock());
        if let Some(prev_move) = last_move {
            let Move { from, to } = prev_move;
            match (from, to) {
                (CardPosition::Stock, CardPosition::Waste) => {
                    valid_moves.extend(self.get_moves_from_waste());
                }
                (CardPosition::Waste, CardPosition::Stock) => {}
                (CardPosition::Waste, CardPosition::Foundation(_)) => {
                    valid_moves.extend(self.get_moves_from_waste());
                }
                (CardPosition::Waste, CardPosition::Tableau((tableau_idx, _))) => {
                    valid_moves.extend(self.get_moves_from_waste());
                }
                (CardPosition::Tableau((tableau_idx, _)), CardPosition::Foundation(_)) => {
                    if let Some(mv) = self.get_move_from_waste_to_tableau(tableau_idx as usize) {
                        valid_moves.insert(mv);
                    }
                    valid_moves.extend(self.get_tableau_moves_from_tableau(tableau_idx as usize));
                    valid_moves.extend(self.get_tableau_moves_to_tableau(tableau_idx as usize));
                }
                (
                    CardPosition::Tableau((from_tableau_idx, card_idx)),
                    CardPosition::Tableau((to_tableau_idx, _)),
                ) => {
                    if let Some(mv) = self.get_move_from_waste_to_tableau(from_tableau_idx as usize)
                    {
                        valid_moves.insert(mv);
                    }
                    if let Some(mv) = self.get_move_from_waste_to_tableau(to_tableau_idx as usize) {
                        valid_moves.insert(mv);
                    }

                    valid_moves
                        .extend(self.get_tableau_moves_from_tableau(from_tableau_idx as usize));
                    valid_moves
                        .extend(self.get_tableau_moves_to_tableau(from_tableau_idx as usize));

                    valid_moves
                        .extend(self.get_tableau_moves_from_tableau(to_tableau_idx as usize));
                    valid_moves.extend(self.get_tableau_moves_to_tableau(to_tableau_idx as usize));
                }
                _ => unreachable!(),
            }
        } else {
            valid_moves.extend(self.get_moves_from_waste());
            valid_moves.extend(self.get_moves_from_tableau());
        }
        valid_moves
    }

    //
    // Logic Checks
    //

    fn can_be_placed_on_top_of(bottom: Card, top: Card) -> bool {
        are_card_ranks_sequential(bottom, top) && are_card_colors_different(bottom, top)
    }

    fn can_move_card_to_tableau(&self, card: Card, tableau_idx: usize) -> bool {
        if let Some(top_tableau_card) = self.tableaus[tableau_idx].0.last() {
            Self::can_be_placed_on_top_of(*top_tableau_card, card)
        } else {
            false
        }
    }

    fn can_move_card_to_foundation(&self, card: Card) -> bool {
        if let Some(top_foundation_card) = self.foundations[suit_rank(card) as usize] {
            are_card_ranks_sequential(top_foundation_card, card)
                && are_card_suits_the_same(top_foundation_card, card)
        } else {
            card_rank(card) == 0
        }
    }

    fn is_card_unlocked(&self, tableau_idx: usize, card_idx: usize) -> bool {
        if card_idx == self.tableaus[tableau_idx].0.len() - 1 {
            true
        } else {
            self.tableaus[tableau_idx].0[card_idx..]
                .iter()
                .fold(
                    (true, None),
                    |(result, prev_card): (bool, Option<&Card>), card| {
                        (
                            result
                                && if let Some(prev_card) = prev_card {
                                    Self::can_be_placed_on_top_of(*prev_card, *card)
                                } else {
                                    true
                                },
                            Some(card),
                        )
                    },
                )
                .0
        }
    }

    fn can_draw_from_stock(&self, count: usize) -> bool {
        self.stock.0.len() >= count
    }

    fn is_game_won(&self) -> bool {
        self.foundations
            .iter()
            .fold(0, |acc, foundation| acc + foundation.unwrap_or(0) as usize)
            == ranking_of_kings()
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
            _ => unreachable!(),
        }
    }

    fn restock(&self) -> Self {
        let mut new_game = self.clone();
        new_game.waste.0.reverse();
        std::mem::swap(&mut new_game.stock, &mut new_game.waste);
        new_game
    }

    fn draw_from_stock(&self, count: usize) -> Self {
        let mut new_game = self.clone();
        (0..count).for_each(|_| {
            new_game
                .waste
                .0
                .push(new_game.stock.0.pop().expect("Popped empty stock"));
        });
        new_game
    }

    fn move_from_waste_to_foundation(&self) -> Self {
        let mut new_game = self.clone();
        let card = new_game.waste.0.pop().expect("Popped empty waste");
        new_game.foundations[suit_rank(card) as usize] = Some(card);
        new_game
    }

    fn move_from_tableau_to_foundation(&self, tableau_idx: u8) -> Self {
        let mut new_game = self.clone();
        let card = new_game.tableaus[tableau_idx as usize]
            .0
            .pop()
            .expect("Popped empty tableau");
        new_game.foundations[suit_rank(card) as usize] = Some(card);
        // If we the tableau is now empty, we need to sort the tableaus
        if new_game.tableaus[tableau_idx as usize].0.is_empty() {
            new_game.sort_tableaus();
        }
        new_game
    }

    fn move_from_waste_to_tableau(&self, tableau_idx: u8) -> Self {
        let mut new_game = self.clone();
        new_game.tableaus[tableau_idx as usize]
            .0
            .push(new_game.waste.0.pop().expect("Popped empty waste"));
        // If we the tableau now only has 1 card, we need to sort the tableaus
        if new_game.tableaus[tableau_idx as usize].0.len() == 1 {
            new_game.sort_tableaus();
        }
        new_game
    }

    fn move_stack_between_tableaus(&self, from_index: u8, card_idx: u8, to_index: u8) -> Self {
        let mut new_game = self.clone();
        let drain_iter = new_game.tableaus[from_index as usize]
            .0
            .drain((card_idx as usize)..)
            .collect::<Vec<_>>();
        let to_prev_len = new_game.tableaus[to_index as usize].0.len();
        new_game.tableaus[to_index as usize].0.extend(drain_iter);
        new_game.tableaus[from_index as usize]
            .0
            .truncate(card_idx as usize);
        // If we from tableau is empty or the to tableau was empty, we need to sort the tableaus
        if new_game.tableaus[from_index as usize].0.is_empty() || to_prev_len == 0 {
            new_game.sort_tableaus();
        }
        new_game
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\n--------- Foundations ---------")?;
        self.foundations.iter().try_for_each(|foundation| {
            write!(
                f,
                "[{}]\t",
                foundation.map_or_else(|| " ".to_string(), |card| pretty_string(card))
            )
        })?;
        writeln!(f)?;
        writeln!(f, "--------- Tableaus ------------")?;
        self.tableaus
            .iter()
            .enumerate()
            .try_for_each(|(idx, tableau)| {
                write!(f, "{}:\t", idx)?;
                tableau
                    .0
                    .iter()
                    .try_for_each(|card| write!(f, "{}\t", pretty_string(*card)))?;
                writeln!(f)
            })?;
        writeln!(f, "--------- Stock ---------------")?;
        self.stock
            .0
            .iter()
            .try_for_each(|card| write!(f, "{} ", pretty_string(*card)))?;
        writeln!(f)?;
        writeln!(f, "--------- Waste ---------------")?;
        self.waste
            .0
            .iter()
            .try_for_each(|card| write!(f, "{} ", pretty_string(*card)))?;
        writeln!(f)?;
        if VERBOSE_PRINT {
            writeln!(f, "--------- Valid Moves ---------")?;
            self.valid_moves(None)
                .iter()
                .try_for_each(|mv| writeln!(f, "{}", mv.pretty_string(self)))?;
        }
        writeln!(f)
    }
}

fn main() {
    let mut solver = Solver::new();
    let timer = Instant::now();
    println!("Game: {:?}", solver.is_solvable());
    println!("Elapsed Time: {}", timer.elapsed().as_millis() as f64);
}
