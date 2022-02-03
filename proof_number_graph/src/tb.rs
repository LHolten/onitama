use crate::bdd::{Bdd, Choice, DecisionDiagram};

type Line<T> = Bdd<[Bdd<[Bdd<[Bdd<[Bdd<[T; 3]>; 3]>; 3]>; 3]>; 3]>;
pub type TB = Line<Line<Line<Line<Line<bool>>>>>;

#[derive(Clone)]
pub struct Action {
    pos: [usize; 2],
    my: [Choice; 2],
}

fn eq<const P: usize>(i: usize) -> bool {
    i == P
}
fn uneq<const P: usize>(i: usize) -> bool {
    i != P
}
pub const PLAYER0: [Choice; 2] = [eq::<0>, uneq::<0>];
pub const PLAYER1: [Choice; 2] = [eq::<1>, uneq::<1>];

pub fn all_actions(player: [Choice; 2]) -> Vec<Action> {
    let mut actions = vec![];
    for a in 0..25 {
        for b in 0..25 {
            let x = (a % 5usize).abs_diff(b % 5);
            let y = (a / 5usize).abs_diff(b / 5);
            if x + y == 1 || x + y == 2 {
                // flips from bit and to bit

                actions.push(Action {
                    pos: [a, b],
                    my: player,
                })
            }
        }
    }
    actions
}

impl Action {
    pub fn undo(&self, mut state: TB) -> TB {
        state = state.set(self.pos[0], self.my[0]);
        state.set(self.pos[1], self.my[1])
    }

    pub fn possible(&self) -> TB {
        self.undo(TB::full(true))
    }
}

impl TB {
    pub fn expand_wins(self) -> TB {
        let mut losses = Self::full(true);
        for action in all_actions(PLAYER1) {
            losses &= action.undo(self.clone()) | !action.possible()
        }
        let mut wins = Self::full(false);
        for action in all_actions(PLAYER0) {
            wins |= action.undo(losses.clone())
        }
        wins
    }
}
