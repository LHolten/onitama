use std::collections::HashMap;

use bumpalo::Bump;

use crate::{
    sdd::{Context, Decision, View},
    sdd_ptr::{ByRef, Never, Sdd, BDD, BDD_ALL},
};

use seq_macro::seq;

pub type TB<'a> = Sdd<'a, Sdd<'a, Sdd<'a, Sdd<'a, BDD>>>>;

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
fn pos_mask(pos: usize) -> (BDD, usize) {
    let mut mask = BDD::one();
    for i in 0..5 {
        let step = 3usize.pow(i as u32);
        if i != pos {
            mask = mask | mask << step | mask << (step * 2)
        }
    }
    (mask, 3usize.pow(pos as u32))
}

fn transform<const PLAYER: usize, const ACTION: usize>(depth: usize, mut bdd: BDD) -> BDD {
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

fn leftover<const PLAYER: usize, const ACTION: usize>(depth: usize) -> BDD {
    BDD_ALL ^ transform::<PLAYER, ACTION>(depth, BDD_ALL)
}

impl<'c> ByRef<'c, TB<'c>> {
    pub fn expand_wins<'a>(self, bump: &'a Bump) -> ByRef<'a, TB<'a>> {
        let mut neg_wins = ByRef(TB::<'_>::NEVER);
        for view in PLAYER1 {
            let mut comp = HashMap::new();
            comp.insert((neg_wins, self), ByRef(TB::<'_>::NEVER));
            Context::new(bump, *view).apply(&mut comp);
            neg_wins = comp[&(neg_wins, self)];
        }
        println!("step");

        let mut wins = ByRef(TB::<'_>::NEVER);
        for view in PLAYER0 {
            let mut comp = HashMap::new();
            comp.insert((wins, neg_wins), ByRef(TB::<'_>::NEVER));
            Context::new(bump, *view).apply(&mut comp);
            wins = comp[&(wins, neg_wins)];
        }
        println!("step");
        wins
    }
}
