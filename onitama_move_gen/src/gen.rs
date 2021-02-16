use bitintr::{Andn, Popcnt, Tzcnt};
use nudge::assume;

use crate::ops::{BitIter, CardIter};
use crate::{SHIFTED, SHIFTED_L, SHIFTED_R, SHIFTED_U};

pub const PIECE_MASK: u32 = (1 << 25) - 1;

#[derive(Clone, Copy)]
pub struct Game {
    pub my: u32,
    pub other: u32,
    pub my_cards: u8,
    pub other_cards: u8,
}

impl Game {
    #[inline(always)]
    pub fn count_moves(&self) -> u64 {
        let mut total = 0;
        for from in self.next_my() {
            let both = unsafe {
                let mut cards = self.next_my_card();
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
        total
    }

    pub fn is_win(&self) -> bool {
        for from in self.next_my() {
            let both = unsafe {
                let mut cards = self.next_my_card();
                SHIFTED
                    .get_unchecked(cards.next().unwrap() as usize)
                    .get_unchecked(from as usize)
                    | SHIFTED
                        .get_unchecked(cards.next().unwrap() as usize)
                        .get_unchecked(from as usize)
            };
            let other_king = 1 << 24 >> self.other.wrapping_shr(25);
            if both & other_king != 0 {
                return true;
            }
            if from == self.my.wrapping_shr(25) && both & (1 << 22) != 0 {
                return true;
            }
        }
        false
    }

    pub fn is_loss(&self) -> bool {
        self.other.wrapping_shr(25) == 22 || self.my & 1 << self.my.wrapping_shr(25) == 0
    }

    pub fn is_other_loss(&self) -> bool {
        self.my.wrapping_shr(25) == 22 || self.other & 1 << self.other.wrapping_shr(25) == 0
    }

    fn next_my(&self) -> BitIter {
        unsafe { assume(self.my & PIECE_MASK != 0) }
        BitIter(self.my & PIECE_MASK)
    }

    pub fn next_other(&self) -> BitIter {
        unsafe { assume(self.other & PIECE_MASK != 0) }
        BitIter(self.other & PIECE_MASK)
    }

    fn next_my_card(&self) -> CardIter {
        CardIter::new(self.my_cards)
    }

    fn next_other_card(&self) -> CardIter {
        CardIter::new(self.other_cards)
    }

    fn next_to(&self, from: u32, card: u8) -> BitIter {
        let &shifted = unsafe {
            SHIFTED
                .get_unchecked(card as usize)
                .get_unchecked(from as usize)
        };
        BitIter(self.my.andn(shifted))
    }

    fn next_from(&self, to: u32, card: u8) -> BitIter {
        let &shifted = unsafe {
            SHIFTED_R
                .get_unchecked(card as usize)
                .get_unchecked(to as usize)
        };
        let mut my_rev = self.my.reverse_bits() >> 7;
        if to == self.other.wrapping_shr(25) {
            my_rev |= 1 << 22
        }
        BitIter(my_rev.andn(self.other.andn(shifted)))
    }

    pub fn forward(&self) -> GameIter {
        let mut from = self.next_my();
        let from_curr = from.next().unwrap();
        let mut card = self.next_my_card();
        let card_curr = card.next().unwrap();
        let to = self.next_to(from_curr, card_curr);
        GameIter {
            game: self,
            from,
            from_curr,
            card,
            card_curr,
            to,
        }
    }

    pub fn backward(&self) -> GameBackIter {
        let table_card = (u8::MAX ^ self.my_cards ^ self.other_cards).tzcnt();
        let mut to = self.next_other();
        let to_curr = to.next().unwrap();
        let mut card = self.next_other_card();
        let card_curr = card.next().unwrap();
        let from = self.next_from(to_curr, table_card);
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

pub struct GameIter<'a> {
    game: &'a Game,
    from: BitIter,
    from_curr: u32,
    card: CardIter,
    card_curr: u8,
    to: BitIter,
}

impl Iterator for GameIter<'_> {
    type Item = Game;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let mut to_new = self.to.next();
        while to_new.is_none() {
            let mut card_new = self.card.next();
            if card_new.is_none() {
                self.from_curr = self.from.next()?;
                self.card = self.game.next_my_card();
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

        let my_cards = 1 << self.card_curr ^ !self.game.other_cards;
        let mut my = self.game.my ^ (1 << self.from_curr) ^ (1 << to_curr);

        if self.from_curr == my_king {
            my = my & PIECE_MASK | to_curr << 25;
        };

        let new_game = Game {
            other: my,
            my: other,
            other_cards: my_cards,
            my_cards: self.game.other_cards,
        };

        Some(new_game)
    }
}

pub struct GameBackIter<'a> {
    game: &'a Game,
    to: BitIter,
    to_curr: u32,
    card: CardIter,
    card_curr: u8,
    from: BitIter,
}

impl Iterator for GameBackIter<'_> {
    type Item = (Game, u32); // (no)take

    fn next(&mut self) -> Option<Self::Item> {
        let table_card = (u8::MAX ^ self.game.my_cards ^ self.game.other_cards).tzcnt();

        let mut from_new = self.from.next();
        while from_new.is_none() {
            let mut card_new = self.card.next();
            if card_new.is_none() {
                self.to_curr = self.to.next()?;
                self.card = self.game.next_other_card();
                card_new = self.card.next();
            }
            self.card_curr = card_new.unwrap();
            self.from = self.game.next_from(self.to_curr, table_card);
            from_new = self.from.next();
        }
        let from_curr = from_new.unwrap();

        let other_king = self.game.other.wrapping_shr(25);

        let other_cards = 1 << self.card_curr ^ !self.game.my_cards;
        let mut other = self.game.other ^ (1 << self.to_curr) ^ (1 << from_curr);

        if self.to_curr == other_king {
            other = other & PIECE_MASK | from_curr << 25;
        };

        Some((
            Game {
                my: other,
                other: self.game.my,
                my_cards: other_cards,
                other_cards: self.game.my_cards,
            },
            (1 << 24) >> self.to_curr,
        ))
    }
}
