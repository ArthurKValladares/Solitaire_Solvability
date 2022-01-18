use crate::{moves::CardPosition, VERBOSE_PRINT};

use super::{card::*, moves::*, Game};
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashSet},
    time::Instant,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GameCompact {
    data: [Card; 52 + 4],
}

pub struct Solver {
    original_game: Game,
    visited_games_states: HashSet<GameCompact>,
    states_to_visit: BinaryHeap<Game>,
    culled_state_count: usize,
    game_overs_reached: usize,
    max_score: usize,
}

impl Game {
    pub fn score(&self) -> usize {
        let foundations_score = self
            .foundations
            .iter()
            .fold(0, |acc, card| acc + card_rank(*card) as usize)
            * 100;

        foundations_score
    }

    pub fn compact_state(&self) -> GameCompact {
        let mut data = [u8::MAX; 52 + 4];

        fn set_highest_bit(bit: &mut u8) {
            *bit = *bit | (1 << 7);
        }

        // First 4 bytes are the foundations
        data[0] = self.foundations[0];
        data[1] = self.foundations[1];
        data[2] = self.foundations[2];
        data[3] = self.foundations[3];
        let mut idx = 4;
        set_highest_bit(&mut data[idx - 1]);

        // Next few bytes are the stock
        let bytes_to_write = self.stock.0.len();
        data[idx..idx + bytes_to_write].copy_from_slice(&self.stock.0);
        idx += bytes_to_write;
        set_highest_bit(&mut data[idx - 1]);

        // Then the waste
        let bytes_to_write = self.waste.0.len();
        data[idx..idx + bytes_to_write].copy_from_slice(&self.waste.0);
        idx += bytes_to_write;
        set_highest_bit(&mut data[idx - 1]);

        // Then the tableaus
        for tableau in &self.tableaus {
            let bytes_to_write = tableau.0.len();
            data[idx..idx + bytes_to_write].copy_from_slice(&tableau.0);
            idx += bytes_to_write;
            set_highest_bit(&mut data[idx - 1]);
        }

        GameCompact { data }
    }
}

impl PartialOrd for Game {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.score().cmp(&other.score()))
    }
}

impl Ord for Game {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score().cmp(&other.score())
    }
}

impl Solver {
    pub fn new() -> Self {
        let original_game = Game::new();
        let mut visited_games_states = HashSet::new();
        visited_games_states.insert(original_game.compact_state());
        let mut states_to_visit = BinaryHeap::new();
        let valid_moves = original_game.valid_moves();
        for valid_move in &valid_moves {
            states_to_visit.push(original_game.handle_move(valid_move));
        }
        Self {
            original_game,
            visited_games_states,
            states_to_visit,
            culled_state_count: 0,
            game_overs_reached: 0,
            max_score: 0,
        }
    }

    pub fn is_game_lost(valid_moves: &HashSet<Move>) -> bool {
        if valid_moves.len() == 1 {
            valid_moves.contains(&Move {
                from: CardPosition::Waste,
                to: CardPosition::Stock,
            })
        } else {
            false
        }
    }

    pub fn log_state(&self, new_state: &Game, print_board: bool) {
        if print_board {
            println!("\nCurrent State:\n{}", new_state);
        }
        println!(
            "States Visited: {}, States to Visit: {}, Culled States: {}, Game Overs: {}",
            self.visited_games_states.len(),
            self.states_to_visit.len(),
            self.culled_state_count,
            self.game_overs_reached
        );
    }

    pub fn is_solvable(&mut self) -> Option<Game> {
        let cutoff_time = 1000.0;
        let timer = Instant::now();
        // TODO: Need to keep track of depth so that we can keep a stack of moves with the solution
        while !self.states_to_visit.is_empty() {
            let new_state = self.states_to_visit.pop().unwrap();
            let new_score = new_state.score();
            if new_score > self.max_score && VERBOSE_PRINT {
                self.max_score = new_score;
                self.log_state(&new_state, true);
            }
            if new_state.is_game_won() {
                self.log_state(&new_state, false);
                return Some(new_state);
            }
            self.visited_games_states.insert(new_state.compact_state());
            let valid_moves = new_state.valid_moves();
            if !Self::is_game_lost(&valid_moves) {
                for valid_move in &valid_moves {
                    let new_state_to_visit = new_state.handle_move(valid_move);
                    if !self
                        .visited_games_states
                        .contains(&new_state_to_visit.compact_state())
                    {
                        self.states_to_visit.push(new_state_to_visit);
                    } else {
                        self.culled_state_count += 1;
                    }
                }
            } else {
                self.game_overs_reached += 1;
            }
            let elapsed_time = timer.elapsed().as_millis() as f64;
            if elapsed_time >= cutoff_time {
                // Hacky return when it takes too long for now
                return None;
            }
        }
        None
    }
}
