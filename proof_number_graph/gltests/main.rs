use bumpalo::Bump;
use proof_number_graph::{
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
        println!("{}", bump.allocated_bytes())
    }
}
