use crate::gen::Game;

#[inline(never)]
fn perft(game: Game, depth: u8) -> u64 {
    let mut total = 0;
    for (new_game, win) in &game {
        if win {
            total += 1;
        } else if depth == 2 {
            total += new_game.count_moves();
        } else {
            total += perft(new_game, depth - 1);
        }
    }
    total
}

pub fn perft_test(depth: u8) -> u64 {
    const TEST_GAME: Game = Game {
        my: 0b11111 + (2 << 25),
        my_cards: 0b0011,
        other: 0b11111 + (2 << 25),
        other_cards: 0b1100,
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
