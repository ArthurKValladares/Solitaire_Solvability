use serde::Serialize;

use crate::{card::*, Game};
use std::{collections::HashSet, fmt};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize)]
pub enum CardPosition {
    Stock,
    Waste,
    Foundation(u8),
    // tableau_idx, card_idx
    Tableau((u8, u8)),
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize)]
pub struct Move {
    pub from: CardPosition,
    pub to: CardPosition,
}

impl Move {
    pub fn pretty_string(&self, game: &Game) -> String {
        let from_card = match self.from {
            CardPosition::Stock => game.stock.0.last().unwrap(),
            CardPosition::Waste => game.waste.0.last().unwrap(),
            CardPosition::Foundation(idx) => &game.foundations[idx as usize],
            CardPosition::Tableau((tableau_idx, card_idx)) => {
                &game.tableaus[tableau_idx as usize].0[card_idx as usize]
            }
        };
        let to_card = match self.to {
            CardPosition::Stock => game.stock.0.last(),
            CardPosition::Waste => game.waste.0.last(),
            CardPosition::Foundation(idx) => Some(&game.foundations[idx as usize]),
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

impl Game {
    fn get_move_from_stock(&self) -> Option<Move> {
        // If stock is not empty we can draw, otherwise we can restock
        if !self.stock.0.is_empty() {
            Some(Move {
                from: CardPosition::Stock,
                to: CardPosition::Waste,
            })
        } else if !self.waste.0.is_empty() {
            // Restock
            Some(Move {
                from: CardPosition::Waste,
                to: CardPosition::Stock,
            })
        } else {
            None
        }
    }

    fn get_move_from_waste_to_tableau(&self, tableau_idx: usize) -> Option<Move> {
        if let Some(card) = self.waste.0.last() {
            if let Some(tableau_card) = self.tableaus[tableau_idx].0.last() {
                // If tableau is not empty, only move is waste card can be placed on tableau
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
                // If tableau is empty, only kings can be moved there
                Some(Move {
                    from: CardPosition::Waste,
                    to: CardPosition::Tableau((tableau_idx as u8, 0 as u8)),
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_moves_from_waste(&self) -> HashSet<Move> {
        let mut set = HashSet::new();
        if !self.waste.0.is_empty() {
            let card = self.waste.0.last().unwrap();

            // Check if card can be moves directly to foundation
            if self.can_move_card_to_foundation(*card) {
                set.insert(Move {
                    from: CardPosition::Waste,
                    to: CardPosition::Foundation(suit_rank(*card)),
                });
            }

            // Check if card can be moved to every tableau
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
        // Check if to tableau is empty
        if let Some(to_tableau_card) = self.tableaus[to_tableau_idx].0.last() {
            if Self::can_be_placed_on_top_of(*to_tableau_card, card) {
                // If the card we are moving can be placed on top of the top card in to tableau,
                // Add move to set
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
            // If to tableau is empty, the only card we can move there is a king
            Some(Move {
                from: CardPosition::Tableau((from_tableau_idx as u8, card_idx as u8)),
                to: CardPosition::Tableau((to_tableau_idx as u8, 0 as u8)),
            })
        } else {
            None
        }
    }

    fn get_tableau_moves_from_tableau(&self, from_tableau_idx: usize) -> HashSet<Move> {
        let mut set = HashSet::new();
        // Check first unlocked card
        let first_unlocked_idx = self.first_unlocked_idx[from_tableau_idx];
        // If it exists
        if first_unlocked_idx != u8::MAX {
            // Iterate trough every tableau
            self.tableaus
                .iter()
                .enumerate()
                .for_each(|(to_tableau_idx, _)| {
                    if from_tableau_idx != to_tableau_idx {
                        // If the tableaus are different, and I can move the stack between them, add to set
                        if let Some(mv) = self.get_specific_move_between_tableaus(
                            from_tableau_idx,
                            first_unlocked_idx as usize,
                            to_tableau_idx,
                        ) {
                            set.insert(mv);
                        }
                    }
                });

            // Check rest of the stack, only if it opens a card that can move to a foundation
            for index in (first_unlocked_idx as usize + 1)..self.tableaus[from_tableau_idx].0.len()
            {
                if self.can_move_card_to_foundation(self.tableaus[from_tableau_idx].0[index - 1]) {
                    self.tableaus
                        .iter()
                        .enumerate()
                        .for_each(|(to_tableau_idx, _)| {
                            if from_tableau_idx != to_tableau_idx {
                                if let Some(mv) = self.get_specific_move_between_tableaus(
                                    from_tableau_idx,
                                    index,
                                    to_tableau_idx,
                                ) {
                                    set.insert(mv);
                                }
                            }
                        });
                }
            }
        } else {
            // The only way for there not to be unlocked cards is an empty tableau
            assert!(self.tableaus[from_tableau_idx].0.is_empty());
        }
        set
    }

    fn get_move_from_tableau_to_foundation(&self, from_tableau_idx: usize) -> Option<Move> {
        if let Some(from_tableau_card) = self.tableaus[from_tableau_idx].0.last() {
            // If the tableau is not empty, check if we can move that card to a foundation
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
        // For every tableau
        for (from_tableau_idx, _) in self.tableaus.iter().enumerate() {
            // Check if we can move the card to a foundation
            if let Some(mv) = self.get_move_from_tableau_to_foundation(from_tableau_idx) {
                set.insert(mv);
            }
            // Get the moves from this tableau to another tableau
            set.extend(self.get_tableau_moves_from_tableau(from_tableau_idx));
        }
        set
    }

    pub fn valid_moves(&self) -> HashSet<Move> {
        // ANOTHER TODO: We could parallelize some of this, sorta annoying tho.
        let mut valid_moves = HashSet::new();
        valid_moves.extend(self.get_moves_from_waste());
        valid_moves.extend(self.get_moves_from_tableau());
        if let Some(mv) = self.get_move_from_stock() {
            valid_moves.insert(mv);
        }
        valid_moves
    }
}

// TODO: Moves to two different empty spaces are the same. Reduce that symmetry
