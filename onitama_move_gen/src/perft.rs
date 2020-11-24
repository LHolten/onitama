use std::mem::MaybeUninit;

use nudge::assume;

use crate::gen::{Game, Move, Player};

pub fn perft(game: Game, depth: usize, mut total: &mut u64) {
    if depth == 0 {
        *total += 1;
        return;
    }
    let mut moves: [MaybeUninit<Move>; 40] = unsafe { MaybeUninit::uninit().assume_init() };

    let mut length = 0;
    game.iter(|m| {
        unsafe { assume(length < 40) }
        moves[length] = MaybeUninit::new(m);
        length += 1;
    });

    unsafe { assume(length < 41) }
    for m in &moves[0..length] {
        let m = unsafe { m.assume_init() };
        // println!("{:#027b}", unsafe { new_game.assume_init() }.other.pieces);
        perft(game.step(m), depth - 1, &mut total);
    }
}

pub fn perft_test(depth: usize) -> u64 {
    const TEST_GAME: Game = Game {
        my: Player {
            pieces: 0b11011,
            king: 0b00100,
            cards: 0b0011,
        },
        other: Player {
            pieces: 0b11011,
            king: 0b00100,
            cards: 0b1100,
        },
    };

    let mut total = 0;
    perft(TEST_GAME, depth, &mut total);
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
