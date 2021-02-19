use std::cmp::max;

use onitama_move_gen::{eval::Eval, gen::Game, tablebase::TableBase};

pub struct Agent {
    table: TableBase,
}

impl Agent {
    pub fn new(table: TableBase) -> Self {
        Self { table }
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
            }
            println!("score: {}", score);
        }
        alpha_game
    }

    fn alpha_beta(&self, game: Game, mut alpha: Eval, beta: Eval, depth_left: usize) -> Eval {
        if game.is_loss() || alpha == Eval::new_win(1) {
            return alpha;
        };
        alpha = max(alpha, Eval::new_loss(1));
        if depth_left == 0 {
            return self.quiesce(game, alpha, beta);
        }
        if beta == Eval::new_loss(0) {
            return Eval::new_win(1);
        }
        assert!(beta != Eval::new_loss(0));
        for new_game in game.forward() {
            let score = self
                .alpha_beta(new_game, beta.forward(), alpha.forward(), depth_left - 1)
                .backward();
            if score >= beta {
                return Eval::new_win(1);
            }
            alpha = max(score, alpha);
            assert!(alpha != Eval::new_win(1));
        }
        alpha
    }

    fn quiesce(&self, game: Game, alpha: Eval, beta: Eval) -> Eval {
        self.table.eval(game)
    }
}
