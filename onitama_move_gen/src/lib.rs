#![feature(stmt_expr_attributes)]
#![feature(maybe_uninit_ref)]
#![feature(maybe_uninit_extra)]
#![feature(test)]
#![feature(try_trait)]
#[macro_use]
extern crate build_const;
extern crate bitintr;
extern crate nudge;
extern crate num_traits;

pub mod eval;
pub mod gen;
pub mod ops;
pub mod perft;
pub mod tablebase;

build_const!("lut");
