extern crate blobwar;
use blobwar::board::Board;
//use blobwar::board::Board;
use blobwar::configuration::Configuration;
use blobwar::strategy::{Greedy, Human, MinMax, AlphaBeta, IterativeDeepening, IterativeStrategy};
use std::time::Instant;

fn main() {
    play_one_game();
    // stat_victoire(&board, 400);
}

pub fn play_one_game() {
    let board = Default::default();
    let mut game = Configuration::new(&board);
    let start = Instant::now();
    // game.battle(IterativeDeepening::new(IterativeStrategy::AlphaBeta), AlphaBeta(5));
    game.battle(Greedy(), AlphaBeta(8));
    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);
}

pub fn stat_victoire(board: &Board, n: f64) {
    let mut player1:f64 = 0.0;
    let mut player2:f64 = 0.0;
    let mut result;
    for _ in 1..n as i32 {
        let mut game = Configuration::new(board);
        result = game.battle(AlphaBeta(2), MinMax(2));
        while result == 0 {
            let mut game = Configuration::new(board);
            result = game.battle(AlphaBeta(2), MinMax(2));
        } 
        if result == 1 {player1 = player1 + 1.0;}
        else if result == -1 {player2 = player2 + 1.0;}
    }
    println!("Player 1 has {} win, Player 2 has {} win\n", player1/n, player2/n);

}
