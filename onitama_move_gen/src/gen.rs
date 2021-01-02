use bitintr::{Andn, Popcnt, Tzcnt};
use nudge::assume;

use crate::ops::{BitIter, CardIter};
use crate::{SHIFTED, SHIFTED_L, SHIFTED_R, SHIFTED_U};

pub const PIECE_MASK: u32 = (1 << 25) - 1;

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
        for from in self.next_piece() {
            let both = unsafe {
                let mut cards = self.next_card();
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
        for from in self.next_piece() {
            let both = unsafe {
                let mut cards = self.next_card();
                SHIFTED
                    .get_unchecked(cards.next().unwrap() as usize)
                    .get_unchecked(from as usize)
                    | SHIFTED
                        .get_unchecked(cards.next().unwrap() as usize)
                        .get_unchecked(from as usize)
            };
            let other_king = (1 << 24) >> self.other.wrapping_shr(25);
            if both & other_king != 0 {
                return true;
            }
            if from == self.my.wrapping_shr(25) && both & (1 << 22) != 0 {
                return true;
            }
        }
        false
    }

    fn next_piece(&self) -> BitIter {
        unsafe { assume(self.my & PIECE_MASK != 0) }
        BitIter(self.my & PIECE_MASK)
    }

    fn next_card(&self) -> CardIter {
        CardIter::new(self.my_cards)
    }

    fn next_to(&self, from: u32, card: u8) -> BitIter {
        let &shifted = unsafe {
            SHIFTED
                .get_unchecked(card as usize)
                .get_unchecked(from as usize)
        };
        BitIter(shifted & !self.my)
    }

    fn next_from(&self, to: u32, card: u8) -> BitIter {
        let &shifted = unsafe {
            SHIFTED_R
                .get_unchecked(card as usize)
                .get_unchecked(to as usize)
        };
        BitIter(shifted & !self.my)
    }

    pub fn forward(&self) -> GameIter {
        let mut from = self.next_piece();
        let from_curr = from.next().unwrap();
        let mut card = self.next_card();
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

    pub fn backwards(&self) -> GameBackIter {
        let table_card = (u8::MAX ^ self.my_cards ^ self.other_cards).tzcnt();
        let mut to = self.next_piece();
        let to_curr = to.next().unwrap();
        let mut from = self.next_from(to_curr, table_card);
        let from_curr = from.next().unwrap();
        let card = self.next_card();
        GameBackIter {
            game: self,
            to,
            to_curr,
            from,
            from_curr,
            card,
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
    type Item = (Game, bool);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let mut to_new = self.to.next();
        while to_new.is_none() {
            let mut card_new = self.card.next();
            if card_new.is_none() {
                self.from_curr = self.from.next()?;
                self.card = self.game.next_card();
                card_new = self.card.next();
            }
            self.card_curr = card_new.unwrap();
            self.to = self.game.next_to(self.from_curr, self.card_curr);
            to_new = self.to.next();
        }
        let to_curr = to_new.unwrap();

        let my_king = self.game.my.wrapping_shr(25);
        let other_king = 24 - self.game.other.wrapping_shr(25);

        let to_other = (1 << 24) >> to_curr;
        let other = to_other.andn(self.game.other);

        let my_cards = 1 << self.card_curr ^ !self.game.other_cards;
        let mut my = self.game.my ^ (1 << self.from_curr) ^ (1 << to_curr);

        let mut win = other_king == to_curr;
        if self.from_curr == my_king {
            my = my & PIECE_MASK | to_curr << 25;
            win |= to_curr == 22
        };

        let new_game = Game {
            other: my,
            my: other,
            other_cards: my_cards,
            my_cards: self.game.other_cards,
        };

        Some((new_game, win))
    }
}

pub struct GameBackIter<'a> {
    game: &'a Game,
    to: BitIter,
    to_curr: u32,
    from: BitIter,
    from_curr: u32,
    card: CardIter,
}

impl Iterator for GameBackIter<'_> {
    type Item = (Game, Game); // (no)take

    fn next(&mut self) -> Option<Self::Item> {
        let table_card = (u8::MAX ^ self.game.my_cards ^ self.game.other_cards).tzcnt();

        let mut card_new = self.card.next();
        if card_new.is_none() {
            let mut from_new = self.from.next();
            while from_new.is_none() {
                self.to_curr = self.to.next()?;
                self.from = self.game.next_from(self.to_curr, table_card);
                from_new = self.from.next();
            }
            self.from_curr = from_new.unwrap();
            self.card = self.game.next_card();
            card_new = self.card.next();
        };
        let card_curr = card_new.unwrap();

        let my_king = self.game.my.wrapping_shr(25);

        let to_other = (1 << 24) >> self.to_curr;
        let other = to_other | self.game.other;

        let my_cards = 1 << card_curr ^ !self.game.other_cards;
        let mut my = self.game.my ^ (1 << self.to_curr) ^ (1 << self.from_curr);

        if self.to_curr == my_king {
            my = my & PIECE_MASK | self.from_curr << 25;
        };

        Some((
            Game {
                my: self.game.other,
                other: my,
                my_cards: self.game.other_cards,
                other_cards: my_cards,
            },
            Game {
                my: other,
                other: my,
                my_cards: self.game.other_cards,
                other_cards: my_cards,
            },
        ))
    }
}
