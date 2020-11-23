use bitintr::{Blsi, Blsr, Tzcnt};
use nudge::assume;

#[inline(always)]
pub fn loop_bits<F: FnMut(u32)>(mut value: u32, mut func: F) {
    while value != 0 {
        unsafe { assume(value.blsi() == 1 << value.tzcnt()) }
        unsafe { assume(value.blsr() == value ^ value.blsi()) }
        unsafe { assume(value.tzcnt() == value.blsi().tzcnt()) }
        func(value.blsi());
        value = value.blsr();
    }
}

#[inline(always)]
pub fn loop_bits_exact<F: FnMut(u32)>(n: usize, mut value: u32, mut func: F) {
    for i in 0..n {
        if i != n - 1 {
            unsafe { assume(value.blsi() == 1 << value.tzcnt()) }
            unsafe { assume(value.blsr() == value ^ value.blsi()) }
            unsafe { assume(value.tzcnt() == value.blsi().tzcnt()) }
            func(value.blsi());
            value = value.blsr();
        } else {
            func(value);
        }
    }
}

#[inline]
pub fn shift_or(card: &[u32; 25], pieces: u32) -> u32 {
    assert!(pieces < 1 << 25);
    let mut result = 0;
    loop_bits(pieces, |pieces| {
        let pos = pieces.tzcnt() as usize;
        result |= unsafe { card.get_unchecked(pos) };
    });
    result
}
