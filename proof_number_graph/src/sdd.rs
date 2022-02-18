use std::{
    collections::{hash_map, HashMap, HashSet},
    mem::replace,
};

use bigint::U256;
use bumpalo::Bump;

use crate::sdd_ptr::{ByRef, Entry, Never, Sdd, BDD_ALL, BDD_NONE};

type TB<'a> = Sdd<'a, Sdd<'a, Sdd<'a, Sdd<'a, U256>>>>;

type Input<'a, I> = (ByRef<'a, I>, ByRef<'a, I>);

struct Context<'a> {
    bump: &'a Bump,
    bdd: HashSet<&'a U256>,
    func: fn(U256, U256) -> U256,
    view: fn(usize, U256) -> U256,
}

impl<'a> Context<'a> {
    fn unique_bdd(&mut self, val: U256) -> ByRef<'a, U256> {
        let ptr = self
            .bdd
            .get_or_insert_with(&val, |val| &*self.bump.alloc(*val));
        ByRef(ptr)
    }
}

trait Decision {
    const DEPTH: usize;
    type Out<'a>: Never<'a> + 'a;

    fn apply<'a>(
        comp: Vec<Input<'_, Self::Out<'_>>>,
        context: &mut Context<'a>,
    ) -> Vec<ByRef<'a, Self::Out<'a>>>;
}

impl Decision for U256 {
    const DEPTH: usize = 0;
    type Out<'a> = U256;

    fn apply<'a>(
        comp: Vec<Input<'_, Self::Out<'_>>>,
        context: &mut Context<'a>,
    ) -> Vec<ByRef<'a, Self::Out<'a>>> {
        let mut res = vec![];
        for (left, right) in comp {
            res.push(context.unique_bdd((context.func)(*left.0, *right.0)))
        }
        res
    }
}

impl<T: Decision> Decision for Sdd<'_, T> {
    const DEPTH: usize = T::DEPTH + 1;
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
                if let hash_map::Entry::Vacant(e) = res_index.entry((left, right)) {
                    e.insert(res_count);
                    res_count += 1;
                    for e1 in left.0 {
                        for e2 in right.0 {
                            // TODO apply view to e2 and add missing branch
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
                    let mut prev = iter.next().unwrap();
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
