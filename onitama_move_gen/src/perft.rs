use std::{hint::black_box, mem::MaybeUninit};

use nudge::assume;

use crate::gen::Game;

pub fn perft(game: Game, max_depth: u8) -> u64 {
    let mut stack: [MaybeUninit<Game>; 40 * 6] = unsafe { MaybeUninit::uninit().assume_init() };
    stack[0] = MaybeUninit::new(game);
    let mut height = 1;

    let mut total = 0;
    while height != 0 {
        height -= 1;
        unsafe { assume(height < stack.len()) }
        let new_game = unsafe { stack[height].assume_init_read() };
        new_game.new_games(
            #[inline(always)]
            |new_game, win| {
                if win {
                    total += 1;
                } else if new_game.depth == max_depth - 1 {
                    total += new_game.count_moves()
                } else {
                    unsafe { assume(height < stack.len()) }
                    stack[height] = MaybeUninit::new(new_game);
                    height += 1;
                }
            },
        )
    }
    total
}

pub fn perft_test(depth: u8) -> u64 {
    const TEST_GAME: Game = Game {
        my: 0b11111 + (2 << 25),
        my_cards: 0b0011,
        other: 0b11111 + (2 << 25),
        other_cards: 0b1100,
        depth: 0,
    };

    perft(TEST_GAME, depth)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_perft() {
        // assert_eq!(perft_test(0), 1);
        // assert_eq!(perft_test(1), 10);
        assert_eq!(perft_test(2), 130);
        assert_eq!(perft_test(3), 1989);
        assert_eq!(perft_test(4), 28509);
        assert_eq!(perft_test(5), 487780);
        assert_eq!(perft_test(6), 7748422);
    }
}
