use bitintr::{Blsi, Blsr, Tzcnt};
use nudge::assume;

#[inline(always)]
pub fn loop_moves<F: FnMut(u32)>(mut value: u32, mut func: F) {
    while value != 0 {
        unsafe { assume(value.blsi() == 1 << value.tzcnt()) }
        func(value.tzcnt());
        value = value.blsr();
    }
}

#[inline(always)]
pub fn loop_cards<F: FnMut(u32)>(mut value: u32, mut func: F) {
    unsafe { assume(value.blsi() == 1u32.wrapping_shl(value.tzcnt())) }
    func(value.tzcnt());
    value = value.blsr();
    func(value.tzcnt());
}

#[inline]
pub fn shift_or(card: &[u32; 25], pieces: u32) -> u32 {
    assert!(pieces < 1 << 25);
    let mut result = 0;
    loop_moves(pieces, |pieces| {
        let pos = pieces.tzcnt() as usize;
        result |= unsafe { card.get_unchecked(pos) };
    });
    result
}
