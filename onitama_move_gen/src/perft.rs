use std::mem::MaybeUninit;

use nudge::assume;

use crate::gen::Game;

pub fn perft(game: u64, depth: usize, mut total: &mut u64) {
    let game = Game::deflate(game);
    if depth == 0 {
        *total += 1;
        return;
    }
    let mut new_games: [MaybeUninit<u64>; 40] = unsafe { MaybeUninit::uninit().assume_init() };

    let mut length = 0;
    game.iter(|m| {
        unsafe { assume(length < 40) }
        new_games[length] = MaybeUninit::new(game.step(m).compress());
        length += 1;
    });

    unsafe { assume(length < 41) }
    for new_game in &new_games[0..length] {
        perft(unsafe { new_game.assume_init() }, depth - 1, &mut total);
    }
}

pub fn perft_test(depth: usize) -> u64 {
    const TEST_GAME: Game = Game {
        my: 0b0011_010_00000_00000_00000_00000_11111,
        other: 0b1100_010_00000_00000_00000_00000_11111,
    };

    let mut total = 0;
    perft(TEST_GAME.compress(), depth, &mut total);
    total
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_perft() {
        assert_eq!(perft_test(0), 1);
        assert_eq!(perft_test(1), 10);
        assert_eq!(perft_test(2), 130);
        assert_eq!(perft_test(3), 1989);
        assert_eq!(perft_test(4), 28509);
        assert_eq!(perft_test(5), 487780);
        assert_eq!(perft_test(6), 7748422);
    }
}
