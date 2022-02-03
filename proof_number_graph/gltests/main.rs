use proof_number_graph::bdd::DecisionDiagram;
use proof_number_graph::tb::{all_actions, PLAYER0, TB};

fn main() {
    all_actions(PLAYER0)
        .into_iter()
        .fold(TB::full(false), |p, a| {
            let new = p | a.possible();
            println!("{}", new.nodes());
            new
        });
}
