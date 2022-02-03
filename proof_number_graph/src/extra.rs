// forward player 1 move, if all of them result in a win for player 0
// backward player 0 move, mark all as win for player 0
// current state is player 1 to move
impl State {
    // every method only returns states that are not a win in 1

    // this returns the set of states that have a temple threat
    // player 0 threatens the temple of player 1
    // this means that player 0 has won
    fn temple_threat() -> State {
        todo!()
    }

    fn double_king_threat() -> State {
        todo!()
    }

    // this returns the set of states with the following conditions
    // - the king of player 1 is threatened by one piece
    // - taking that piece is still a win for player 0
    // input is the set of states won by player 0
    fn single_king_threat(&self) -> State {
        todo!()
    }

    // return states where all pawn moves result in states won by player 0
    // and also king of player 1 is not threatened
    fn no_king_threat(&self) -> State {
        todo!()
    }

    // return states where all king moves result in states won by player 0
    fn king1all(&self) -> State {
        todo!()
    }

    fn all_lost(&self) -> State {
        Self::temple_threat() | {
            // king can not move
            self.king1all() & {
                // it's also not possible to leave the king
                self.no_king_threat() | self.single_king_threat() | Self::double_king_threat()
            }
        }
    }
}

impl State {
    // get position with player 0 to move from position with player 1 to move
    fn all_won(&self) -> State {
        self.king0any() | self.pawn0any()
    }

    fn king0any(&self) -> State {
        // generate all possible moves with cards from all begin positions
        // or them all together
        todo!()
    }

    fn pawn0any(&self) -> State {
        todo!()
    }
}
