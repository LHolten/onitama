use proof_number_graph::tb::{all_actions, PLAYER0, TB};

fn main() {
    let mut tb = TB::new(false);

    println!("actions len: {}", all_actions(PLAYER0).len());
    // println!(
    //     "action size: {}",
    //     all_actions(PLAYER0)[0].undo_take(&TB::new(true)).nodes()
    // );
    // let mut count = tb.count();
    loop {
        tb = tb.expand_wins();
        // let new_count = tb.count();
        println!("got an iteration, nodes: {}", tb.nodes());
        // println!("nodes: {}, wins: {}", tb.nodes(), new_count);
        // if count == new_count {
        //     break;
        // }
        // count = new_count;
    }
    // println!("total states: {}", TB::new(true).count())
}
