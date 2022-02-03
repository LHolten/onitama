use proof_number_graph::bdd::DecisionDiagram;
use proof_number_graph::tb::{all_actions, PLAYER0, TB};

fn main() {
    let mut tb = TB::full(false);
    let mut count = tb.count();
    loop {
        tb = tb.expand_wins();
        let new_count = tb.count();
        println!("nodes: {}, wins: {}", tb.nodes(), new_count);
        if count == new_count {
            break;
        }
        count = new_count;
    }
    println!("total states: {}", TB::full(true).count())
}
