use std::{
    alloc::{alloc_zeroed, Layout},
    cmp::{max, min, Ordering},
    mem::take,
    ops::{Index, IndexMut},
};

use bitintr::{Andn, Popcnt};

use crate::{
    eval::Eval,
    gen::{Game, PIECE_MASK},
    ops::{BitIter, CardIter},
};

type TableData = [[[[[Eval; 26]; 26]; 25]; 25]; 30];
struct TableBase(Box<TableData>, Vec<Game>);

impl TableBase {
    pub fn new() -> Self {
        let val = unsafe {
            let layout = Layout::new::<TableData>();
            Box::from_raw(alloc_zeroed(layout) as *mut TableData)
        };
        let mut table = TableBase(val, Vec::new());
        let cards = card_config();

        for other_king in 0..25 {
            for (my, other) in piece_config(1 << 24 >> other_king) {
                let full_other = 1 << 24 >> other | 1 << 24 >> other_king;
                let my_king_iter = if other_king == 22 {
                    BitIter((1 << 22).andn(PIECE_MASK))
                } else {
                    BitIter((1 << 22).andn(full_other))
                };
                for my_king in my_king_iter {
                    for &(my_cards, other_cards) in &cards {
                        let game = Game {
                            my_cards,
                            other_cards,
                            my: (1 << 25).andn(1 << my)
                                | full_other.andn(1 << my_king)
                                | my_king << 25,
                            other: (1 << 25).andn(1 << other) | 1 << other_king | other_king << 25,
                        };
                        table[game] = Eval::new_loss(0);
                        for (mut prev_game, take) in game.backward() {
                            if !prev_game.is_other_loss() {
                                table.check_win(prev_game, Eval::new_win(1));
                            }
                            if (prev_game.other & PIECE_MASK).popcnt() < 2 {
                                prev_game.other |= take;
                                if !prev_game.is_other_loss() {
                                    table.check_win(prev_game, Eval::new_win(1));
                                }
                            }
                        }
                    }
                }
            }
            dbg!("done");
        }

        while !table.1.is_empty() {
            dbg!(table.1.len());
            for game in take(&mut table.1) {
                if table[game] != Eval::new_loss(0) {
                    continue;
                }
                let mut eval = Eval::new_loss(0);
                for new_game in game.forward() {
                    if table[new_game] == Eval::new_loss(0) {
                        eval = Eval::new_tie();
                        break;
                    } else {
                        eval = max(eval, table[new_game].backward());
                    }
                }
                table[game] = eval;
                if eval < Eval::new_tie() {
                    let prev_eval = eval.backward();
                    for (mut prev_game, take) in game.backward() {
                        table.check_win(prev_game, prev_eval);
                        if (prev_game.other & PIECE_MASK).popcnt() < 2 {
                            prev_game.other |= take;
                            table.check_win(prev_game, prev_eval);
                        }
                    }
                }
            }
        }

        table
    }

    fn check_win(&mut self, game: Game, eval: Eval) {
        if eval > self[game] {
            self[game] = eval;
            for (mut prev_game, take) in game.backward() {
                self.check_loss(prev_game);
                if (prev_game.other & PIECE_MASK).popcnt() < 2 {
                    prev_game.other |= take;
                    self.check_loss(prev_game);
                }
            }
        }
    }

    fn check_loss(&mut self, game: Game) {
        if self[game] == Eval::new_tie() {
            self[game] = Eval::new_loss(0);
            self.1.push(game)
        }
    }

    pub fn eval(&self, game: Game) -> Eval {
        let my_king = game.my.wrapping_shr(25);
        let mut my = game.my ^ 1 << my_king;
        if my == 0 {
            my |= 1 << 25
        }
        let other_king = game.other.wrapping_shr(25);
        let mut other = game.other ^ 1 << other_king;
        if other == 0 {
            other |= 1 << 25;
        }

        let mut max_eval = Eval::new_loss(0);
        for m in BitIter(my) {
            let mut min_eval = Eval::new_win(1);
            for o in BitIter(other) {
                let new_game = Game {
                    my: (1 << 25).andn(1 << m) | 1 << my_king | my_king << 25,
                    other: (1 << 25).andn(1 << o) | 1 << other_king | other_king << 25,
                    my_cards: game.my_cards,
                    other_cards: game.other_cards,
                };
                min_eval = min(min_eval, self[new_game])
            }
            max_eval = max(max_eval, min_eval);
        }
        max_eval
    }
}

