use bitintr::Popcnt;
use nudge::assume;

use crate::ops::{BitIter, CardIter};
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
    pub fn count_moves(&self) -> u64 {
        let mut total = 0;
        for from in BitIter(self.my & PIECE_MASK) {
            for card in CardIter::new(self.my_cards) {
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

    pub fn is_win(&mut self) -> bool {
        self.into_iter().any(
            #[inline(always)]
            |(_, win)| win,
        )
    }

    fn next_from(&self) -> BitIter {
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
        let mut to_curr = self.to.next();
        while to_curr.is_none() {
            match self.card.next() {
                Some(val) => self.card_curr = val,
                None => {
                    self.from_curr = self.from.next()?;
                    self.card = self.game.next_card();
                    self.card_curr = self.card.next().unwrap();
                }
            }
            self.to = self.game.next_to(self.from_curr, self.card_curr);
            to_curr = self.to.next();
        }
        let to_curr = to_curr.unwrap();

        let my_king = self.game.my.wrapping_shr(25);
        let other_king = 24 - self.game.other.wrapping_shr(25);
        let king = self.from_curr == my_king;
        let m = Move {
            from: self.from_curr,
            card: self.card_curr,
            to: to_curr,
            king,
        };
        let mut win = other_king == to_curr;
        if king {
            win |= to_curr == 22
        }
        Some((self.game.step(m), win))
    }
}

impl<'a> IntoIterator for &'a Game {
    type Item = (Game, bool);

    type IntoIter = GameIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let mut from = self.next_from();
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
}
