use std::{
    cmp::{max, min},
    mem::swap,
};

use arrayvec::ArrayVec;
use onitama_move_gen::{eval::Eval, gen::Game, tablebase::TableBase};

use crate::transpose::Transpose;

pub struct Agent {
    tablebase: TableBase,
    transpose: Transpose,
    eval: i8,
}

impl Agent {
    pub fn new(tablebase: TableBase) -> Self {
        Self {
            tablebase,
            transpose: Transpose::new(),
            eval: 0,
        }
    }

    pub fn search(&mut self, game: Game, depth: usize) -> Game {
        let games = game.forward().collect();
        self.bns(games, i8::MIN, i8::MAX, depth)
    }

    fn bns(&mut self, games: ArrayVec<[Game; 40]>, alpha: i8, beta: i8, depth: usize) -> Game {
        if self.eval == i8::MIN {
            return games[0];
        }
        let mut good_games = ArrayVec::new();
        for new_game in games.clone() {
            if new_game.is_loss() {
                return new_game;
            }
            if !self.scout_all(new_game, Eval(self.eval).forward(), depth) {
                good_games.push(new_game)
            }
        }
        println!(
            "a:{}, b:{}, e:{}, g:{}, t:{}",
            alpha,
            beta,
            self.eval,
            good_games.len(),
            games.len()
        );
        if good_games.len() == 1 || self.eval + 1 == beta && !good_games.is_empty() {
            good_games.pop().unwrap()
        } else if good_games.is_empty() {
            let beta = self.eval;
            let offset = (beta.wrapping_sub(alpha) as u8 / games.len() as u8) as i8;
            self.eval = max(beta - max(offset, 1), self.eval.saturating_sub(10));
            self.bns(games, alpha, beta, depth)
        } else {
            let alpha = self.eval;
            let offset = (beta.wrapping_sub(alpha) as u8 / good_games.len() as u8) as i8;
            self.eval = min(beta - max(offset, 1), self.eval.saturating_add(10));
            self.bns(good_games, alpha, beta, depth)
        }
    }

    fn scout_all(&mut self, game: Game, beta: Eval, depth_left: usize) -> bool {
        debug_assert!(beta != Eval::new_win(1));
        for new_game in game.forward() {
            if new_game.is_loss() {
                return true;
            }
            if beta == Eval::new_win(2) {
                continue;
            }
            if !self.scout_cut(new_game, beta.forward(), depth_left) {
                return true;
            }
        }
        false // happy path
    }

    fn scout_cut(&mut self, game: Game, beta: Eval, depth_left: usize) -> bool {
        if let Some(forward) = self.sorted_games(game) {
            if beta == Eval::new_win(1) {
                return false;
            }
            let my_count = game.count_pieces() <= 2;
            for new_game in forward {
                let other_count = game.count_pieces() <= 2;
                let val = if depth_left == 0 || my_count && other_count {
                    self.eval(new_game) <= beta.forward()
                } else {
                    !self.scout_all(new_game, beta.forward(), depth_left - 1)
                };
                if val {
                    self.transpose[game] = max(self.transpose[game], beta);
                    return true; // happy path
                } else {
                    self.transpose[game] = min(self.transpose[game], Eval(beta.0 - 1))
                }
            }
            false
        } else {
            true // happy path
        }
    }

    fn sorted_games(&mut self, game: Game) -> Option<SelectionIter> {
        let mut games: ArrayVec<[Game; 40]> = ArrayVec::new();
        for mut new_game in game.forward() {
            if new_game.is_loss() {
                return None;
            }
            new_game.table |= (self.transpose[new_game].0 as u32) << 16;
            debug_assert!(new_game.table.wrapping_shr(16) as i8 == self.transpose[new_game].0);
            unsafe { games.push_unchecked(new_game) }
        }
        Some(SelectionIter(games))
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
        let mut elem = self.0.pop()?;
        let other = self
            .0
            .iter_mut()
            .max_by_key(|new_game| new_game.table.wrapping_shr(16) as i8);
        if let Some(other) = other {
            if other.table.wrapping_shr(16) as i8 > elem.table.wrapping_shr(16) as i8 {
                swap(&mut elem, other)
            }
        }
        elem.table &= (1 << 16) - 1;
        Some(elem)
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

    // #[test]
    // fn test_transpose() {
    //     const TEST_GAME: Game = Game {
    //         my: 0b11111 | 2 << 25,
    //         other: 0b11111 | 2 << 25,
    //         cards: 0b00011 | 0b01100 << 16,
    //         table: 4,
    //         hash: 0,
    //     };

    //     let mut agent = Agent::new(TableBase::new([0, 1, 2, 3, 4]));

    //     for depth in 0..8 {
    //         agent.search(TEST_GAME, depth);
    //     }
    //     // println!("{}", agent.transpose.0.iter().max().unwrap());
    //     // println!("{}", agent.count);
    //     // normal sort 2244078
    //     // reverse sort 4344789
    //     // no sort 2075894
    //     // big tt normal sort 476768
    //     // medium tt zobrist 680827
    //     // big tt correct zobrist 450429
    //     // medium tt zobrist 685519
    // }

    // #[test]
    // fn table_base_agent() {
    //     // [6, 13, 15, 12, 9]
    //     let mut agent = Agent::new(TableBase::new([0, 1, 2, 3, 4]));
    //     let cards = card_config([0, 1, 2, 3, 4]);

    //     for other_king in 0..25 {
    //         for my_king in 0..25 {
    //             for (my, other) in piece_config(1 << my_king | 1 << 24 >> other_king) {
    //                 for &(cards, center) in &cards {
    //                     let game = Game {
    //                         cards,
    //                         table: center,
    //                         my: 1 << my & PIECE_MASK | 1 << my_king | my_king << 25,
    //                         other: 1 << other & PIECE_MASK | 1 << other_king | other_king << 25,
    //                         hash: 0,
    //                     };
    //                     if !game.is_loss() && !game.is_other_loss() {
    //                         assert_eq!(
    //                             agent.alpha_beta(game, Eval::new_loss(0), Eval::new_win(1), 4),
    //                             agent.tablebase[game]
    //                         );
    //                     }
    //                 }
    //             }
    //         }
    //         dbg!("done");
    //     }
    // }

    #[test]
    fn wrapping() {
        let a = 3i8;
        let b = -5i8;
        assert_eq!(a.wrapping_sub(b) as u8, 8)
    }
}
