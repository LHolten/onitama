use bitintr::{Blsr, Tzcnt};
use nudge::assume;

pub struct BitIter(pub u32);

impl Iterator for BitIter {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.peek() {
            Some(val) => {
                self.0 = self.0.blsr();
                Some(val)
            }
            None => None,
        }
    }
}

impl BitIter {
    #[inline]
    pub fn peek(&self) -> Option<<Self as Iterator>::Item> {
        if self.0 != 0 {
            Some(self.0.tzcnt())
        } else {
            None
        }
    }
}

pub struct CardIter {
    card1: Option<u32>,
    card2: Option<u32>,
}

impl CardIter {
    #[inline]
    pub fn new(mut value: u32) -> Self {
        let card1 = Some(value.tzcnt());
        unsafe { assume(value != 0) }
        value = value.blsr();
        let card2 = Some(value.tzcnt());
        Self { card1, card2 }
    }
}

impl Iterator for CardIter {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let val = self.card1;
        self.card1 = self.card2;
        self.card2 = None;
        val
    }
}

pub fn cards_or(table: &[[u32; 25]; 16], cards: impl Iterator<Item = u32>, from: u32) -> u32 {
    let mut total = 0;
    for card in cards {
        total |= unsafe {
            table
                .get_unchecked(card as usize)
                .get_unchecked(from as usize)
        };
    }
    total
}

// #[inline]
// pub fn shift_or(card: &[u32; 25], pieces: u32) -> u32 {
//     let mut result = 0;
//     loop_moves(pieces, |pieces| {
//         let pos = pieces.tzcnt() as usize;
//         result |= unsafe { card.get_unchecked(pos) };
//     });
//     result
// }
