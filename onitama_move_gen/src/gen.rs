use std::fmt::Debug;
use std::iter::once;

use bitintr::{Andn, Popcnt};
use nudge::assume;

use crate::ops::{cards_or, BitIter, CardIter};
use crate::{SHIFTED, SHIFTED_L, SHIFTED_R, SHIFTED_U};

pub const PIECE_MASK: u32 = (1 << 25) - 1;
pub const TEMPLE: u32 = 22;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Game {
    pub my: u32,
    pub other: u32,
    pub cards: u32,
    pub table: u32,
}

impl Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "\nx: {}, o: {}",
            self.my.wrapping_shr(25),
            self.other.wrapping_shr(25)
        ))?;
        for i in 0..5 {
            f.write_str("\n")?;
            for j in 0..5 {
                let pos = i * 5 + j;
                if self.my & 1 << pos != 0 {
                    f.write_str("x")?;
                } else if self.other & 1 << 24 >> pos != 0 {
                    f.write_str("o")?;
                } else {
                    f.write_str(".")?;
                }
            }
        }
        Ok(())
    }
}

impl Game {
    #[inline(always)]
    pub fn count_moves(&self) -> usize {
        let mut total = 0;
        for from in self.piece_iter::<My>() {
            let mut cards = self.card_iter::<My>();
            let both = unsafe {
                SHIFTED_L
                    .get_unchecked(cards.next().unwrap() as usize)
                    .get_unchecked(from as usize)
                    | SHIFTED_U
                        .get_unchecked(cards.next().unwrap() as usize)
                        .get_unchecked(from as usize)
            };
            let my = self.my as u64 | (self.my as u64) << 32;
            total += my.andn(both).popcnt();
        }
        total as usize
    }

    #[inline]
    pub fn is_win(&self) -> bool {
        for from in self.piece_iter::<My>() {
            let cards = self.card_iter::<My>();
            let both = cards_or(&SHIFTED, cards, from);
            let other_king = 1 << 24 >> self.king::<Other>();

            if both & other_king != 0 {
                return true;
            }
            if from == self.king::<My>() && both & (1 << TEMPLE) != 0 {
                return true;
            }
        }
        false
    }

    #[inline]
    pub fn count_pieces(&self) -> usize {
        (self.my & PIECE_MASK).popcnt() as usize
    }

    #[inline]
    pub fn is_loss(&self) -> bool {
        self.other.wrapping_shr(25) == 22 || self.my & 1 << self.my.wrapping_shr(25) == 0
    }

    #[inline]
    pub fn is_other_loss(&self) -> bool {
        self.my.wrapping_shr(25) == 22 || self.other & 1 << self.other.wrapping_shr(25) == 0
    }

    #[inline]
    pub fn king<P: Player>(&self) -> u32 {
        P::my_or_other(self.my, self.other).wrapping_shr(25)
    }

    #[inline]
    pub fn piece_iter<P: Player>(&self) -> BitIter {
        let pieces = P::my_or_other(self.my, self.other) & PIECE_MASK;
        unsafe { assume(pieces != 0) }
        BitIter(pieces)
    }

    #[inline]
    pub fn card_iter<P: Player>(&self) -> CardIter {
        CardIter::new(P::my_or_other(self.cards, self.cards.wrapping_shr(16)))
    }

    #[inline]
    fn next_to(&self, from: u32, card: u32) -> BitIter {
        let shifted = cards_or(&SHIFTED, once(card), from);
        BitIter(self.my.andn(shifted))
    }

    #[inline]
    fn next_from(&self, to: u32, card: u32) -> BitIter {
        let shifted = cards_or(&SHIFTED_R, once(card), to);
        let my_rev = self.my.reverse_bits() >> 7;
        BitIter((my_rev | self.other).andn(shifted))
    }

