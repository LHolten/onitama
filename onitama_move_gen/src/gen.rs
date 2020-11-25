use std::mem::MaybeUninit;

use bitintr::{Bzhi, Pdep, Popcnt, Tzcnt};
use nudge::assume;

use crate::ops::{loop_cards, loop_moves};
use crate::SHIFTED;

const PIECE_MASK: u32 = (1 << 25) - 1;

pub struct Game {
    pub my: u32,
    pub other: u32,
    pub my_cards: u8,
    pub other_cards: u8,
    pub depth: u8,
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
            depth: self.depth + 1,
        }
    }

    #[inline(always)]
    pub fn new_games<F: FnMut(Game, bool)>(&self, mut func: F) {
        let mut handle_cards = #[inline(always)]
        |from: u32, king: bool| {
            loop_cards(
                self.my_cards,
                #[inline(always)]
                |card| {
                    let &shifted = unsafe {
                        SHIFTED
                            .get_unchecked(card as usize)
                            .get_unchecked(from as usize)
                    };
                    loop_moves(
                        shifted & !self.my,
                        #[inline(always)]
                        |to| {
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
                        },
                    )
                },
            )
        };

        handle_cards(self.my.wrapping_shr(25), true);
        loop_moves(
            self.my & PIECE_MASK ^ 1 << self.my.wrapping_shr(25),
            #[inline(always)]
            |from| handle_cards(from, false),
        );
    }
}