impl Index<Game> for TableBase {
    type Output = Eval;

    fn index(&self, game: Game) -> &Self::Output {
        let cards = compress_cards(game.my_cards, game.other_cards) as usize;
        let my_king = game.my.wrapping_shr(25) as usize;
        let other_king = game.other.wrapping_shr(25) as usize;
        let my_pieces = compress_pieces(game.my) as usize;
        let other_pieces = compress_pieces(game.other) as usize;

        unsafe {
            self.0
                .get_unchecked(cards)
                .get_unchecked(my_king)
                .get_unchecked(other_king)
                .get_unchecked(my_pieces)
                .get_unchecked(other_pieces)
        }
    }
}

impl IndexMut<Game> for TableBase {
    fn index_mut(&mut self, game: Game) -> &mut Self::Output {
        let cards = compress_cards(game.my_cards, game.other_cards) as usize;
        let my_king = game.my.wrapping_shr(25) as usize;
        let other_king = game.other.wrapping_shr(25) as usize;
        let my_pieces = compress_pieces(game.my) as usize;
        let other_pieces = compress_pieces(game.other) as usize;

        unsafe {
            self.0
                .get_unchecked_mut(cards)
                .get_unchecked_mut(my_king)
                .get_unchecked_mut(other_king)
                .get_unchecked_mut(my_pieces)
                .get_unchecked_mut(other_pieces)
        }
    }
}

fn compress_cards(my_cards: u8, other_cards: u8) -> u8 {
    let mut card_iter = CardIter::new(my_cards);
    let total = card_iter.next().unwrap() * 5 + card_iter.next().unwrap();
    let total = if total >= 10 { 19 - total } else { total };
    total + 10 * ((!(my_cards | other_cards) - 1) & other_cards).popcnt()
}

pub fn piece_config(mask: u32) -> impl Iterator<Item = (u32, u32)> {
    (0..26)
        .filter(move |&my| (1 << my) & mask == 0)
        .flat_map(move |my| {
            let mask = mask | (1 << 25).andn(1 << my);
            (0..26)
                .filter(move |&other| 1 << 24 >> other & mask == 0)
                .map(move |other| (my, other))
        })
}

pub fn card_config() -> [(u8, u8); 30] {
    let mut res = [(0, 0); 30];
    let mut i = 0;
    for center in 0..5 {
        for my1 in 0..4 {
            if center != my1 {
                for my2 in my1 + 1..5 {
                    if center != my2 {
                        let my_cards = 1 << my1 | 1 << my2;
                        let other_cards = !my_cards ^ (1 << center);
                        res[i] = (my_cards, other_cards);
                        i += 1;
                    }
                }
            }
        }
    }
    assert!(i == 30);
    res
}

fn compress_pieces(my: u32) -> u32 {
    let mut piece_iter = BitIter((1 << my.wrapping_shr(25)).andn(my) & PIECE_MASK);
    let piece_pos = piece_iter.next().unwrap_or(25);
    // assert!(piece_iter.next().is_none());
    piece_pos
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{card_config, compress_cards, compress_pieces, piece_config, TableBase};

    #[test]
    fn test_pieces() {
        let mut set = HashSet::new();
        for (m, o) in piece_config(0) {
            let m = 1 << 24 >> m | 25 << 25;
            let o = 1 << 24 >> o | 25 << 25;
            assert!(set.insert((compress_pieces(m), compress_pieces(o))));
        }
        assert!(set.len() == 651)
    }

    #[test]
    fn test_compress_cards() {
        let mut set = HashSet::new();
        for &(my_cards, other_cards) in &card_config() {
            let val = compress_cards(my_cards, other_cards);
            assert!(val < 30);
            assert!(set.insert(val));
        }
        assert!(set.len() == 30)
    }

    #[test]
    fn test_tablebase() {
        let mut counts = [0; 256];
        let table = TableBase::new();
        for v in table.0.iter() {
            for v in v {
                for v in v {
                    for v in v {
                        for v in v {
                            counts[v.plies() as usize] += 1;
                        }
                    }
                }
            }
        }
        assert!(counts[0] == 1229010);
        assert!(counts[7] == 294903);
        assert!(counts[56] == 65);
        // for (i, &c) in counts.iter().enumerate() {
        //     println!("{}: {}", i, c);
        // }
    }
}
