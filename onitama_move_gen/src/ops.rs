use bitintr::{Blsi, Blsr, Tzcnt};
use nudge::assume;

pub struct BitIter(pub u32);

impl Iterator for BitIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 != 0 {
            unsafe { assume(self.0.blsi() == 1 << self.0.tzcnt()) }
            let res = self.0.tzcnt();
            self.0 = self.0.blsr();
            Some(res)
        } else {
            None
        }
    }
}

pub fn card_iter(mut value: u8) -> impl Iterator<Item = u8> {
    unsafe { assume(value.blsi() == 1 << value.tzcnt()) }
    let card1 = value.tzcnt();
    value = value.blsr();
    unsafe { assume(value.blsi() == 1 << value.tzcnt()) }
    let card2 = value.tzcnt();
    (0..2).map(move |i| match i {
        0 => card1,
        1 => card2,
        _ => unreachable!(),
    })
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
