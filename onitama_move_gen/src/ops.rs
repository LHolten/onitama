use bitintr::{Blsi, Pdep, Popcnt, Tzcnt};

build_const!("lut");

#[inline]
pub fn shift_or(card: usize, player: usize, pieces: u32) -> u32 {
    let mut result = 0;
    for to in BitIter(pieces) {
        let pos = to.tzcnt() as usize;
        result |= SHIFTED[card][player][pos]
    }
    result
}

#[inline]
pub fn shift_or_pdep(card: usize, player: usize, pieces: u32) -> u32 {
    let mut result = 0;
    for i in 0..pieces.popcnt() {
        let to = (1 << i).pdep(pieces);
        let pos = to.tzcnt() as usize;
        result |= SHIFTED[card][player][pos]
    }
    result
}

#[inline]
pub fn shift_or_simd(card: usize, player: usize, pieces: u32) -> u32 {
    let mut result = 0;
    for i in 0..pieces.popcnt() {
        let to = (1 << i).pdep(pieces);
        let pos = to.tzcnt() as usize;
        result |= SHIFTED[card][player][pos]
    }
    result
}

struct BitIter(u32);

impl Iterator for BitIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            None
        } else {
            let pos = self.0.blsi();
            self.0 ^= pos;
            Some(pos)
        }
    }
}
