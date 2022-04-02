use std::collections::{HashMap, HashSet};

use bumpalo::Bump;

use crate::sdd_ptr::{ByRef, Entry, Never, Sdd, BDD, BDD_ALL, BDD_NONE};

type Input<'a, I> = (ByRef<'a, I>, ByRef<'a, I>);

#[derive(Debug, Clone, Copy)]
pub struct View {
    pub func: fn(usize, BDD) -> BDD, // will map inputs with precond to postcond
    pub mask: fn(usize) -> BDD,      // everything that does not fit in the postcond
}

pub struct Context<'a> {
    bump: &'a Bump,
    bdd: HashSet<&'a BDD>,
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

    fn unique_bdd(&mut self, val: BDD) -> ByRef<'a, BDD> {
        let ptr = self
            .bdd
            .get_or_insert_with(&val, |val| &*self.bump.alloc(*val));
        ByRef(ptr)
    }

    fn pairs<'s, T: Never<'s>>(&self, input: Input<'s, Sdd<'s, T>>) -> Vec<(BDD, Input<'s, T>)> {
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

impl<'a> Decision<'a, BDD> for Context<'a> {
    type In<'s> = BDD;

    fn apply(&mut self, comp: &mut HashMap<Input<'_, BDD>, ByRef<'a, BDD>>) {
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

        for input in comp.keys() {
            for (_, pair) in self.pairs(*input) {
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

pub trait Count: Sized {
    fn count(comp: &mut HashMap<ByRef<'_, Self>, u64>);
}

impl Count for BDD {
    fn count(comp: &mut HashMap<ByRef<'_, Self>, u64>) {
        for (input, output) in comp {
            *output = input.0 .0.into_iter().map(|v| v.count_ones() as u64).sum()
        }
    }
}
impl<T: Count> Count for Sdd<'_, T> {
    fn count(comp: &mut HashMap<ByRef<'_, Self>, u64>) {
        let mut required = HashMap::new();

        for input in comp.keys() {
            for edge in input.0 {
                required.insert(edge.next, 0);
            }
        }

        T::count(&mut required);

        for (input, res) in comp {
            for edge in input.0 {
                let cond = edge.cond.0 .0.into_iter();
                let count = cond.map(|v| v.count_zeros() as u64).sum::<u64>();
                *res += count * required[&edge.next];
            }
        }
    }
}
