//! Alpha - Beta algorithm.
use std::fmt;
use std::sync::{Arc, Mutex};

use super::Strategy;
use crate::configuration::{Configuration, Movement};
use crate::shmem::AtomicMove;
use rayon::prelude::*;

/// Alpha - Beta algorithm with given maximum number of recursions.
pub struct AlphaBeta(pub u8);

impl fmt::Display for AlphaBeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Alpha - Beta (max level: {})", self.0)
    }
}

/*--------------------------------------------------------------------------------*/
/* ----------------------- AlphaBeta algorithm -----------------------------------*/
/*--------------------------------------------------------------------------------*/

impl Strategy for AlphaBeta {
    fn compute_next_move(&mut self, state: &Configuration) -> Option<Movement> {
        // let (best_move, _) = alpha_beta_par(state,self.0);
        // let (best_move, _) = alpha_beta_rec(state,self.0, 64, -64);
        let (best_move, _) = sorted_alpha_beta(state,self.0, 64, -64, &self.0);
        best_move
    }
}

impl AlphaBeta {
    /// Custom compute next move for the iterative deepening
    pub fn compute_next_move_custom(&mut self, state: &Configuration, previous_move : &Option<Movement>, original_depth: &u8) -> Option<Movement> {
        let (mvt,_) = itdp_alpha_beta(state,self.0, 64, -64, original_depth, previous_move);
        return mvt;
    }
}

/// Implement the recursive search for the minmax with alphabeta pruning algorithm.
/// The alpha value is the value that you should not exceed,
/// and the beta value the one you should not go under
pub fn alpha_beta_rec(state: &Configuration, depth: u8, mut alpha: i8, mut beta: i8) -> (Option<Movement>, i8) {
    if depth == 0 || state.movements().next().is_none() {
        return (None, state.value());
    }
    let maximizer = !state.current_player;
    let mut value = if maximizer {-64} else {64};
    let mut best_move = None;
    for m in state.movements() {
        let (_, temp) = alpha_beta_rec(&state.play(&m), depth - 1, alpha, beta);
        if (maximizer && value < temp) || (!maximizer && temp < value) {
            value = temp;
            best_move = Some(m);
        }
        // Suppose the current node is that of the maximizer therefore the previous is the minimizer.
        // So the value shall not exceed alpha because the maximizer will always pick the
        // biggest value out of all the nodes, so if the value returned is bigger than alpha,
        // the previous node will never pick this current node as a value so there is no need
        // to continue to explore this subtree. Same idea in reversed
        if (maximizer && value >= alpha) || (!maximizer && value <= beta) {
            return (best_move, value);
        }
        // Suppose again the current node is that of the maximizer. We have to update beta because it is the value
        // to no underpass. Same idea in reversed
        if maximizer {beta = i8::max(value, beta)} else {alpha = i8::min(value, alpha);};
    }
    return (best_move, value);
}


