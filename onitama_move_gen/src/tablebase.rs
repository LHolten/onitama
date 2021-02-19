use std::{
    alloc::{alloc_zeroed, Layout},
    cmp::{max, min},
    mem::take,
    ops::{Index, IndexMut},
};

use bitintr::{Andn, Pext, Popcnt};

use crate::{
    eval::Eval,
    gen::{Game, PIECE_MASK},
    ops::{BitIter, CardIter},
};

type TableData = [[[[[Eval; 26]; 26]; 25]; 25]; 30];
pub struct TableBase(Box<TableData>, Vec<Game>);

impl TableBase {
    pub fn new(cards: [u32; 5]) -> Self {
        let val = unsafe {
            let layout = Layout::new::<TableData>();
            Box::from_raw(alloc_zeroed(layout) as *mut TableData)
        };
        let mut table = TableBase(val, Vec::new());
        let cards = card_config(cards);

        for other_king in 0..25 {
            for (my, other) in piece_config(1 << 24 >> other_king) {
                let full_other = 1 << 24 >> other | 1 << 24 >> other_king;
                let my_king_iter = if other_king == 22 {
                    BitIter((1 << 22).andn(PIECE_MASK))
                } else {
                    BitIter((1 << 22).andn(full_other))
                };
                for my_king in my_king_iter {
                    for &(cards, center) in &cards {
                        let game = Game {
                            cards,
                            table: center,
                            my: 1 << my & PIECE_MASK | 1 << my_king & !full_other | my_king << 25,
                            other: 1 << other & PIECE_MASK | 1 << other_king | other_king << 25,
                            hash: 0,
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
                    if table[new_game] == Eval::new_loss(0) || table[new_game] == Eval::new_tie() {
                        eval = Eval::new_tie();
                        break;
                    } else {
                        debug_assert!(table[new_game] >= Eval::new_tie());
                        eval = max(eval, table[new_game].backward());
                    }
                }
                debug_assert!(eval <= Eval::new_tie());
                debug_assert!(eval != Eval::new_loss(0));
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
        debug_assert!(eval > Eval::new_tie());
        if eval > self[game] {
            self[game] = eval;
            let prev_eval = eval.backward();
            for (mut prev_game, take) in game.backward() {
                self.check_loss(prev_game, prev_eval);
                if (prev_game.other & PIECE_MASK).popcnt() < 2 {
                    prev_game.other |= take;
                    self.check_loss(prev_game, prev_eval);
                }
            }
        }
    }

    fn check_loss(&mut self, game: Game, eval: Eval) {
        if eval < self[game] && self[game] <= Eval::new_tie() {
            self[game] = Eval::new_loss(0);
            self.1.push(game)
        }
    }

    pub fn eval(&self, game: Game) -> Eval {
        let my_king = game.my.wrapping_shr(25);
        let mut my = game.my & PIECE_MASK ^ 1 << my_king;
        if my == 0 {
            my |= 1 << 25
        }
        let other_king = game.other.wrapping_shr(25);
        let mut other = game.other & PIECE_MASK ^ 1 << other_king;
        if other == 0 {
            other |= 1 << 25;
        }

        // let m = BitIter(my).next().unwrap_or(25);
        // let o = BitIter(other).next().unwrap_or(25);

        // let new_game = Game {
        // my: (1 << m) & PIECE_MASK | 1 << my_king | my_king << 25,
        // other: (1 << o) & PIECE_MASK | 1 << other_king | other_king << 25,
        //     cards: game.cards,
        //     table: game.table,
        // };
        // self[new_game]

        let mut max_eval = Eval::new_loss(0);
        for m in BitIter(my) {
            let mut min_eval = Eval::new_win(1);
            for o in BitIter(other) {
                let new_game = Game {
                    my: (1 << m) & PIECE_MASK | 1 << my_king | my_king << 25,
                    other: (1 << o) & PIECE_MASK | 1 << other_king | other_king << 25,
                    cards: game.cards,
                    table: game.table,
                    hash: 0,
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
        let cards = compress_cards(game.cards, game.table) as usize;
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
        let cards = compress_cards(game.cards, game.table) as usize;
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

fn compress_cards(cards: u32, table: u32) -> u32 {
    let combined = cards | cards.wrapping_shr(16);
    let temp = (((1 << table) - 1) & combined).popcnt();
    temp * 6 + ((cards.pext(combined) & 7) - 1)
}

pub fn piece_config(mask: u32) -> impl Iterator<Item = (u32, u32)> {
    (0..26)
        .filter(move |&my| (1 << my) & mask == 0)
        .flat_map(move |my| {
            let mask = mask | (1 << my) & PIECE_MASK;
            (0..26)
                .filter(move |&other| (1 << 24 >> other) & mask == 0)
                .map(move |other| (my, other))
        })
}

pub fn card_config(cards: [u32; 5]) -> [(u32, u32); 30] {
    let mut res = [(0, 0); 30];
    let mut i = 0;
    for center in 0..5 {
        for my1 in 0..4 {
            for my2 in (my1 + 1)..5 {
                if center != my1 && center != my2 {
                    let my_cards = 1 << my1 | 1 << my2;
                    let mut other = CardIter::new(!my_cards ^ (1 << center));
                    let combined = 1 << cards[my1 as usize]
                        | 1 << cards[my2 as usize]
                        | 1 << 16 << cards[other.next().unwrap() as usize]
                        | 1 << 16 << cards[other.next().unwrap() as usize];
                    res[i] = (combined, cards[center as usize]);
                    i += 1;
                }
            }
        }
    }
    assert!(i == 30);
    res
}

fn compress_pieces(my: u32) -> u32 {
    let king = 1 << my.wrapping_shr(25);
    let mut piece_iter = BitIter(my & !king & PIECE_MASK);
    piece_iter.next().unwrap_or(25)
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
        for &(cards, table) in &card_config([3, 4, 5, 6, 7]) {
            let val = compress_cards(cards, table);
            assert!(val < 30);
            assert!(set.insert(val));
        }
        assert_eq!(set.len(), 30);

        let mut set2 = HashSet::new();
        for &(cards, table) in &card_config([7, 6, 5, 4, 3]) {
            let val = compress_cards(cards, table);
            assert!(val < 30);
            assert!(set2.insert(val));
        }
        assert_eq!(set2.len(), 30);

        assert_eq!(set, set2);
    }

    #[test]
    fn test_tablebase() {
        let mut counts = [0; 256];
        let table = TableBase::new([4, 3, 2, 1, 0]);
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
        assert_eq!(counts[0], 1229010);
        assert_eq!(counts[7], 299591);
        assert_eq!(counts[56], 8);
        for (i, &c) in counts.iter().enumerate() {
            println!("{}: {}", i, c);
        }
    }
}
