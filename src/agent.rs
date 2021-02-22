use std::todo;

use arrayvec::ArrayVec;
use onitama_move_gen::{eval::Eval, gen::Game, tablebase::TableBase};

use crate::transpose::Transpose;

pub struct Agent {
    tablebase: TableBase,
    transpose: Transpose,
    // count: usize,
}

impl Agent {
    pub fn new(tablebase: TableBase) -> Self {
        Self {
            tablebase,
            transpose: Transpose::new(),
            // count: 0,
        }
    }

    pub fn search(&mut self, game: Game, depth: usize) -> Game {
        let mut alpha = Eval::new_loss(1);
        let mut alpha_game = game;
        if let Some(forward) = self.sorted_games(game) {
            for new_game in forward {
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
        } else {
            alpha = Eval::new_win(1);
            alpha_game = game
                .forward()
                .find(|new_game| new_game.is_loss())
                .unwrap_or(game);
        }
        println!("alpha: {}", alpha);
        alpha_game
    }

    fn alpha_beta(&mut self, game: Game, mut alpha: Eval, beta: Eval, depth_left: usize) -> Eval {
        debug_assert!(alpha < beta);
        debug_assert!(!game.is_loss() && !game.is_other_loss());
        if Eval::new_loss(1) > alpha {
            alpha = Eval::new_loss(1);
            if alpha >= beta {
                self.transpose[game] = alpha;
                return Eval::new_win(1);
            }
        }
        if let Some(forward) = self.sorted_games(game) {
            for new_game in forward {
                debug_assert!(!new_game.is_loss() && !new_game.is_other_loss());
                let score = if depth_left >= new_game.count_pieces() {
                    self.alpha_beta(new_game, beta.forward(), alpha.forward(), depth_left - 1)
                        .backward()
                } else {
                    self.eval(new_game).backward()
                };
                if score > alpha {
                    alpha = score;
                    if alpha >= beta {
                        self.transpose[game] = alpha;
                        return Eval::new_win(1);
                    }
                }
            }
            self.transpose[game] = alpha;
            alpha
        } else {
            self.transpose[game] = Eval::new_win(1);
            Eval::new_win(1)
        }
    }

    #[inline(always)]
    fn sorted_games(&mut self, game: Game) -> Option<SelectionIter> {
        let mut games: ArrayVec<[Game; 40]> = ArrayVec::new();
        for mut new_game in game.forward() {
            if new_game.is_loss() {
                return None;
            }
            new_game.table |= (self.transpose[new_game].0 as u32) << 16;
            debug_assert!(new_game.table.wrapping_shr(16) as i8 == self.transpose[new_game].0);
            games.push(new_game)
        }
        games.sort_unstable_by_key(|new_game| new_game.table.wrapping_shr(16) as i8);
        for new_game in games.iter_mut() {
            new_game.table &= (1 << 16) - 1;
        }
        Some(games)
    }

    pub fn eval(&mut self, game: Game) -> Eval {
        // self.count += 1;
        debug_assert!(!game.is_loss() && !game.is_other_loss());
        self.tablebase.eval(game)
    }
}

struct SelectionIter(ArrayVec<[Game; 40]>);

impl Iterator for SelectionIter {
    type Item = Game;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use onitama_move_gen::{
        eval::Eval,
        gen::{Game, PIECE_MASK},
        tablebase::{card_config, piece_config, TableBase},
    };

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
        // println!("{}", agent.count);
        // normal sort 2244078
        // reverse sort 4344789
        // no sort 2075894
        // big tt normal sort 476768
        // medium tt zobrist 680827
        // big tt correct zobrist 450429
        // medium tt zobrist 685519
    }

    #[test]
    fn table_base_agent() {
        // [6, 13, 15, 12, 9]
        let mut agent = Agent::new(TableBase::new([0, 1, 2, 3, 4]));
        let cards = card_config([0, 1, 2, 3, 4]);

        for other_king in 0..25 {
            for my_king in 0..25 {
                for (my, other) in piece_config(1 << my_king | 1 << 24 >> other_king) {
                    for &(cards, center) in &cards {
                        let game = Game {
                            cards,
                            table: center,
                            my: 1 << my & PIECE_MASK | 1 << my_king | my_king << 25,
                            other: 1 << other & PIECE_MASK | 1 << other_king | other_king << 25,
                            hash: 0,
                        };
                        if !game.is_loss() && !game.is_other_loss() {
                            assert_eq!(
                                agent.alpha_beta(game, Eval::new_loss(0), Eval::new_win(1), 4),
                                agent.tablebase[game]
                            );
                        }
                    }
                }
            }
            dbg!("done");
        }
    }
}
