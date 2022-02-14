use std::{
    array::from_fn,
    collections::HashMap,
    fmt::Debug,
    ops::{BitAnd, BitOr},
};

use bigint::U256;

const EMPTY_BITS: usize = 256 - 3 * 3 * 3 * 3 * 3;
const BDD_ALL: U256 = U256([u64::MAX, u64::MAX, u64::MAX, u64::MAX >> EMPTY_BITS]);
const BDD_NONE: U256 = U256([0; 4]);

pub struct Sdd {
    bdds: Vec<U256>,
    rows: [Vec<Edge>; 5], // first index is always empty
    compute: [HashMap<Computation, usize>; 5],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Edge {
    next: usize,
    bdd: usize,
}

type BoolFn = fn(bool, bool) -> bool;
type U256Op = fn(U256, U256) -> U256;

#[derive(Debug, PartialEq, Eq, Hash)]
struct Computation {
    args: [usize; 2],
    func: BoolFn,
}

impl Sdd {
    fn unique_bdd(&mut self, val: U256) -> usize {
        if let Some(res) = self.bdds.iter().position(|old| old == &val) {
            res
        } else {
            self.bdds.push(val);
            self.bdds.len() - 1
        }
    }

    fn apply_bdd(&mut self, args: [usize; 2], func: U256Op) -> usize {
        self.unique_bdd(func(self.bdds[args[0]], self.bdds[args[1]]))
    }

    fn unique_sdd(&mut self, row: usize, mut val: Vec<Edge>) -> usize {
        // insert empty for missing bdds
        let mut total = BDD_NONE;
        for e in &val {
            total = total | self.bdds[e.bdd]
        }
        val.push(Edge {
            next: 0,
            bdd: self.unique_bdd(BDD_ALL & !total),
        });

        // remove impossible and combine duplicates
        val.retain(|e| e.bdd != 0);
        val.sort_unstable();
        let mut iter = val.into_iter();
        let mut combined = vec![];

        let mut e1 = iter.next().unwrap();
        for e2 in iter {
            if e2.next == e1.next {
                e1.bdd = self.apply_bdd([e1.bdd, e2.bdd], BitOr::bitor);
            } else {
                combined.push(e1);
                e1 = e2
            }
        }
        combined.push(e1);

        let mut start = 0;
        for (i, edge) in self.rows[row].iter().enumerate() {
            if edge == &combined[i - start] {
                if i + 1 - start == combined.len() {
                    return start;
                }
            } else {
                start = i + 1;
            }
        }
        self.rows[row].extend(combined);
        start
    }

    pub fn apply(&mut self, row: usize, args: [usize; 2], func: BoolFn) -> usize {
        if row == self.rows.len() {
            return func(args[0] != 0, args[1] != 0) as usize;
        }
        let computation = Computation { args, func };
        if let Some(res) = self.compute[row].get(&computation) {
            return *res;
        }

        let mut all = vec![];
        let edges1 = self.edges(row, args[0]);
        let edges2 = self.edges(row, args[1]);
        for e1 in &edges1 {
            for e2 in &edges2 {
                let bdd = self.apply_bdd([e1.bdd, e2.bdd], BitAnd::bitand);
                if bdd != 0 {
                    all.push(Edge {
                        next: self.apply(row + 1, [e1.next, e2.next], func),
                        bdd,
                    })
                }
            }
        }

        let res = self.unique_sdd(row, all);
        self.compute[row].try_insert(computation, res).unwrap();
        res
    }

    fn edges(&mut self, row: usize, mut arg: usize) -> Vec<Edge> {
        let mut all = vec![];
        let mut total = BDD_NONE;
        while total != BDD_ALL {
            let edge = self.rows[row][arg];
            total = total | self.bdds[edge.bdd];
            all.push(edge);
            arg += 1;
        }
        all
    }

    // first use BitAnd to get positions that can be unmoved with the given move
    // then use this function to do the unmove, it assumes that all illegal state are already handled
    // func should turn a bdd that requires to into one that requires from but is otherwise the same
    // TODO implement caching
    pub fn transform(
        &mut self,
        row: usize,
        arg: usize,
        func: fn(U256) -> U256,
        target: usize,
    ) -> usize {
        let edges = self.edges(row, arg).into_iter();
        let new = if row == target {
            edges
                .map(|e| Edge {
                    next: e.next,
                    bdd: self.unique_bdd(func(self.bdds[e.bdd])),
                })
                .collect() // this is not complete
        } else {
            edges
                .map(|e| Edge {
                    next: self.transform(row + 1, e.next, func, target),
                    bdd: e.bdd,
                })
                .collect()
        };
        self.unique_sdd(row, new)
    }
}

impl Default for Sdd {
    fn default() -> Self {
        Self {
            bdds: vec![BDD_NONE, BDD_ALL],
            rows: from_fn(|_| vec![Edge { next: 0, bdd: 1 }, Edge { next: 1, bdd: 1 }]),
            compute: Default::default(),
        }
    }
}

impl Debug for Sdd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sdd")
            .field("bdds", &self.bdds.len())
            .field(
                "rows",
                &self.rows.iter().map(|row| row.len()).collect::<Vec<_>>(),
            )
            .field(
                "compute",
                &self.compute.iter().map(|row| row.len()).collect::<Vec<_>>(),
            )
            .finish()
    }
}
