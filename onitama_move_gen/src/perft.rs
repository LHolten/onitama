use crate::gen::Game;

pub fn perft(game: Game, depth: usize, player: usize) -> u64 {
    if depth == 0 {
        return 1;
    }
    let mut total = 0;
    game.iter(player, |from, to, card, king| {
        let new_game = game.step(from, to, card, king);
        total += perft(new_game, depth - 1, player ^ 1);
    });
    total
}

// let opp_king = (1 << ((self.0 & KING_MASK) >> 25)).pdep(self.0);
// let opp_temple = [2, 22][player];

pub const TEST_GAME: Game =
    Game(0b1100_010_11111_00000_00000_00000_00000__0011_010_00000_00000_00000_00000_11111);

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_perft() {
        assert_eq!(perft(TEST_GAME, 0, 0), 1);
        assert_eq!(perft(TEST_GAME, 1, 0), 10);
        assert_eq!(perft(TEST_GAME, 2, 0), 130);
        assert_eq!(perft(TEST_GAME, 3, 0), 1989);
        assert_eq!(perft(TEST_GAME, 4, 0), 28509);
        assert_eq!(perft(TEST_GAME, 5, 0), 487780);
        assert_eq!(perft(TEST_GAME, 6, 0), 7748422);
    }
}
