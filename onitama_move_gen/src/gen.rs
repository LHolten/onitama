use std::mem::MaybeUninit;

use bitintr::Tzcnt;
use nudge::assume;

use crate::ops::{loop_bits, loop_bits_exact};
use crate::SHIFTED;

pub struct Player {
    pub pieces: u32,
    pub king: u32,
    pub cards: u8,
}

pub struct Game {
    pub my: Player,
    pub other: Player,
    pub depth: u8,
}

pub struct Move {
    from: u8, // sparse
    to: u8,   // sparse
    card: u8,
    king: bool,
}

impl Game {
    pub fn step(&self, m: Move) -> Self {
        let my_cards = m.card ^ !self.other.cards;
        let other_pieces = self.other.pieces & !(1 << (24 - m.to));

        let mut my_pieces = self.my.pieces;
        let mut my_king = self.my.king;
        if m.king {
            my_king = 1 << m.to;
        } else {
            my_pieces ^= (1 << m.from) ^ (1 << m.to)
        };

        Game {
            my: Player {
                pieces: other_pieces,
                king: self.other.king,
                cards: self.other.cards,
            },
            other: Player {
                pieces: my_pieces,
                king: my_king,
                cards: my_cards,
            },
            depth: self.depth + 1,
        }
    }

    #[inline(always)]
    pub fn new_games(&self, stack: &mut [MaybeUninit<Game>], height: &mut usize) {
        let mut handle_cards = #[inline(always)]
        |from: u32, king: bool| {
            loop_bits_exact(
                2,
                self.my.cards,
                #[inline(always)]
                |card| {
                    let &shifted = unsafe {
                        SHIFTED
                            .get_unchecked(card.tzcnt() as usize)
                            .get_unchecked(from.tzcnt() as usize)
                    };
                    let shifted = shifted & !(self.my.pieces | self.my.king);
                    loop_bits(
                        shifted,
                        #[inline(always)]
                        |to| {
                            let m = Move {
                                from: from.tzcnt() as u8,
                                to: to.tzcnt() as u8,
                                card,
                                king,
                            };
                            unsafe { assume(*height < stack.len()) }
                            stack[*height] = MaybeUninit::new(self.step(m));
                            *height += 1;
                        },
                    );
                },
            );
        };

        handle_cards(self.my.king, true);
        loop_bits(
            self.my.pieces,
            #[inline(always)]
            |from| handle_cards(from, false),
        );
    }
}

// pub fn finished(game: &Game) {
//     let goal = [1 << 2, 1 << 22][game.player];
//     let stone = other_pieces & kings != 0;
//     let stream = game.kings & goal != 0;
//     stone || stream;
// }
