mod card;
mod moves;
mod solver;

use arrayvec::ArrayVec;
use card::*;
use mersenne_twister::MT19937;
use moves::*;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use serde::Serialize;
use solver::*;
use std::{cmp::Ordering, fmt};

const VERBOSE_PRINT: bool = false;
const DEBUG: bool = false;

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct CardStack<const CAP: usize>(pub ArrayVec<Card, CAP>);

impl<const CAP: usize> CardStack<CAP> {
    pub fn score(&self) -> u8 {
        *self.0.first().unwrap_or(&u8::MAX)
    }

    pub fn flip_face_up(&mut self) {
        if !self.0.is_empty() {
            *self.0.last_mut().unwrap() = self.0.last().unwrap().face_up();
        }
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

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Game {
    tableaus: [CardStack<20>; 7],
    first_unlocked_idx: [u8; 7],
    foundations: [Card; 4],
    foundation_stack: u64,
    // TODO: Some optimizations in stock and waste
    pub stock: CardStack<52>,
    waste: CardStack<52>,
    prev_move: Option<Move>,
    random_engine: MT19937,
    pub random_seed: u32,
}

impl Game {
    fn new() -> Self {
        let mut game = Game::default();
        game.foundations = [u8::MAX; 4];
        game.set_stock();
        game.initial_deal();
        game.validate();
        game
    }

    fn validate(&self) {
        let mut game_stack: u64 = 0;
        for card in &self.stock.0 {
            game_stack |= 1 << card.face_up();
        }
        for card in &self.waste.0 {
            game_stack |= 1 << card.face_up();
        }
        for tableau in &self.tableaus {
            for card in &tableau.0 {
                game_stack |= 1 << card.face_up();
            }
        }
        let full_game_stack = self.foundation_stack | game_stack;
        if full_game_stack != 0b0000000000001111111111111111111111111111111111111111111111111111 {
            println!("Invalid Game State:\n{}", self);
            let missing_bit = full_game_stack
                ^ 0b0000000000001111111111111111111111111111111111111111111111111111;
            let card = (missing_bit as f64).log2() as u32;
            println!("Missing card: {}", pretty_string(card as u8));
            panic!("Invalid state");
        }
    }

    fn sort_tableaus(&self) -> Self {
        let mut new_game = self.clone();
        // This trick helps reduce the problem space by eliminating symmetrical setups
        new_game.tableaus.sort();
        // we need to reset first unlocked when we sort.
        for idx in 0..7 {
            new_game.set_first_unlocked_index(idx)
        }
        new_game
    }

    fn set_stock(&mut self) {
        let mut rng = rand::thread_rng();
        self.random_seed = rng.gen();
        self.random_engine.reseed(self.random_seed);
        self.stock.0 = (0..NUM_CARDS_DECK).collect::<ArrayVec<Card, 52>>();
        self.random_engine.shuffle(&mut self.stock.0);
    }

    fn set_first_unlocked_index(&mut self, tableau_idx: usize) {
        if self.tableaus[tableau_idx].0.is_empty() {
            self.first_unlocked_idx[tableau_idx] = u8::MAX;
        } else {
            self.first_unlocked_idx[tableau_idx] = 0;
            for (card_idx, _) in self.tableaus[tableau_idx].0.iter().enumerate().rev() {
                if !self.is_card_unlocked(tableau_idx, card_idx) {
                    self.first_unlocked_idx[tableau_idx] = card_idx as u8 + 1;
                    break;
                }
            }
        }
    }

    fn initial_deal(&mut self) {
        self.tableaus[0]
            .0
            .try_extend_from_slice(&[self.stock.0[51]])
            .expect("Could not extend tableau");
        self.tableaus[1]
            .0
            .try_extend_from_slice(&[self.stock.0[51 - 1].face_down(), self.stock.0[51 - 7]])
            .expect("Could not  extend tableau");
        self.tableaus[2]
            .0
            .try_extend_from_slice(&[
                self.stock.0[51 - 2].face_down(),
                self.stock.0[51 - 8].face_down(),
                self.stock.0[51 - 13],
            ])
            .expect("Could not extend tableau");
        self.tableaus[3]
            .0
            .try_extend_from_slice(&[
                self.stock.0[51 - 3].face_down(),
                self.stock.0[51 - 9].face_down(),
                self.stock.0[51 - 14].face_down(),
                self.stock.0[51 - 18],
            ])
            .expect("Could not extend tableau");
        self.tableaus[4]
            .0
            .try_extend_from_slice(&[
                self.stock.0[51 - 4].face_down(),
                self.stock.0[51 - 10].face_down(),
                self.stock.0[51 - 15].face_down(),
                self.stock.0[51 - 19].face_down(),
                self.stock.0[51 - 22],
            ])
            .expect("Could not extend tableau");
        self.tableaus[5]
            .0
            .try_extend_from_slice(&[
                self.stock.0[51 - 5].face_down(),
                self.stock.0[51 - 11].face_down(),
                self.stock.0[51 - 16].face_down(),
                self.stock.0[51 - 20].face_down(),
                self.stock.0[51 - 23].face_down(),
                self.stock.0[51 - 25],
            ])
            .expect("Could not extend tableau");
        self.tableaus[6]
            .0
            .try_extend_from_slice(&[
                self.stock.0[51 - 6].face_down(),
                self.stock.0[51 - 12].face_down(),
                self.stock.0[51 - 17].face_down(),
                self.stock.0[51 - 21].face_down(),
                self.stock.0[51 - 24].face_down(),
                self.stock.0[51 - 26].face_down(),
                self.stock.0[51 - 27],
            ])
            .expect("Could not extend tableau");
        for tableau_idx in 0..7 {
            self.set_first_unlocked_index(tableau_idx);
        }
        self.stock.0.truncate(NUM_CARDS_DECK as usize - 28);
    }

    //
    // Logic Checks
    //

    fn can_be_placed_on_top_of(bottom: Card, top: Card) -> bool {
        are_card_ranks_descending(bottom, top) && are_card_colors_different(bottom, top)
    }

    fn can_move_card_to_foundation(&self, card: Card) -> bool {
        let top_foundation_card = self.foundations[suit_rank(card) as usize];
        if top_foundation_card != u8::MAX {
            are_card_ranks_ascending(top_foundation_card, card)
                && are_card_suits_the_same(top_foundation_card, card)
        } else {
            card_rank(card) == 1
        }
    }

    fn is_card_unlocked(&self, tableau_idx: usize, card_idx: usize) -> bool {
        if card_idx == self.tableaus[tableau_idx].0.len() - 1 {
            true
        } else {
            let card = self.tableaus[tableau_idx].0[card_idx];
            let card_above = self.tableaus[tableau_idx].0[card_idx + 1];
            card.is_face_up() && Self::can_be_placed_on_top_of(card, card_above)
        }
    }

    fn is_game_won(&self) -> bool {
        self.foundations
            .iter()
            .fold(0, |acc, card| acc + card_rank(*card) as usize)
            == ranking_of_kings()
    }

    //
    // Actions
    //
    fn handle_move(&self, mv: &Move) -> Self {
        let Move { from, to } = mv;
        let mut game = match (from, to) {
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
        };
        game.prev_move = Some(*mv);
        if DEBUG {
            game.validate()
        }
        game
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
        new_game.foundations[suit_rank(card) as usize] = card;
        new_game.foundation_stack |= 1 << card;
        new_game
    }

    fn move_from_tableau_to_foundation(&self, tableau_idx: u8) -> Self {
        let mut new_game = self.clone();
        let card = new_game.tableaus[tableau_idx as usize]
            .0
            .pop()
            .expect("Popped empty tableau");
        new_game.tableaus[tableau_idx as usize].flip_face_up();
        new_game.foundations[suit_rank(card) as usize] = card;

        new_game.set_first_unlocked_index(tableau_idx as usize);

        new_game.foundation_stack |= 1 << card;
        new_game
    }

    fn move_from_waste_to_tableau(&self, tableau_idx: u8) -> Self {
        let mut new_game = self.clone();
        new_game.tableaus[tableau_idx as usize]
            .0
            .push(new_game.waste.0.pop().expect("Popped empty waste"));

        if new_game.tableaus[tableau_idx as usize].0.len() == 1 {
            new_game.set_first_unlocked_index(tableau_idx as usize);
        }
        new_game
    }

    fn move_stack_between_tableaus(&self, from_index: u8, card_idx: u8, to_index: u8) -> Self {
        let mut new_game = self.clone();
        let drain_iter = new_game.tableaus[from_index as usize]
            .0
            .drain((card_idx as usize)..)
            .collect::<Vec<_>>();

        // TODO: You are the problem!
        new_game.tableaus[to_index as usize].0.extend(drain_iter);
        new_game.tableaus[from_index as usize]
            .0
            .truncate(card_idx as usize);
        new_game.tableaus[from_index as usize].flip_face_up();

        new_game.set_first_unlocked_index(from_index as usize);
        new_game.set_first_unlocked_index(to_index as usize);

        new_game
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "================== SEED: {:?} ==================",
            self.random_seed
        )?;
        writeln!(f, "--------- Foundations ---------")?;
        self.foundations
            .iter()
            .try_for_each(|card| write!(f, "[{}]\t", pretty_string(*card)))?;
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
        writeln!(f, "--------- Unlocked ------------")?;
        self.first_unlocked_idx
            .iter()
            .try_for_each(|idx| write!(f, "{:?}\t", idx))?;
        writeln!(f)?;
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
            self.valid_moves()
                .iter()
                .try_for_each(|mv| writeln!(f, "{}", mv.pretty_string(self)))?;
        }
        writeln!(f, "--------- Prev Move -----------")?;
        writeln!(f, "{:?}", self.prev_move)
    }
}

#[derive(Serialize)]
struct SolvableGames {
    solvers: Vec<Solver>,
}

fn main() {
    let num_iters = 10000;
    let solvers = (0..num_iters)
        .into_par_iter()
        .map(|_| Solver::new().is_solvable())
        .filter_map(|solver| solver)
        .collect::<Vec<_>>();

    static PRINT_ORIGINAL_STATE: bool = false;
    if PRINT_ORIGINAL_STATE {
        for solver in &solvers {
            solver.log_original_state()
        }
    }

    let stocks = solvers
        .iter()
        .map(|solver| solver.original_stock())
        .collect::<Vec<_>>();
    std::fs::write("compact_solvable_games.txt", format!("{:?}", stocks))
        .expect("could not write json file");

    let solvable_games = SolvableGames { solvers };
    let json_string =
        serde_json::to_string_pretty(&solvable_games).expect("could not create json string");
    std::fs::write("solvable_games.json", &json_string).expect("could not write json file");
}

// TODO: Reduce symmetry in suit permutation
