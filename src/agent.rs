use std::cmp::max;

use onitama_move_gen::{eval::Eval, gen::Game, tablebase::TableBase};

use crate::transpose::Transpose;

pub struct Agent {
    table: TableBase,
    transpose: Transpose,
}

impl Agent {
    pub fn new(table: TableBase) -> Self {
        Self {
            table,
            transpose: Transpose::new(),
        }
    }

    pub fn search(&self, game: Game, depth: usize) -> Game {
        let mut alpha = Eval::new_loss(1);
        let mut alpha_game = game;
        for new_game in game.forward() {
            let score = self
                .alpha_beta(new_game, Eval::new_loss(0), alpha.forward(), depth)
                .backward();
            if score >= alpha {
                alpha = score;
                alpha_game = new_game;
                if alpha == Eval::new_win(1) {
                    break;
                }
            }
            println!("score: {}", score);
        }
        alpha_game
    }

    fn alpha_beta(&self, game: Game, mut alpha: Eval, beta: Eval, depth_left: usize) -> Eval {
        assert!(alpha < beta);
        if game.is_loss() {
            return Eval::new_loss(0);
        };
        if Eval::new_loss(1) > alpha {
            alpha = Eval::new_loss(1);
            if alpha >= beta {
                return Eval::new_win(1);
            }
        }
        if depth_left == 0 {
            return self.eval(game);
        }
        for new_game in game.forward() {
            let score = self
                .alpha_beta(new_game, beta.forward(), alpha.forward(), depth_left - 1)
                .backward();
            if score > alpha {
                alpha = score;
                if alpha >= beta {
                    return Eval::new_win(1);
                }
            }
        }
        alpha
    }

    fn eval(&self, game: Game) -> Eval {
        self.table.eval(game)
    }
}