///alpha beta para
/// A better parallel algorithm can consist to compute node in parral if the first node dosn't cut.
pub fn alpha_beta_par(state: &Configuration, depth: u8) -> (Option<Movement>, i8) {
    if depth == 0 || state.movements().next().is_none() {
        return (None, state.value());
    }
    let maximizer = !state.current_player;
    let movements = state.movements_valuable().collect::<Vec<_>>();
    // movements.shuffle(&mut rand::thread_rng());
    let (best_move, value) = movements.into_par_iter().map(|m| {
    // let (best_move, value) = state.movements().par_bridge().map(|m| { 
        let (_, v) = alpha_beta_rec(&state.play(&m), depth - 1, 64, -64);
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

/// Alpha beta but with sorted movements
pub fn sorted_alpha_beta(state: &Configuration, depth: u8, mut alpha: i8, mut beta: i8, original_depth: &u8) -> (Option<Movement>, i8) {
    if depth == 0 || state.movements().next().is_none() {
        return (None, state.value());
    }
    let k = 4; // Depth to which we use the movements as sorted
    let maximizer = !state.current_player;
    let mut value = if maximizer {-64} else {64};
    let mut best_move = None;
    if *original_depth > 3 && depth >= *original_depth-k {
        for m in state.movements_sorted() {
            let (_, temp) = sorted_alpha_beta(&state.play(&m), depth - 1, alpha, beta, original_depth);
            if (maximizer && value < temp) || (!maximizer && temp < value) {
                value = temp;
                best_move = Some(m);
            }
            if (maximizer && value >= alpha) || (!maximizer && value <= beta) {
                return (best_move, value);
            }
            if maximizer {beta = i8::max(value, beta)} else {alpha = i8::min(value, alpha);};
        }
    }
    else {
        for m in state.movements_random() {
            let (_, temp) = alpha_beta_rec(&state.play(&m), depth - 1, alpha, beta);
            if (maximizer && value < temp) || (!maximizer && temp < value) {
                value = temp;
                best_move = Some(m);
            }
            if (maximizer && value >= alpha) || (!maximizer && value <= beta) {
                return (best_move, value);
            }
            if maximizer {beta = i8::max(value, beta)} else {alpha = i8::min(value, alpha);};
        }
    }
    return (best_move, value);
}

/// Alpha beta but for iterative deepening
pub fn itdp_alpha_beta(state: &Configuration, depth: u8, mut alpha: i8, mut beta: i8, original_depth: &u8, previous_move: &Option<Movement>) -> (Option<Movement>, i8) {
    let maximizer = !state.current_player;
    let mut value = if maximizer {-64} else {64};
    let mut best_move = None;

    if *previous_move != None {
        let new_iter = Configuration::put_element_infront_iterator(state.movements_sorted(), previous_move.unwrap());
        for m in new_iter {
            let (_, temp) = sorted_alpha_beta(&state.play(&m), depth - 1, alpha, beta, original_depth);
            if (maximizer && value < temp) || (!maximizer && temp < value) {
                value = temp;
                best_move = Some(m);
            }
            if (maximizer && value >= alpha) || (!maximizer && value <= beta) {
                return (best_move, value);
            }
            if maximizer {beta = i8::max(value, beta)} else {alpha = i8::min(value, alpha);};
        }
    }
    else {
        for m in state.movements_sorted() {
            let (_, temp) = sorted_alpha_beta(&state.play(&m), depth - 1, alpha, beta, original_depth);
            if (maximizer && value < temp) || (!maximizer && temp < value) {
                value = temp;
                best_move = Some(m);
            }
            if (maximizer && value >= alpha) || (!maximizer && value <= beta) {
                return (best_move, value);
            }
            if maximizer {beta = i8::max(value, beta)} else {alpha = i8::min(value, alpha);};
        }
    }
    return (best_move, value);
}


/// Anytime alpha beta algorithm.
/// Any time algorithms will compute until a deadline is hit and the process is killed.
/// They are therefore run in another process and communicate through shared memory.
/// This function is intended to be called from blobwar_iterative_deepening.
pub fn alpha_beta_anytime(state: &Configuration) {
    // alpha_beta_anytime_par(state)
    alpha_beta_anytime_seq(state)
    // anytime_original(state)
}

/// Anytime alpha beta sequential algorithm
pub fn alpha_beta_anytime_seq(state: &Configuration) {
    let mut movement = AtomicMove::connect().expect("failed connecting to shmem");
    let mut previous_movement: Option<Movement> = None;
    for depth in 1..100 {
        let chosen_movement = AlphaBeta(depth).compute_next_move_custom(state, &previous_movement, &depth);
        previous_movement = chosen_movement;
        movement.store(chosen_movement);
        println!("{}", depth);
    }
}

/// Original anytime
pub fn anytime_original(state: &Configuration) {
    let mut movement = AtomicMove::connect().expect("failed connecting to shmem");
    for depth in 1..100 {
        let chosen_movement = AlphaBeta(depth).compute_next_move(state);
        movement.store(chosen_movement);
    }
}



use std::marker::{Send, Sync};
/// Safe atomic move new struct
pub struct SafeAtomicMove {
    inner: AtomicMove,
}

unsafe impl Send for SafeAtomicMove {}
unsafe impl Sync for SafeAtomicMove {}

impl SafeAtomicMove {
    /// Function connect
    pub fn connect() -> Result<Self, Box<dyn std::error::Error>> {
        let inner = AtomicMove::connect()?;
        Ok(SafeAtomicMove { inner })
    }
    /// Function store
    pub fn store(&mut self, movement: Option<Movement>) {
        self.inner.store(movement);
    }
}

/// Anytime alpha beta parallel algorithm
pub fn alpha_beta_anytime_par(state: &Configuration) {
    let movement = Arc::new(Mutex::new(SafeAtomicMove::connect().expect("failed connecting to shmem")));

    (1..100).into_par_iter().for_each(|depth| {
        let chosen_movement = AlphaBeta(depth).compute_next_move(state);
        {
            let mut shared_movement = movement.lock().unwrap();
            shared_movement.store(chosen_movement);
        }
        println!("{}", depth);
    });
}