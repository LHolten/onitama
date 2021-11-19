use crate::{
    for_each_iter::ForEachIter,
    side::{Left, Side},
    state::State,
};

#[inline(never)]
fn perft<S: Side>(state: &mut State<S>, depth: u8) -> usize {
    let mut total = 0;
    state.for_each(|new_state| {
        if depth == 1 {
            total += 1;
        } else {
            total += perft(new_state, depth - 1)
        }
    });
    total
}

pub fn perft_test(depth: u8) -> usize {
    let mut state = State::<Left>::default();
    perft(&mut state, depth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perft() {
        assert_eq!(perft_test(1), 10);
        assert_eq!(perft_test(2), 108);
        assert_eq!(perft_test(3), 1328);
        assert_eq!(perft_test(4), 12388);
        assert_eq!(perft_test(5), 144384);
        assert_eq!(perft_test(6), 1432826);
    }
}
