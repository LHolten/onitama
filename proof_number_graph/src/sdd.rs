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
}

pub trait Decision<'s>: Never<'s> {
    type Out<'a>: Decision<'a>;

    fn apply<'a>(
        comp: &mut HashMap<Input<'s, Self>, ByRef<'a, Self::Out<'a>>>,
        context: &mut Context<'a>,
    );
}

impl<'s> Decision<'s> for U256 {
    type Out<'a> = U256;

    fn apply<'a>(
        comp: &mut HashMap<Input<'s, Self>, ByRef<'a, Self::Out<'a>>>,
        context: &mut Context<'a>,
    ) {
        for ((left, right), res) in comp {
            let new = (context.view.func)(Self::Out::<'a>::DEPTH, BDD_ALL ^ *right.0);
            *res = context.unique_bdd(*left.0 | new)
        }
    }
}

impl<'s, T: Decision<'s>> Decision<'s> for Sdd<'s, T> {
    type Out<'a> = Sdd<'a, T::Out<'a>>;

    fn apply<'a>(
        comp: &mut HashMap<Input<'s, Self>, ByRef<'a, Self::Out<'a>>>,
        context: &mut Context<'a>,
    ) {
        let mut required = HashMap::new();

        for inputs in comp.keys().copied() {
            for (_, pair) in Self::pairs(inputs, context) {
                required.insert(pair, ByRef(T::Out::<'a>::NEVER));
            }
        }

        T::apply(&mut required, context);

        let mut unique = HashSet::new();

        for (input, res) in comp {
            let mut new = vec![];
            for (cond, pair) in Self::pairs(*input, context) {
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
                        cond: context.unique_bdd(prev.0),
                    };
                    prev = item;
                    Some(entry)
                }
            });
            let mut new = iter.collect::<Vec<_>>();
            new.push(Entry {
                next: prev.1,
                cond: context.unique_bdd(prev.0),
            });

            let ptr = *unique.get_or_insert_with(&*new, |sdd| &*context.bump.alloc_slice_copy(sdd));
            *res = ByRef(Sdd::new(ptr));
        }
    }
}

impl<'s, T: Decision<'s>> Sdd<'s, T> {
    fn pairs(
        input: (ByRef<'s, Self>, ByRef<'s, Self>),
        context: &Context<'_>,
    ) -> Vec<(U256, (ByRef<'s, T>, ByRef<'s, T>))> {
        let mut right_0 = vec![];
        for e in input.1 .0.into_iter().copied() {
            let cond = (context.view.func)(Sdd::<'s, T>::DEPTH, *e.cond.0);
            right_0.push((cond, e.next))
        }
        let mask = (context.view.mask)(Sdd::<'s, T>::DEPTH);
        if mask != BDD_NONE {
            right_0.push((mask, ByRef(T::NEVER)));
        }

        let mut res = vec![];
        for e1 in input.0 .0 {
            for e2 in &right_0 {
                let cond = *e1.cond.0 & e2.0;
                if cond != BDD_NONE {
                    res.push((cond, (e1.next, e2.1)));
                }
            }
        }
        res
    }
}
