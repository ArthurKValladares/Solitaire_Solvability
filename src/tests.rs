use crate::{card::*, Game};

pub fn is_valid_game_state(game: &Game) -> bool {
    let num_cards = {
        let cards_in_tableau = game
            .tableaus
            .iter()
            .fold(0, |acc, tableau| acc + tableau.0.len());
        let cards_in_stock = game.stock.0.len();
        let cards_in_waste = game.waste.0.len();
        let cards_in_foundations = game
            .foundations
            .iter()
            .fold(0, |acc, card| acc + card_rank(*card) as usize);

        cards_in_tableau + cards_in_stock + cards_in_waste + cards_in_foundations
    };

    num_cards == NUM_CARDS_DECK as usize
}
