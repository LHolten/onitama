use std::ops::{BitOr, BitXor};

use bigint::U256;
use seq_macro::seq;

use crate::sdd::Sdd;

include!(concat!(env!("OUT_DIR"), concat!("/", "constants.rs")));

#[derive(Clone, Copy)]
pub struct Action {
    from: (usize, fn(U256) -> U256),
    to: (usize, fn(U256) -> U256),
}

seq!(N in 0..80 {
    pub const PLAYER0: &[Action] = &[
        #(Action {
            from: (FROM_TO[N].0, transform_from::<0, {FROM_TO[N].1}>),
            to: (FROM_TO[N].2, transform_to::<0, {FROM_TO[N].3}>),
        },)*
    ];
});

seq!(N in 0..80 {
    pub const PLAYER1: &[Action] = &[
        #(Action {
            from: (FROM_TO[N].0, transform_from::<1, {FROM_TO[N].1}>),
            to: (FROM_TO[N].2, transform_to::<1, {FROM_TO[N].3}>),
        },)*
    ];
});

//assume pos < 5 and val < 3
fn mask(pos: usize) -> (U256, usize) {
    let mut mask = U256::one();
    for i in 0..5 {
        let step = 3usize.pow(i as u32);
        if i != pos {
            mask = mask | mask << step | mask << (step * 2)
        }
    }
    (mask, 3usize.pow(pos as u32))
}

fn transform_from<const PLAYER: usize, const POS: usize>(bdd: U256) -> U256 {
    let (mask, offset) = mask(POS);
    let bdd_mask = mask << (offset * 2); // empty space
    (bdd & bdd_mask) >> (offset * 2) << (offset * PLAYER)
}

fn transform_to<const PLAYER: usize, const POS: usize>(bdd: U256) -> U256 {
    let (mask, offset) = mask(POS);
    let bdd_mask = mask << (offset * PLAYER);
    let unshifted = (bdd & bdd_mask) >> (offset * PLAYER);
    unshifted << ((1 - PLAYER) * offset) | unshifted << (offset * 2)
}

impl Sdd {
    pub fn undo(&mut self, action: &Action, state: usize) -> usize {
        let state = self.transform(0, state, action.from.1, action.from.0);
        self.transform(0, state, action.to.1, action.to.0)
    }

    pub fn expand_wins(&mut self, wins: usize) -> usize {
        let loss_draw = {
            let neg_wins = self.apply(0, [wins, 1], BitXor::bitxor);
            // from the perspective of player 0
            let mut total = 0;
            for action in PLAYER1 {
                let prev = self.undo(action, neg_wins);
                total = self.apply(0, [total, prev], BitOr::bitor);
            }
            dbg!(&self);
            total
        };

        let neg_loss_draw = self.apply(0, [loss_draw, 1], BitXor::bitxor);
        // from the perspective of player 0
        let mut total = 0;
        for action in PLAYER0 {
            let prev = self.undo(action, neg_loss_draw);
            total = self.apply(0, [total, prev], BitOr::bitor);
        }
        dbg!(&self);
        total
    }
}
