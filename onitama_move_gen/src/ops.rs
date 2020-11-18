use bitintr::{Blsi, Blsr, Tzcnt};
use nudge::assume;
use num_traits::Num;

pub trait MyNum: Num + Blsr + Blsi + Copy {}
impl<T: Num + Blsr + Blsi + Copy> MyNum for T {}

#[inline]
pub fn loop_n<T: MyNum, F: FnMut(T)>(n: usize, mut value: T, mut func: F) {
    for i in 0..n {
        if value.is_zero() {
            break;
        }
        if i != n - 1 {
            func(value.blsi());
            value = value.blsr();
        } else {
            func(value);
        }
    }
}

#[inline]
pub fn loop_n_exact<T: MyNum, F: FnMut(T)>(n: usize, mut value: T, mut func: F) {
    for i in 0..n {
        if i != n - 1 {
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
    loop_n(5, pieces, |pieces| {
        let pos = pieces.tzcnt() as usize;
        result |= unsafe { card.get_unchecked(pos) };
    });
    result
}
