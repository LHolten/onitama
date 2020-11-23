use bitintr::{Pdep, Popcnt, Tzcnt};

use crate::ops::{loop_bits, loop_bits_exact};
use crate::SHIFTED;

const CARD_MASK: u32 = 0b1111 << 28;
const KING_MASK: u32 = 0b111 << 25;
const PIECES_MASK: u32 = (1 << 25) - 1;
const FIRST_MASK: u64 = u32::MAX as u64;

#[derive(Copy, Clone)]
pub struct Game {
    pub my: u32,
    pub other: u32,
}

pub struct Move {
    from: u32,
    to: u32,
    to_inv: u32,
    card: u32,
    king: bool,
}

impl Game {
    pub fn compress(self) -> u64 {
        self.my as u64 | (self.other as u64) << 32
    }

    pub fn deflate(val: u64) -> Self {
        Game {
            my: (val & FIRST_MASK) as u32,
            other: ((val & !FIRST_MASK) >> 32) as u32,
        }
    }

    pub fn step(self, m: Move) -> Self {
        let cards = m.card ^ !self.other & CARD_MASK;
        let other = self.other & !m.to_inv;

        let my = if m.king {
            let pieces = self.my ^ m.from;
            let king_pos = ((m.to - 1) & pieces).popcnt() << 25;
            cards ^ m.to ^ king_pos ^ pieces & PIECES_MASK
        } else {
            cards ^ m.from ^ m.to ^ self.my & !CARD_MASK
        };

        Game {
            my: other,
            other: my,
        }
    }

    pub fn iter<F: FnMut(Move)>(self, mut func: F) {
        let mut any_moves = false;
        let mut handle_cards = #[inline(always)]
        |from: u32, king: bool| {
            loop_bits_exact(
                2,
                self.my & CARD_MASK,
                #[inline(always)]
                |card| {
                    let &shifted = unsafe {
                        SHIFTED
                            .get_unchecked(card.tzcnt() as usize - 28)
                            .get_unchecked(from.tzcnt() as usize)
                    };
                    let shifted = shifted & !self.my;
                    loop_bits(
                        shifted,
                        #[inline(always)]
                        |to| {
                            let to_inv = (1 << 24) >> to.tzcnt();
                            func(Move {
                                from,
                                to,
                                to_inv,
                                card,
                                king,
                            });
                            any_moves = true;
                        },
                    );
                },
            );
        };

        let my_king = (1 << ((self.my & KING_MASK) >> 25)).pdep(self.my);
        handle_cards(my_king, true);
        loop_bits(
            (self.my ^ my_king) & PIECES_MASK,
            #[inline(always)]
            |from| handle_cards(from, false),
        );

        if !any_moves {
            loop_bits_exact(
                2,
                self.my & CARD_MASK,
                #[inline(always)]
                |card| {
                    func(Move {
                        from: 0,
                        to: 0,
                        to_inv: 0,
                        card,
                        king: true,
                    })
                },
            );
        }
    }
}

// pub fn finished(game: &Game) {
//     let goal = [1 << 2, 1 << 22][game.player];
//     let stone = other_pieces & kings != 0;
//     let stream = game.kings & goal != 0;
//     stone || stream;
// }
