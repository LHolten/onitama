use std::{collections::HashMap, hash::Hash};

use crate::solver::{Literal, IPASIR};

mod solver;

pub trait Signature<S> {
    fn signature(&self) -> S;
}

struct SatSolver<T, S> {
    literals: HashMap<S, Literal>,
    unexpanded: HashMap<Literal, T>,
    solver: IPASIR,
}

impl<T, S> Default for SatSolver<T, S> {
    fn default() -> Self {
        Self {
            literals: Default::default(),
            unexpanded: Default::default(),
            solver: Default::default(),
        }
    }
}

impl<T, S> SatSolver<T, S>
where
    T: Iterator<Item = T> + Signature<S>,
    S: Eq + Hash,
{
    pub fn expand(&mut self, literal: Literal) {
        if let Some(state) = self.unexpanded.remove(&literal) {
            for next_state in state {
                self.solver.add(-literal);
                for next_next_state in next_state {
                    let literal = self.get_literal(next_next_state);
                    self.solver.add(literal);
                }
                self.solver.add(Literal::default()); // finish the clause
            }
        }
    }

    pub fn get_literal(&mut self, state: T) -> Literal {
        let signature = state.signature();
        if let Some(&literal) = self.literals.get(&signature) {
            literal
        } else {
            let literal = self.solver.new_literal();
            self.literals.insert(signature, literal);
            self.unexpanded.insert(literal, state);
            literal
        }
    }

    pub fn is_win_draw<N>(&mut self, next_literals: N) -> Option<impl '_ + Fn(Literal) -> bool>
    where
        N: IntoIterator<Item = Literal>,
    {
        let win_draw = self.solver.new_literal();

        // one of the next states has to be loss_draw
        self.solver.add(-win_draw);
        for next_literal in next_literals {
            self.solver.add(next_literal);
        }
        self.solver.add(Literal::default());

        self.solver.assume(win_draw);
        self.solver.solve()
    }
}

pub fn is_win<T>(mut state: T) -> bool
where
    T: Iterator<Item = T>,
{
    state.all(|mut next| next.any(is_win))
}
