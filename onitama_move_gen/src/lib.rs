#![feature(stmt_expr_attributes)]
#![feature(maybe_uninit_ref)]
#![feature(maybe_uninit_extra)]
#![feature(test)]
#[macro_use]
extern crate build_const;
extern crate bitintr;
extern crate nudge;
extern crate num_traits;

pub mod gen;
pub mod ops;
pub mod perft;

build_const!("lut");
