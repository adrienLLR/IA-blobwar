//! Dumb greedy algorithm.
use super::Strategy;
use crate::configuration::{Configuration, Movement};
use std::fmt;

/// Dumb algorithm.
/// Amongst all possible movements return the one which yields the configuration with the best
/// immediate value.
pub struct Greedy();

impl fmt::Display for Greedy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Greedy")
    }
}

impl Strategy for Greedy {
    fn compute_next_move(&mut self, state: &Configuration) -> Option<Movement> {
        // No need to check move on the move yielded by the iterator because they are valid
        if !state.current_player {
            state.movements().max_by_key(|m| state.play(&m).value())
        }
        else {
            state.movements().min_by_key(|m| state.play(&m).value())
        }
    }
}
