use std::collections::{HashMap, HashSet};

use bigint::U256;
use bumpalo::Bump;

use crate::sdd_ptr::{ByRef, Entry, Never, Sdd, BDD_ALL, BDD_NONE};

type Input<'a, I> = (ByRef<'a, I>, ByRef<'a, I>);

#[derive(Debug, Clone, Copy)]
pub struct View {
    pub func: fn(usize, U256) -> U256, // will map inputs with precond to postcond
    pub mask: fn(usize) -> U256,       // everything that does not fit in the postcond
}

pub struct Context<'a> {
    bump: &'a Bump,
    bdd: HashSet<&'a U256>,
    view: View,
}

impl<'a> Context<'a> {
    pub fn new(bump: &'a Bump, view: View) -> Self {
        Context {
            bump,
            view,
            bdd: Default::default(),
        }
    }

    fn unique_bdd(&mut self, val: U256) -> ByRef<'a, U256> {
        let ptr = self
            .bdd
            .get_or_insert_with(&val, |val| &*self.bump.alloc(*val));
        ByRef(ptr)
    }

    fn pairs<'s, T: Never<'s>>(&self, input: Input<'s, Sdd<'s, T>>) -> Vec<(U256, Input<'s, T>)> {
        let mut right_0 = vec![];
        for e in input.1 .0.into_iter().copied() {
            let cond = (self.view.func)(Sdd::<'s, T>::DEPTH, *e.cond.0);
            right_0.push((cond, e.next))
        }
        let mask = (self.view.mask)(Sdd::<'s, T>::DEPTH);
        if mask != BDD_NONE {
            right_0.push((mask, ByRef(T::NEVER)));
        }

        let mut res = vec![];
        for e1 in input.0 .0 {
            for (e2_cond, e2_next) in right_0.iter().copied() {
                let cond = *e1.cond.0 & e2_cond;
                if cond != BDD_NONE {
                    res.push((cond, (e1.next, e2_next)));
                }
            }
        }
        res
    }
}

pub trait Decision<'a, Out> {
    type In<'s>: Never<'s>;

    fn apply<'s>(&mut self, comp: &mut HashMap<Input<'s, Self::In<'s>>, ByRef<'a, Out>>);
}

impl<'a> Decision<'a, U256> for Context<'a> {
    type In<'s> = U256;

    fn apply(&mut self, comp: &mut HashMap<Input<'_, U256>, ByRef<'a, U256>>) {
        for ((left, right), res) in comp {
            let new = (self.view.func)(Self::In::<'_>::DEPTH, BDD_ALL ^ *right.0);
            *res = self.unique_bdd(*left.0 | new)
        }
    }
}

impl<'a, T: Never<'a>> Decision<'a, Sdd<'a, T>> for Context<'a>
where
    Self: Decision<'a, T>,
{
    type In<'s> = Sdd<'s, <Self as Decision<'a, T>>::In<'s>>;

    fn apply<'s>(&mut self, comp: &mut HashMap<Input<'s, Self::In<'s>>, ByRef<'a, Sdd<'a, T>>>) {
        let mut required = HashMap::new();

        for inputs in comp.keys().copied() {
            for (_, pair) in self.pairs(inputs) {
                required.insert(pair, ByRef(T::NEVER));
            }
        }

        self.apply(&mut required);

        let mut unique = HashSet::new();

        for (input, res) in comp {
            let mut new = vec![];
            for (cond, pair) in self.pairs(*input) {
                new.push((cond, required[&pair]));
            }

            new.sort_unstable_by_key(|e| e.1);
            let mut iter = new.drain(..);
            let mut prev = iter.next().unwrap(); // there is at least one item
            let iter = iter.filter_map(|item| {
                if item.1 == prev.1 {
                    prev.0 = prev.0 | item.0;
                    None
                } else {
                    let entry = Entry {
                        next: prev.1,
                        cond: self.unique_bdd(prev.0),
                    };
                    prev = item;
                    Some(entry)
                }
            });
            let mut new = iter.collect::<Vec<_>>();
            new.push(Entry {
                next: prev.1,
                cond: self.unique_bdd(prev.0),
            });

            let ptr = *unique.get_or_insert_with(&*new, |sdd| &*self.bump.alloc_slice_copy(sdd));
            *res = ByRef(Sdd::new(ptr));
        }
    }
}
