use super::Game;

pub struct Solver {
    original_game: Game,
    // TODO: Will need a set of "game states" that have already been visited, so that we never visit the same node twice.
    // We need a efficient way to save each game state that is not just saving the entire game state, since that will be memory intensive.
    // Might just be as simple as saving all moves made from the starting state.
    // Would be easy to reconstruct, but is that really cheaper than saving the entire state?
}

impl Solver {
    pub fn new() -> Self {
        Self {
            original_game: Game::default(),
        }
    }
}
