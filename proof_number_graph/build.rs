#![feature(int_abs_diff)]

pub fn all_actions() -> Vec<(usize, usize, usize, usize)> {
    let mut actions = vec![];
    for a in 0..25usize {
        for b in 0..25 {
            let x = (a % 5).abs_diff(b % 5);
            let y = (a / 5).abs_diff(b / 5);
            if x + y == 1 {
                actions.push((a / 5, a % 5, b / 5, b % 5))
            }
        }
    }
    actions
}

fn main() {
    use build_const::ConstWriter;

    let consts = ConstWriter::for_build("constants").unwrap();
    let mut consts = consts.finish_dependencies();

    let values = all_actions();
    consts.add_array("FROM_TO", "(usize, usize, usize, usize)", &values);
}
