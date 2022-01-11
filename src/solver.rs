use super::{card::*, Game};
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashSet},
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
}

impl Game {
    pub fn score(&self) -> usize {
        self.foundations
            .iter()
            .fold(0, |acc, card| acc + card_rank(*card) as usize)
    }

    pub fn compact_state(&self) -> GameCompact {
        let mut data = [u8::MAX; 52 + 4];

        fn set_hihest_bit(bit: &mut u8) {
            *bit = *bit | (1 << 7);
        }

        // First 4 bytes are the foundations
        data[0] = self.foundations[0];
        data[1] = self.foundations[1];
        data[2] = self.foundations[2];
        data[3] = self.foundations[3];
        let mut idx = 4;
        set_hihest_bit(&mut data[idx - 1]);

        // Next few bytes are the stock
        let bytes_to_write = self.stock.0.len();
        data[idx..idx + bytes_to_write].copy_from_slice(&self.stock.0);
        idx += bytes_to_write;
        set_hihest_bit(&mut data[idx - 1]);

        // Then the waste
        let bytes_to_write = self.waste.0.len();
        data[idx..idx + bytes_to_write].copy_from_slice(&self.waste.0);
        idx += bytes_to_write;
        set_hihest_bit(&mut data[idx - 1]);

        // Then the tableaus
        for tableau in &self.tableaus {
            let bytes_to_write = tableau.0.len();
            data[idx..idx + bytes_to_write].copy_from_slice(&tableau.0);
            idx += bytes_to_write;
            set_hihest_bit(&mut data[idx - 1]);
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
        }
    }

    pub fn is_solvable(&mut self) -> Option<Game> {
        println!("Original Game:\n{}", self.original_game);
        // TODO: Need to keep track of depth so that we can keep a stack of moves with the solution
        let mut iter = 0;
        while !self.states_to_visit.is_empty() {
            iter += 1;
            let new_state = self.states_to_visit.pop().unwrap();
            if iter % 1000000 == 0 {
                println!("\nCurrent State:\n{}", new_state);
                println!(
                    "States Visited: {}, States to Visit: {}, Culled States: {}",
                    self.visited_games_states.len(),
                    self.states_to_visit.len(),
                    self.culled_state_count
                );
            }
            if new_state.is_game_won() {
                return Some(new_state);
            }
            self.visited_games_states.insert(new_state.compact_state());
            let valid_moves = new_state.valid_moves();
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
        }
        None
    }
}
