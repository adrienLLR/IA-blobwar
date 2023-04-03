//! Implementation of the min max algorithm.
use super::Strategy;
use crate::configuration::{Configuration, Movement};
use crate::shmem::AtomicMove;
use std::fmt;
use std::sync::Mutex;
use rayon::prelude::*;

/// Min-Max algorithm with a given recursion depth.
pub struct MinMax(pub u8);

impl Strategy for MinMax {
    fn compute_next_move(&mut self, state: &Configuration) -> Option<Movement> {
        let (best_move, _) = min_max_rec_par(state,self.0);
        best_move
        // compute_next_move_iteratively(&self.0, state)
    }
}

/// Implement the next move iteratively
pub fn compute_next_move_iteratively(initial_depth: &u8, initial_state: &Configuration) -> Option<Movement> {
    type NodeType<'a> = (Option<Movement>, Configuration<'a>, i32, u8, i32);
    let maximizer = !initial_state.current_player;
    let initial_root_value = if maximizer {std::i32::MIN} else {std::i32::MAX};
    let mut best_move: Option<Movement> = None;
    let mut stack: Vec<NodeType> = Vec::new();
    stack.push((None::<Movement>, *initial_state, initial_root_value, initial_depth + 1, 0));

    // We pop the current node to work with it, we either push it back or not, depending if we are backtracking yet or not
    while let Some((move_to_get_here, current_state, node_value, depth, iter_pos)) = stack.pop() {
        let maximizer = !current_state.current_player;
        let some_movement = current_state.movements().nth(iter_pos as usize);
        // If the current node is a leaf or it is not and there is no movement left to explore,
        // we update the value of the father and remove this current node from the stack
        if (some_movement.is_none() && depth > 0) || depth == 0 || current_state.game_over() {
            if let Some((_, _, previous_node_value, _, _)) = stack.last_mut() {
                // If the node is a leaf, we evaluate with heuristic function its value, otherwise, we juste take its current value
                let val = if some_movement.is_none() && depth > 0 {node_value} else {current_state.value() as i32};
                *previous_node_value = if !maximizer {i32::max(*previous_node_value, val)} else {i32::min(*previous_node_value, val)};
                // If we are just below the root and that the root value is that of this node
                // it means this is the best move, so we update the value of the variable
                if depth == *initial_depth && val == *previous_node_value {
                    best_move = move_to_get_here;
                }
            } 
        } 
        // If we are not on a leaf and there is a move to play, we explore the branch
        else {
            let movement = current_state.movements().nth(iter_pos as usize).unwrap();
            let new_node = current_state.play(&movement);
            let new_node_value = if !maximizer {std::i32::MIN} else {std::i32::MAX};
            stack.push((move_to_get_here, current_state, node_value, depth, iter_pos + 1));
            stack.push((Some(movement), new_node, new_node_value, (depth - 1) as u8, 0));
        }
    }
    best_move
}

/// Implement the next move recursively
pub fn min_max_rec(state: &Configuration, depth: u8) -> (Option<Movement>, i8) {
    if depth == 0 || state.movements().next().is_none() {
        return (None, state.value());
    }
    let maximizer = !state.current_player;
    let mut value = 64;
    let mut best_move = None;
    if maximizer {
        value = -value;
        state.movements().for_each(|m| {
            let (_, temp) = min_max_rec(&state.play(&m), depth - 1);
            if value < temp {
                value = temp;
                best_move = Some(m);
            }
        });
    } 
    else {
        state.movements().for_each(|m| {
            let (_, temp) = min_max_rec(&state.play(&m), depth - 1);
            if value > temp {
                value = temp;
                best_move = Some(m);
            }
        });
    }
    return (best_move, value);
}


/// Implement the next move recursively and with use of parallelism
pub fn min_max_rec_par(state: &Configuration, depth: u8) -> (Option<Movement>, i8) {
    if depth == 0 || state.movements().next().is_none() {
        return (None, state.value());
    }
    let maximizer = !state.current_player;
    let movements = state.movements().collect::<Vec<_>>();
    let (best_move, value) = movements.into_par_iter().map(|m| {
    // let (best_move, value) = state.movements().par_bridge().map(|m| { 
        let (_, v) = min_max_rec(&state.play(&m), depth - 1);
        (Some(m), v)
    }).reduce(|| (None, if maximizer { -64 } else { 64 }), |(m1, v1) , (m2, v2)| {
        if (maximizer && v1 < v2) || (!maximizer && v1 > v2) {
            (m2, v2)
        } else {
            (m1, v1)
        }
    });
    (best_move, value)
}



impl fmt::Display for MinMax {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Min - Max (max level: {})", self.0)
    }
}

/// Anytime min max algorithm.
/// Any time algorithms will compute until a deadline is hit and the process is killed.
/// They are therefore run in another process and communicate through shared memory.
/// This function is intended to be called from blobwar_iterative_deepening.
pub fn min_max_anytime(state: &Configuration) {
    let mut movement = AtomicMove::connect().expect("failed connecting to shmem");
    let movements = Mutex::new(Vec::new());
    (1..100).into_par_iter().for_each(|depth| {
        let move_result = MinMax(depth).compute_next_move(state);
        let mut borrowed_moves = movements.lock().unwrap();
        borrowed_moves.push(move_result);
        drop(borrowed_moves);
    });
    for m in movements.lock().unwrap().iter() {
        movement.store(*m);
    }
}
