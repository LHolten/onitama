use std::{
    collections::{hash_map, HashMap, HashSet},
    mem::replace,
};

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

pub trait Decision {
    type Out<'a>: Never<'a>;

    fn apply<'a>(
        comp: Vec<Input<'_, Self::Out<'_>>>,
        context: &mut Context<'a>,
    ) -> Vec<ByRef<'a, Self::Out<'a>>>;
}

impl Decision for U256 {
    type Out<'a> = U256;

    fn apply<'a>(
        comp: Vec<Input<'_, Self::Out<'_>>>,
        context: &mut Context<'a>,
    ) -> Vec<ByRef<'a, Self::Out<'a>>> {
        let mut res = vec![];
        for (left, right) in comp {
            let new = (context.view.func)(Self::Out::<'_>::DEPTH, BDD_ALL ^ *right.0);
            res.push(context.unique_bdd(*left.0 | new))
        }
        res
    }
}

impl<T: Decision> Decision for Sdd<'_, T> {
    type Out<'a> = Sdd<'a, T::Out<'a>>;

    fn apply<'a>(
        comp: Vec<Input<'_, Self::Out<'_>>>,
        context: &mut Context<'a>,
    ) -> Vec<ByRef<'a, Self::Out<'a>>> {
        let mut required = vec![];
        let mut conds = vec![];
        let mut comp_index = vec![];
        {
            let mut res_index = HashMap::new();
            let mut res_count = 0;
            for (left, right) in comp {
                if let hash_map::Entry::Vacant(hash_entry) = res_index.entry((left, right)) {
                    hash_entry.insert(res_count);
                    res_count += 1;

                    let mut edges2 = vec![];
                    for mut e in right.0.into_iter().copied() {
                        e.cond = context
                            .unique_bdd((context.view.func)(Self::Out::<'_>::DEPTH, *e.cond.0));
                        edges2.push(e)
                    }
                    edges2.push(Entry {
                        next: ByRef(T::Out::<'_>::NEVER),
                        cond: context.unique_bdd((context.view.mask)(Self::Out::<'_>::DEPTH)),
                    });

                    for e1 in left.0 {
                        for e2 in &edges2 {
                            let cond = *e1.cond.0 & *e2.cond.0;
                            if cond != BDD_NONE {
                                conds.push(context.unique_bdd(cond));
                                required.push((e1.next, e2.next));
                            }
                        }
                    }
                }
                comp_index.push(res_index[&(left, right)]);
            }
        }

        let next = T::apply(required, context);

        let mut res = vec![];
        {
            let mut res_unique = HashSet::new();
            let mut new = vec![];
            let mut total = BDD_NONE;
            for (cond, next) in conds.into_iter().zip(next) {
                new.push(Entry { next, cond });
                total = total | *cond.0;
                if total == BDD_ALL {
                    new.sort_unstable_by_key(|e| e.next);

                    let mut iter = new.drain(..);
                    let mut prev = iter.next().unwrap(); // there is at least one item
                    let iter = iter.filter_map(|item| {
                        if item.next == prev.next {
                            prev.cond = context.unique_bdd(*prev.cond.0 | *item.cond.0);
                            None
                        } else {
                            Some(replace(&mut prev, item))
                        }
                    });
                    let mut new = iter.collect::<Vec<_>>();
                    new.push(prev);

                    let ptr = res_unique
                        .get_or_insert_with(&*new, |sdd| &*context.bump.alloc_slice_copy(sdd));
                    res.push(ByRef(Sdd::new(ptr)));

                    new.clear();
                    total = BDD_NONE;
                }
            }
        }

        comp_index.into_iter().map(|i| res[i]).collect()
    }
}