    #[inline]
    pub fn forward(&self) -> GameIter {
        let mut from = self.piece_iter::<My>();
        let from_curr = from.next().unwrap();
        let mut card = self.card_iter::<My>();
        let card_curr = card.next().unwrap();
        let to = self.next_to(from_curr, card_curr);
        GameIter {
            game: *self,
            from,
            from_curr,
            card,
            card_curr,
            to,
        }
    }

    #[inline]
    pub fn backward(&self) -> GameBackIter {
        let mut to = self.piece_iter::<Other>();
        let to_curr = to.next().unwrap();
        let mut card = self.card_iter::<Other>();
        let card_curr = card.next().unwrap();
        let from = self.next_from(to_curr, self.table);
        GameBackIter {
            game: self,
            to,
            to_curr,
            card,
            card_curr,
            from,
        }
    }
}

pub struct GameIter {
    game: Game,
    from: BitIter,
    from_curr: u32,
    card: CardIter,
    card_curr: u32,
    to: BitIter,
}

impl Iterator for GameIter {
    type Item = Game;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut to_new = self.to.next();
        while to_new.is_none() {
            let mut card_new = self.card.next();
            if card_new.is_none() {
                self.from_curr = self.from.next()?;
                self.card = self.game.card_iter::<My>();
                card_new = self.card.next();
            }
            self.card_curr = card_new.unwrap();
            self.to = self.game.next_to(self.from_curr, self.card_curr);
            to_new = self.to.next();
        }
        let to_curr = to_new.unwrap();

        let my_king = self.game.my.wrapping_shr(25);

        let to_other = 1 << 24 >> to_curr;
        let other = to_other.andn(self.game.other);

        let my_cards = self.game.cards ^ 1 << self.card_curr ^ 1 << self.game.table;
        let cards = my_cards.wrapping_shl(16) | my_cards.wrapping_shr(16);

        let mut my = self.game.my ^ (1 << self.from_curr) ^ (1 << to_curr);

        if self.from_curr == my_king {
            my = my & PIECE_MASK | to_curr << 25;
        };

        let new_game = Game {
            other: my,
            my: other,
            cards,
            table: self.card_curr,
        };
        Some(new_game)
    }
}

impl ExactSizeIterator for GameIter {
    fn len(&self) -> usize {
        self.game.count_moves()
    }
}

pub struct GameBackIter<'a> {
    game: &'a Game,
    to: BitIter,
    to_curr: u32,
    card: CardIter,
    card_curr: u32,
    from: BitIter,
}

impl Iterator for GameBackIter<'_> {
    type Item = (Game, u32); // (no)take

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut from_new = self.from.next();
        while from_new.is_none() {
            let mut card_new = self.card.next();
            if card_new.is_none() {
                self.to_curr = self.to.next()?;
                self.card = self.game.card_iter::<Other>();
                card_new = self.card.next();
            }
            self.card_curr = card_new.unwrap();
            self.from = self.game.next_from(self.to_curr, self.game.table);
            from_new = self.from.next();
        }
        let from_curr = from_new.unwrap();

        let cards = self.game.cards.wrapping_shl(16) | self.game.cards.wrapping_shr(16);
        let cards = cards ^ 1 << self.card_curr ^ 1 << self.game.table;
        let mut other = self.game.other ^ (1 << self.to_curr) ^ (1 << from_curr);

        if self.to_curr == self.game.king::<Other>() {
            other = other & PIECE_MASK | from_curr << 25;
        };

        let prev_game = Game {
            my: other,
            other: self.game.my,
            cards,
            table: self.card_curr,
        };
        Some((prev_game, (1 << 24) >> self.to_curr))
    }
}

pub struct My;
pub struct Other;

pub trait Player {
    fn my_or_other<T>(my: T, other: T) -> T;
}

impl Player for My {
    fn my_or_other<T>(my: T, _other: T) -> T {
        my
    }
}

impl Player for Other {
    fn my_or_other<T>(_my: T, other: T) -> T {
        other
    }
}
