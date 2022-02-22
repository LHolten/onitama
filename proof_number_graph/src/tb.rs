use bigint::U256;
use bumpalo::Bump;

use crate::{
    sdd::{Context, Decision, View},
    sdd_ptr::{ByRef, Never, Sdd, BDD_ALL},
};

use seq_macro::seq;

pub type TB<'a> = Sdd<'a, Sdd<'a, Sdd<'a, Sdd<'a, U256>>>>;

include!(concat!(env!("OUT_DIR"), concat!("/", "constants.rs")));

seq!(N in 0..80 {
    pub const PLAYER0: &[View] = &[
        #(View {
            func: transform::<0, N>,
            mask: leftover::<0, N>
        },)*
    ];
});

seq!(N in 0..80 {
    pub const PLAYER1: &[View] = &[
        #(View {
            func: transform::<1, N>,
            mask: leftover::<1, N>
        },)*
    ];
});

//assume pos < 5 and val < 3
fn pos_mask(pos: usize) -> (U256, usize) {
    let mut mask = U256::one();
    for i in 0..5 {
        let step = 3usize.pow(i as u32);
        if i != pos {
            mask = mask | mask << step | mask << (step * 2)
        }
    }
    (mask, 3usize.pow(pos as u32))
}

fn transform<const PLAYER: usize, const ACTION: usize>(depth: usize, mut bdd: U256) -> U256 {
    let (from_depth, from_pos, to_depth, to_pos) = FROM_TO[ACTION];
    if from_depth == depth {
        let (mask, offset) = pos_mask(from_pos);
        let bdd_mask = mask << (offset * 2); // empty space
        bdd = (bdd & bdd_mask) >> (offset * 2) << (offset * PLAYER);
    }
    if to_depth == depth {
        let (mask, offset) = pos_mask(to_pos);
        let bdd_mask = mask << (offset * PLAYER);
        let unshifted = (bdd & bdd_mask) >> (offset * PLAYER);
        bdd = unshifted << ((1 - PLAYER) * offset) | unshifted << (offset * 2)
    }
    bdd
}

fn leftover<const PLAYER: usize, const ACTION: usize>(depth: usize) -> U256 {
    BDD_ALL ^ transform::<PLAYER, ACTION>(depth, BDD_ALL)
}

impl<'c> ByRef<'c, TB<'c>> {
    pub fn expand_wins<'a>(self, bump: &'a Bump) -> ByRef<'a, TB<'a>> {
        let mut neg_wins = ByRef(TB::<'_>::NEVER);
        for view in PLAYER1 {
            neg_wins = TB::apply(vec![(neg_wins, self)], &mut Context::new(bump, *view))
                .pop()
                .unwrap();
        }
        println!("step");

        let mut wins = ByRef(TB::<'_>::NEVER);
        for view in PLAYER0 {
            wins = TB::apply(vec![(wins, wins)], &mut Context::new(bump, *view))
                .pop()
                .unwrap();
        }
        println!("step");
        wins
    }
}
