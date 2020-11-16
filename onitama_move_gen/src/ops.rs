use bitintr::{Blsr, Tzcnt};

build_const!("lut");

#[inline]
pub fn shift_or(card: usize, player: usize, mut pieces: u32) -> u32 {
    assert!(card < 16 && player < 2 && pieces < 1 << 25);
    let mut result = 0;
    for _ in 0..5 {
        if pieces == 0 {
            return result;
        }
        let pos = pieces.tzcnt() as usize;
        pieces = pieces.blsr();
        result |= unsafe {
            SHIFTED
                .get_unchecked(card)
                .get_unchecked(player)
                .get_unchecked(pos)
        };
    }
    result
}
