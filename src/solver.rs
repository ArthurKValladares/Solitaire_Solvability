use super::Game;
use std::collections::HashSet;

pub struct Solver {
    original_game: Game,
    visited_games_states: HashSet<Game>,
    states_to_visit: Vec<Game>,
    visited_states: Vec<Game>,
}

impl Solver {
    pub fn new() -> Self {
        let original_game = Game::new();
        let mut visited_games_states = HashSet::new();
        visited_games_states.insert(original_game.clone());
        let mut states_to_visit = Vec::new();
        let valid_moves = original_game.valid_moves();
        for valid_move in &valid_moves {
            states_to_visit.push(original_game.handle_move(valid_move));
        }
        let visited_states = vec![original_game.clone()];
        Self {
            original_game,
            visited_games_states,
            states_to_visit,
            visited_states,
        }
    }

    pub fn is_solvable(&mut self) -> Option<Game> {
        println!("Original Game:\n{}", self.original_game);
        // TODO: Need to keep track of depth so that we can keep a stack of moves with the solution
        while !self.states_to_visit.is_empty() {
            let new_state = self.states_to_visit.pop().unwrap();
            println!("\nCurrent State:\n{}", new_state);
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
