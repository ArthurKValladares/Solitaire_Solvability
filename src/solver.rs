use crate::{moves::CardPosition, VERBOSE_PRINT};

use super::{card::*, moves::*, Game};
use std::{collections::HashSet, time::Instant};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GameCompact {
    data: [Card; 52 + 4],
}

pub struct Solver {
    original_game: Game,
    visited_games_states: HashSet<GameCompact>,
    states_to_visit: Vec<(u32, Game)>,
    moves_made: Vec<Move>,
    culled_state_count: usize,
    game_overs_reached: usize,
}

impl Game {
    pub fn compact_state(&self) -> GameCompact {
        let sorted_game = self.sort_tableaus();

        let mut data = [u8::MAX; 52 + 4];

        fn set_highest_bit(bit: &mut u8) {
            *bit = *bit | (1 << 7);
        }

        // First 4 bytes are the foundations
        data[0] = sorted_game.foundations[0];
        data[1] = sorted_game.foundations[1];
        data[2] = sorted_game.foundations[2];
        data[3] = sorted_game.foundations[3];
        let mut idx = 4;
        set_highest_bit(&mut data[idx - 1]);

        // Next few bytes are the stock
        let bytes_to_write = sorted_game.stock.0.len();
        data[idx..idx + bytes_to_write].copy_from_slice(&sorted_game.stock.0);
        idx += bytes_to_write;
        set_highest_bit(&mut data[idx - 1]);

        // Then the waste
        let bytes_to_write = sorted_game.waste.0.len();
        data[idx..idx + bytes_to_write].copy_from_slice(&sorted_game.waste.0);
        idx += bytes_to_write;
        set_highest_bit(&mut data[idx - 1]);

        // Then the tableaus
        for tableau in &sorted_game.tableaus {
            let bytes_to_write = tableau.0.len();
            data[idx..idx + bytes_to_write].copy_from_slice(&tableau.0);
            idx += bytes_to_write;
            set_highest_bit(&mut data[idx - 1]);
        }

        GameCompact { data }
    }
}

impl Solver {
    pub fn new() -> Self {
        let original_game = Game::new();
        let mut visited_games_states = HashSet::new();
        visited_games_states.insert(original_game.compact_state());
        let mut states_to_visit = Vec::new();
        let valid_moves = original_game.valid_moves();
        for valid_move in &valid_moves {
            states_to_visit.push((1, original_game.handle_move(valid_move)));
        }
        Self {
            original_game,
            visited_games_states,
            states_to_visit,
            moves_made: Vec::new(),
            culled_state_count: 0,
            game_overs_reached: 0,
        }
    }

    pub fn game_seed(&self) -> u32 {
        self.original_game.random_seed
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

    pub fn log_state(&self, new_state: &Game) {
        if VERBOSE_PRINT {
            println!("\nOriginal Game:\n{}", self.original_game);
            println!("\nCurrent State:\n{}", new_state);
        }
        println!(
            "States Visited: {}, States to Visit: {}, Culled States: {}, Game Overs: {}",
            self.visited_games_states.len(),
            self.states_to_visit.len(),
            self.culled_state_count,
            self.game_overs_reached,
        );
        if VERBOSE_PRINT {
            println!("Move Made:");
            for mv in &self.moves_made {
                println!("{:?}", mv);
            }
        }
    }

    pub fn is_solvable(&mut self) -> Option<Game> {
        let cutoff_time = 5000.0;
        let timer = Instant::now();
        let mut current_depth = 0;
        while !self.states_to_visit.is_empty() {
            let (new_depth, new_state) = self.states_to_visit.pop().unwrap();
            if new_depth > current_depth {
                self.moves_made.push(new_state.prev_move.unwrap());
            } else {
                let depth_diff = current_depth - new_depth;
                for _ in 0..depth_diff + 1 {
                    self.moves_made.pop();
                }
            }
            current_depth = new_depth;
            if VERBOSE_PRINT {
                self.log_state(&new_state);
            }
            if new_state.is_game_won() {
                self.log_state(&new_state);
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
                        self.states_to_visit
                            .push((current_depth + 1, new_state_to_visit));
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
