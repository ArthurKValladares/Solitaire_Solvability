use super::Game;
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashSet},
};

pub struct Solver {
    original_game: Game,
    visited_games_states: HashSet<Game>,
    states_to_visit: BinaryHeap<Game>,
}

impl Game {
    pub fn score(&self) -> usize {
        /*
        self.foundations
            .iter()
            .fold(0, |acc, foundation| acc + foundation.unwrap_or(0) as usize)
            */
        0
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
        visited_games_states.insert(original_game.clone());
        let mut states_to_visit = BinaryHeap::new();
        let valid_moves = original_game.valid_moves();
        for valid_move in &valid_moves {
            states_to_visit.push(original_game.handle_move(valid_move));
        }
        Self {
            original_game,
            visited_games_states,
            states_to_visit,
        }
    }

    pub fn is_solvable(&mut self) -> Option<Game> {
        println!("Original Game:\n{}", self.original_game);
        // TODO: Need to keep track of depth so that we can keep a stack of moves with the solution
        let mut iter = 0;
        while !self.states_to_visit.is_empty() {
            iter += 1;
            let new_state = self.states_to_visit.pop().unwrap();
            if iter % 100000 == 0 {
                println!("\nCurrent State:\n{}", new_state);
            }
            if new_state.is_game_won() {
                return Some(new_state);
            }
            self.visited_games_states.insert(new_state.clone());
            let valid_moves = new_state.valid_moves();
            for valid_move in &valid_moves {
                let new_state_to_visit = new_state.handle_move(valid_move);
                if !self.visited_games_states.contains(&new_state_to_visit) {
                    self.states_to_visit.push(new_state_to_visit);
                }
            }
        }
        None
    }
}
