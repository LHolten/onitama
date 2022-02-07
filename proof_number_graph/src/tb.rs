use crate::bdd::{Bdd, RcStore};

pub type TB = Bdd;

#[derive(Clone)]
pub struct Action {
    pos: [u32; 2],
    my_other_empty: [usize; 3],
}

pub const PLAYER0: [usize; 3] = [0, 1, 2];
pub const PLAYER1: [usize; 3] = [1, 0, 2];

pub fn all_actions(player: [usize; 3]) -> Vec<Action> {
    let mut actions = vec![];
    for a in 0..25 {
        for b in 0..25 {
            let x = (a % 5u32).abs_diff(b % 5);
            let y = (a / 5u32).abs_diff(b / 5);
            if x + y == 1 {
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
    pub fn undo_take(&self, store: &mut RcStore, state: &TB) -> TB {
        let [my, other, empty] = self.my_other_empty;
        let state = store.set(state, self.pos[1], my, other, 3);
        store.set(&state, self.pos[0], empty, my, 3)
    }

    pub fn undo_no_take(&self, store: &mut RcStore, state: &TB) -> TB {
        let [my, _other, empty] = self.my_other_empty;
        let state = store.set(state, self.pos[1], my, empty, 3);
        store.set(&state, self.pos[0], empty, my, 3)
    }
}

impl TB {
    pub fn expand_wins(self) -> TB {
        let loss_draw = {
            let neg_wins = !self;
            // from the perspective of player 0
            let mut store = RcStore::default();
            let all: Vec<_> = all_actions(PLAYER1)
                .into_iter()
                .flat_map(|a| {
                    [
                        a.undo_take(&mut store, &neg_wins),
                        a.undo_no_take(&mut store, &neg_wins),
                    ]
                })
                .collect();
            TB::or(all)
        };

        let neg_loss_draw = !loss_draw;
        // from the perspective of player 0
        let mut store = RcStore::default();
        let all: Vec<_> = all_actions(PLAYER0)
            .into_iter()
            .flat_map(|a| {
                [
                    a.undo_take(&mut store, &neg_loss_draw),
                    a.undo_no_take(&mut store, &neg_loss_draw),
                ]
            })
            .collect();
        TB::or(all)
    }
}
