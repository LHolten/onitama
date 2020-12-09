use std::slice::Iter;

use bitintr::Popcnt;

use crate::ops::{card_iter, BitIter};
use crate::SHIFTED;

const PIECE_MASK: u32 = (1 << 25) - 1;

pub struct Game {
    pub my: u32,
    pub other: u32,
    pub my_cards: u8,
    pub other_cards: u8,
}

pub struct Move {
    from: u32, // sparse
    to: u32,   // sparse
    card: u8,  // sparse
    king: bool,
}

impl Game {
    pub fn step(&self, m: Move) -> Self {
        let to_other = (1 << 24) >> m.to;
        let other = self.other & !to_other;

        let my_cards = 1 << m.card ^ !self.other_cards;
        let mut my = self.my ^ (1 << m.from) ^ (1 << m.to);

        if m.king {
            my = my & PIECE_MASK | m.to << 25;
        };

        Game {
            other: my,
            my: other,
            other_cards: my_cards,
            my_cards: self.other_cards,
        }
    }

    pub fn new_games(&self) -> impl Iterator<Item = (Game, bool)> + '_ {
        let handle_piece = #[inline(always)]
        move |from: u32, king: bool| {
            card_iter(self.my_cards).flat_map(
                #[inline(always)]
                move |card| {
                    let &shifted = unsafe {
                        SHIFTED
                            .get_unchecked(card as usize)
                            .get_unchecked(from as usize)
                    };
                    BitIter(shifted & !self.my).map(
                        #[inline(always)]
                        move |to| {
                            let m = Move {
                                from,
                                to,
                                card,
                                king,
                            };
                            let mut win = self.other.wrapping_shr(25) == 24 - to;
                            if king {
                                win |= to == 22
                            }
                            (self.step(m), win)
                        },
                    )
                },
            )
        };

        handle_piece(self.my.wrapping_shr(25), true).chain(
            BitIter(self.my & PIECE_MASK ^ 1 << self.my.wrapping_shr(25)).flat_map(
                #[inline(always)]
                move |from| handle_piece(from, false),
            ),
        )
    }

    // #[inline(always)]
    // pub fn new_games<F: FnMut(Game, bool)>(&self, mut func: F) {
    //     let mut handle_cards = #[inline(always)]
    //     |from: u32, king: bool| {
    //         for card in card_iter(self.my_cards) {
    //             let &shifted = unsafe {
    //                 SHIFTED
    //                     .get_unchecked(card as usize)
    //                     .get_unchecked(from as usize)
    //             };
    //             for to in BitIter(shifted & !self.my) {
    //                 let m = Move {
    //                     from,
    //                     to,
    //                     card,
    //                     king,
    //                 };
    //                 let mut win = self.other.wrapping_shr(25) == 24 - to;
    //                 if king {
    //                     win |= to == 22
    //                 }
    //                 func(self.step(m), win)
    //             }
    //         }
    //     };

    //     handle_cards(self.my.wrapping_shr(25), true);
    //     for from in BitIter(self.my & PIECE_MASK ^ 1 << self.my.wrapping_shr(25)) {
    //         handle_cards(from, false)
    //     }
    // }

    pub fn count_moves(&self) -> u64 {
        let mut total = 0;
        for from in BitIter(self.my & PIECE_MASK) {
            for card in card_iter(self.my_cards) {
                let &shifted = unsafe {
                    SHIFTED
                        .get_unchecked(card as usize)
                        .get_unchecked(from as usize)
                };
                total += (shifted & !self.my).popcnt() as u64
            }
        }
        total
    }

    pub fn is_win(&self) -> bool {
        for (_, win) in self.new_games() {
            if win {
                return true;
            }
        }
        false
    }
}
