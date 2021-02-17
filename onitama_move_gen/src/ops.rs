use bitintr::{Blsi, Blsr, Tzcnt};
use nudge::assume;

pub struct BitIter(pub u32);

impl Iterator for BitIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 != 0 {
            unsafe { assume(self.0.blsi() == 1 << self.0.tzcnt()) }
            let val = self.0.tzcnt();
            self.0 = self.0.blsr();
            Some(val)
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

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.card1;
        self.card1 = self.card2;
        self.card2 = None;
        val
    }
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
