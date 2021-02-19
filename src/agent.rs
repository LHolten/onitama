use arrayvec::ArrayVec;
use onitama_move_gen::{eval::Eval, gen::Game, tablebase::TableBase};

use crate::transpose::Transpose;

pub struct Agent {
    tablebase: TableBase,
    transpose: Transpose,
    count: usize,
}

impl Agent {
    pub fn new(tablebase: TableBase) -> Self {
        Self {
            tablebase,
            transpose: Transpose::new(),
            count: 0,
        }
    }

    pub fn search(&mut self, game: Game, depth: usize) -> Game {
        let mut alpha = Eval::new_loss(1);
        let mut alpha_game = game;
        for new_game in self.sorted_games(game) {
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
        }
        println!("alpha: {}", alpha);
        alpha_game
    }

    fn alpha_beta(&mut self, game: Game, mut alpha: Eval, beta: Eval, depth_left: usize) -> Eval {
        assert!(alpha < beta);
        let res = (|| {
            if game.is_loss() {
                return Eval::new_loss(0);
            }
            if Eval::new_loss(1) > alpha {
                alpha = Eval::new_loss(1);
                if alpha >= beta {
                    return Eval::new_win(1);
                }
            }
            if depth_left == 0 {
                return self.eval(game);
            }
            for new_game in self.sorted_games(game) {
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
        })();
        // self.count += 1;
        self.transpose[game] = res;
        // self.transpose[game] += 1;
        res
    }

    fn sorted_games(&self, game: Game) -> ArrayVec<[Game; 40]> {
        let mut games: ArrayVec<[Game; 40]> = ArrayVec::new();
        for mut new_game in game.forward() {
            new_game.table |= (self.transpose[new_game].0 as u32) << 16;
            assert!(new_game.table.wrapping_shr(16) as i8 == self.transpose[new_game].0);
            games.push(new_game)
        }
        games.sort_unstable_by_key(|new_game| new_game.table.wrapping_shr(16) as i8);
        for new_game in games.iter_mut() {
            new_game.table &= (1 << 16) - 1;
        }
        games
    }

    fn eval(&mut self, game: Game) -> Eval {
        self.count += 1;
        self.tablebase.eval(game)
    }
}

#[cfg(test)]
mod test {
    use onitama_move_gen::{gen::Game, tablebase::TableBase};

    use super::Agent;

    #[test]
    fn test_transpose() {
        const TEST_GAME: Game = Game {
            my: 0b11111 | 2 << 25,
            other: 0b11111 | 2 << 25,
            cards: 0b00011 | 0b01100 << 16,
            table: 4,
            hash: 0,
        };

        let mut agent = Agent::new(TableBase::new([0, 1, 2, 3, 4]));

        for depth in 0..8 {
            agent.search(TEST_GAME, depth);
        }
        // println!("{}", agent.transpose.0.iter().max().unwrap());
        println!("{}", agent.count);
        // normal sort 2244078
        // reverse sort 4344789
        // no sort 2075894
        // big tt normal sort 476768
        // medium tt zobrist 680827
        // big tt correct zobrist 450429
        // mecium tt zobrist 685519
    }
}
