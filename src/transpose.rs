use std::{
    alloc::{alloc_zeroed, Layout},
    ops::{Index, IndexMut},
};

use onitama_move_gen::{eval::Eval, gen::Game};

pub struct Transpose(Box<[Eval; 1 << 16]>);

impl Transpose {
    pub fn new() -> Self {
        let val = unsafe {
            let layout = Layout::new::<[Eval; 1 << 16]>();
            Box::from_raw(alloc_zeroed(layout) as *mut [Eval; 1 << 16])
        };
        Self(val)
    }
}

impl Index<Game> for Transpose {
    type Output = Eval;

    fn index(&self, index: Game) -> &Self::Output {
        &self.0[index.hash() as usize]
    }
}

impl IndexMut<Game> for Transpose {
    fn index_mut(&mut self, index: Game) -> &mut Self::Output {
        &mut self.0[index.hash() as usize]
    }
}
