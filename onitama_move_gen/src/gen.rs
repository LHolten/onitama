use bitintr::{Bextr, Blsfill, Blsi, Bzhi, Pdep, Popcnt, Tzcnt};

use crate::ops::{loop_n, loop_n_exact};
use crate::SHIFTED;

const CARD_MASK: u64 = 0b1111 << 28;
const KING_MASK: u64 = 0b111 << 25;
const PIECES_MASK: u64 = (1 << 25) - 1;

#[derive(Copy, Clone)]
pub struct Game(pub u64);

impl Game {
    #[inline]
    fn split(self) -> (u64, u64) {
        (self.0.bextr(0, 32), self.0.bextr(32, 32))
    }

    #[inline]
    fn combine(my: u64, other: u64) -> Self {
        Game(my | other << 32)
    }

    #[inline]
    pub fn step(self, from: u64, to: u64, card: u64, king: bool) -> Self {
        let (mut my, mut other) = self.split();

        my ^= from ^ to;
        other &= !to;
        if king {
            let king = ((to.blsfill() & my).popcnt() - 1) << 25;
            my = my & PIECES_MASK ^ king ^ card ^ (u64::MAX ^ other) & CARD_MASK;
        } else {
            my ^= card ^ (my ^ u64::MAX ^ other) & CARD_MASK;
        }

        Game::combine(other, my)
    }

    #[inline]
    pub fn iter<F: FnMut(u64, u64, u64, bool)>(self, player: usize, mut func: F) {
        let mut any_moves = false;
        let my_king = (1 << ((self.0 & KING_MASK) >> 25)).pdep(self.0);

        let mut handle_cards = |from: u64, king: bool| {
            loop_n_exact(2, self.0 & CARD_MASK, |card| {
                let &shifted = unsafe {
                    SHIFTED
                        .get_unchecked(player)
                        .get_unchecked((card as u32).tzcnt() as usize - 28)
                        .get_unchecked(from.tzcnt() as usize)
                };
                let shifted = shifted as u64 & !self.0;
                loop_n(4, shifted, |to| {
                    func(from, to, card, king);
                    any_moves = true;
                });
            });
        };

        handle_cards(my_king, true);
        loop_n(4, (self.0 ^ my_king) & PIECES_MASK, |from| {
            handle_cards(from, false)
        });

        if !any_moves {
            loop_n_exact(2, self.0 & CARD_MASK, |card| func(0, 0, card, false));
        }
    }
}

// pub fn finished(game: &Game) {
//     let goal = [1 << 2, 1 << 22][game.player];
//     let stone = other_pieces & kings != 0;
//     let stream = game.kings & goal != 0;
//     stone || stream;
// }
