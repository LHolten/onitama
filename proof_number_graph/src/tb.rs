use crate::bdd::{Bdd, DecisionDiagram};

type Line<T> = Bdd<[Bdd<[Bdd<[Bdd<[Bdd<[T; 3]>; 3]>; 3]>; 3]>; 3]>;
pub type TB = Line<Line<Line<Line<Line<bool>>>>>;

#[derive(Clone)]
pub struct Action {
    pos: [usize; 2],
    my_other_empty: [usize; 3],
}

pub const PLAYER0: [usize; 3] = [0, 1, 2];
pub const PLAYER1: [usize; 3] = [1, 0, 2];

pub fn all_actions(player: [usize; 3]) -> Vec<Action> {
    let mut actions = vec![];
    for a in 0..25 {
        for b in 0..25 {
            let x = (a % 5usize).abs_diff(b % 5);
            let y = (a / 5usize).abs_diff(b / 5);
            if x + y == 1 || x + y == 2 {
                // flips from bit and to bit

                actions.push(Action {
                    pos: [a, b],
                    my_other_empty: player,
                })
            }
        }
    }
    actions
}

impl Action {
    pub fn undo(&self, mut state: TB) -> TB {
        state = state.set(self.pos[0], self.my_other_empty[1], self.my_other_empty[0]);
        state = state.set(self.pos[0], self.my_other_empty[2], self.my_other_empty[0]); // overwrites the prev one
        let mut take = state.clone();
        take = take.set(self.pos[1], self.my_other_empty[0], self.my_other_empty[1]);
        state = state.set(self.pos[1], self.my_other_empty[0], self.my_other_empty[2]);
        state | take
    }

    pub fn possible(&self) -> TB {
        self.undo(TB::full(true))
    }
}

impl TB {
    pub fn expand_wins(self) -> TB {
        let neg_self = !self;
        // from the perspective of player 0
        let mut loss_draw = Self::full(false);
        for action in all_actions(PLAYER1) {
            // check if player 1 can force a loss or draw
            loss_draw |= action.undo(neg_self.clone());
            println!("loss_draw size: {}", loss_draw.nodes());
        }
        let neg_loss_draw = !loss_draw;
        let mut wins = Self::full(false);
        for action in all_actions(PLAYER0) {
            // check if player 0 can force a win
            wins |= action.undo(neg_loss_draw.clone());
            println!("wins size: {}", wins.nodes());
        }
        wins
    }
}
