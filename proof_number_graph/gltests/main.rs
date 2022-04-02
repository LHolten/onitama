use std::collections::HashMap;

use bumpalo::Bump;
use proof_number_graph::{
    sdd::Count,
    sdd_ptr::{ByRef, Never},
    tb::TB,
};

// use proof_number_graph::sdd::Sdd;

fn main() {
    let mut bump;
    let mut tb = ByRef(<TB<'_> as Never>::NEVER);

    loop {
        let tmp_bump = Bump::new();
        let tmp_tb = tb.expand_wins(&tmp_bump);
        bump = Bump::new();
        tb = tmp_tb.expand_wins(&bump);

        let mut count = HashMap::new();
        count.insert(tb, 0);
        TB::count(&mut count);
        println!("{}", count[&tb])
    }
}
