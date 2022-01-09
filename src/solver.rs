use super::Game;
use std::collections::HashSet;

pub struct Solver {
    original_game: Game,
    visited_games_states: HashSet<Game>,
    states_to_visit: Vec<Game>,
}

impl Solver {
    pub fn new() -> Self {
        let mut original_game = Game::default();
        original_game.initial_deal();
        let mut visited_games_states = HashSet::new();
        visited_games_states.insert(original_game.clone());
        let mut states_to_visit = Vec::new();
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

    pub fn is_solvable(&self) -> bool {
        false
    }
}
