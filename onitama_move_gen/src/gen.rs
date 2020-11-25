use std::mem::MaybeUninit;

use bitintr::{Bzhi, Pdep, Popcnt, Tzcnt};
use nudge::assume;

use crate::ops::{loop_cards, loop_moves};
use crate::SHIFTED;

const KING_MASK: u32 = 0b111 << 25;
const PIECE_MASK: u32 = (1 << 25) - 1;
const CARD_MASK: u32 = 0b1111 << 28;

pub struct Game {
    pub my: u32,
    pub other: u32,
    pub depth: u8,
}

pub struct Move {
    from: u32, // sparse
    to: u32,   // sparse
    card: u32, // sparse
    king: bool,
}

impl Game {
    pub fn step(&self, m: Move, other_king: u32) -> Self {
        let my_cards = 1u32.wrapping_shl(m.card) ^ !self.other;
        let mut other = self.other;
        if self.other & (1 << 24) >> m.to != 0 {
            other ^= (1 << 24) >> m.to;
            other = (other & (other_king - 1)).popcnt() << 25 | other & !KING_MASK;
        }

        let my_pieces = self.my ^ (1 << m.from) ^ (1 << m.to);
        let mut my_king = my_pieces;
        if m.king {
            my_king = my_pieces.bzhi(m.from).popcnt() << 25;
        };

        Game {
            other: my_king & KING_MASK | my_pieces & PIECE_MASK | my_cards & CARD_MASK,
            my: other,
            depth: self.depth + 1,
        }
    }

    #[inline(always)]
    pub fn new_games<F: FnMut(Game, bool)>(&self, mut func: F) {
        let other_king = (1 << ((self.other & KING_MASK) >> 25)).pdep(self.other);

        let mut handle_cards = #[inline(always)]
        |from: u32, king: bool| {
            loop_cards(
                self.my & CARD_MASK,
                #[inline(always)]
                |card| {
                    let &shifted = unsafe {
                        SHIFTED
                            .get_unchecked(card as usize - 28)
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
                            let mut win = other_king == (1 << 24) >> to;
                            if king {
                                win |= to == 22
                            }
                            func(self.step(m, other_king), win)
                        },
                    )
                },
            )
        };

        let my_king = (1 << ((self.my & KING_MASK) >> 25)).pdep(self.my);
        unsafe { assume(my_king == 1 << my_king.tzcnt()) }
        handle_cards(my_king.tzcnt(), true);
        loop_moves(
            self.my & PIECE_MASK ^ my_king,
            #[inline(always)]
            |from| handle_cards(from, false),
        );
    }
}
