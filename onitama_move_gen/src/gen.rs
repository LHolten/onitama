use bitintr::Popcnt;

use crate::ops::{card_array, BitIter};
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

    #[inline(always)]
    pub fn new_games<F: FnMut(Game, bool)>(&self, mut func: F) {
        let mut handle_cards = #[inline(always)]
        |from: u32, king: bool| {
            for &card in &card_array(self.my_cards) {
                let &shifted = unsafe {
                    SHIFTED
                        .get_unchecked(card as usize)
                        .get_unchecked(from as usize)
                };
                for to in BitIter(shifted & !self.my) {
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
                    func(self.step(m), win)
                }
            }
        };

        handle_cards(self.my.wrapping_shr(25), true);
        for from in BitIter(self.my & PIECE_MASK ^ 1 << self.my.wrapping_shr(25)) {
            handle_cards(from, false)
        }
    }

    pub fn count_moves(&self) -> u64 {
        let mut total = 0;
        for from in BitIter(self.my & PIECE_MASK) {
            for &card in &card_array(self.my_cards) {
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

    // pub fn reach(&self) -> u32 {
    //     let mut result = 0;
    //     loop_moves(self.my & PIECE_MASK, |from| {
    //         loop_cards(self.my_cards, |card| {
    //             result |= unsafe {
    //                 SHIFTED
    //                     .get_unchecked(card as usize)
    //                     .get_unchecked(from as usize)
    //             };
    //         })
    //     });
    //     result
    // }

    // pub fn is_win(&self) -> bool {
    //     let reach = self.reach();
    //     let other_king = (1 << 24) >> self.other.wrapping_shr(25);
    // }
}
