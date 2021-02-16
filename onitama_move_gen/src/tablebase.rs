use std::{
    alloc::{alloc_zeroed, Layout},
    cmp::{max, Ordering},
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
    fn new() -> Self {
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
                        assert!(game.is_loss());
                        for (mut new_game, take) in game.backward() {
                            if !new_game.is_other_loss() {
                                table.check_win(new_game, Eval::new_win(1));
                            }
                            new_game.other |= take;
                            if !new_game.is_other_loss() {
                                table.check_win(new_game, Eval::new_win(1));
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
                    if new_game.is_loss() {
                        eval = Eval::new_win(1);
                        break;
                    }
                    eval = max(eval, table[new_game].backward());
                }
                table[game] = eval;
                let prev_eval = eval.backward();
                for (mut new_game, take) in game.backward() {
                    table.check_win(new_game, prev_eval);
                    if (game.other & PIECE_MASK).popcnt() < 2 {
                        new_game.other |= take;
                        table.check_win(new_game, prev_eval);
                    }
                }
            }
            dbg!("list");
        }

        table
    }

    fn check_win(&mut self, game: Game, eval: Eval) {
        if eval > self[game] {
            self[game] = eval;
            let prev_eval = eval.backward();
            for (mut new_game, take) in game.backward() {
                self.check_loss(new_game, prev_eval);
                if (game.other & PIECE_MASK).popcnt() < 2 {
                    new_game.other |= take;
                    self.check_loss(new_game, prev_eval);
                }
            }
        }
    }

    fn check_loss(&mut self, game: Game, eval: Eval) {
        if self[game] == Eval::new_tie() {
            self[game] = Eval::new_loss(0);
            self.1.push(game)
        }
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
    piece_iter.next().unwrap_or(25)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{card_config, compress_cards, piece_config, TableBase};

    #[test]
    fn test_pieces() {
        let mut set = HashSet::new();
        for val in piece_config(0) {
            assert!(set.insert(val));
        }
        assert!(set.len() == 651)
    }

    #[test]
    fn test_compress_cards() {
        let mut set = HashSet::new();
        for &(my_cards, other_cards) in &card_config() {
            let val = compress_cards(my_cards, other_cards);
            assert!(val < 30);
            assert!(!set.contains(&val));
            set.insert(val);
        }
        assert!(set.len() == 30)
    }

    #[test]
    fn test_tablebase() {
        TableBase::new();
    }
}
