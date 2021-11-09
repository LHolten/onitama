use std::{ffi::c_void, ops::Neg};

use ipasir_sys::{
    ipasir_add, ipasir_assume, ipasir_init, ipasir_release, ipasir_solve, ipasir_val,
};

pub struct IPASIR {
    ptr: *mut c_void,
    lit: Literal,
}

impl Default for IPASIR {
    fn default() -> Self {
        unsafe {
            Self {
                ptr: ipasir_init(),
                lit: Literal::default(),
            }
        }
    }
}

impl Drop for IPASIR {
    fn drop(&mut self) {
        unsafe { ipasir_release(self.ptr) }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Literal(i32);

impl Neg for Literal {
    type Output = Literal;

    fn neg(self) -> Self::Output {
        Literal(-self.0)
    }
}

impl IPASIR {
    pub fn new_literal(&mut self) -> Literal {
        self.lit.0 += 1;
        self.lit
    }

    pub fn assume(&mut self, literal: Literal) {
        unsafe { ipasir_assume(self.ptr, literal.0) }
    }

    pub fn add(&mut self, literal: Literal) {
        unsafe { ipasir_add(self.ptr, literal.0) }
    }

    pub fn solve(&mut self) -> Option<impl '_ + Fn(Literal) -> bool> {
        if unsafe { ipasir_solve(self.ptr) == 10 } {
            Some(|lit: Literal| unsafe { ipasir_val(self.ptr, lit.0) >= 0 })
        } else {
            None
        }
    }
}
