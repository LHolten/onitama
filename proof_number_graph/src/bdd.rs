use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    ops::Not,
    ptr,
    rc::Rc,
};

pub type Op = fn(bool, bool) -> bool;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Bdd {
    Bool(bool),
    Edge(Edge),
}

#[derive(Clone, Eq)]
pub struct Edge {
    var: u32,
    ptr: Rc<[Bdd]>,
}

#[derive(Default)]
pub struct RcStore {
    unique: HashSet<Rc<[Bdd]>>,
    or: HashMap<Vec<Bdd>, Bdd>,
    apply: HashMap<Bdd, Bdd>,
}

impl RcStore {
    fn unique(&mut self, val: Rc<[Bdd]>) -> Rc<[Bdd]> {
        self.unique.get_or_insert(val).clone()
    }

    fn or_cost(&mut self, slice: &[Bdd], var: u32, len: usize) -> usize {
        let mut total = 0;
        'val: for val in 0..len {
            let mut all_false = true;
            for mut zdd in slice {
                if let Bdd::Edge(edge) = zdd {
                    if edge.var == var {
                        zdd = &edge.ptr[val];
                    }
                }
                match zdd {
                    Bdd::Bool(true) => continue 'val,
                    Bdd::Bool(false) => {}
                    Bdd::Edge(_) => all_false = false,
                }
            }
            if !all_false {
                total += 1;
            }
        }
        total
    }

    fn or(&mut self, slice: Vec<Bdd>) -> Bdd {
        let mut edges = vec![];
        let mut vars = vec![];
        // if slice contains True, then return true.
        // if slice constains False, then remove it.
        for zdd in &*slice {
            match zdd {
                Bdd::Bool(true) => return Bdd::new(true),
                Bdd::Bool(false) => {}
                Bdd::Edge(edge) => {
                    vars.push((edge.var, edge.ptr.len()));
                    edges.push(zdd.clone());
                }
            }
        }
        edges.sort_unstable();
        edges.dedup();
        if let Some(res) = self.or.get(&edges) {
            return res.clone();
        }

        // find the most occuring variable
        vars.sort_unstable();
        vars.dedup();
        let res = if let Some((var, vals)) = vars.into_iter().max() {
            // change all to remove that variable
            let ptr = (0..vals)
                .map(|val| {
                    self.apply = HashMap::new(); // reset because we are going to apply something else
                    let filtered = edges.iter().map(|zdd| self.filter(zdd, var, val)).collect();
                    self.or(filtered)
                })
                .collect();
            let ptr = self.unique(ptr);
            Bdd::Edge(Edge { var, ptr })
        } else {
            Bdd::new(false)
        };

        self.or.try_insert(edges, res).unwrap().clone()
    }

    fn apply<F: FnMut(&mut Bdd)>(&mut self, old: &Bdd, f: &mut F) -> Bdd {
        if let Some(res) = self.apply.get(old) {
            return res.clone();
        }

        let mut new = old.clone();
        f(&mut new);
        if let Bdd::Edge(edge) = &mut new {
            let new = edge.ptr.iter().map(|zdd| self.apply(zdd, f)).collect();
            edge.ptr = self.unique(new)
        }

        self.apply.try_insert(old.clone(), new).unwrap().clone()
    }

    pub fn filter(&mut self, old: &Bdd, var: u32, val: usize) -> Bdd {
        self.apply(old, &mut |zdd| {
            if let Bdd::Edge(edge) = zdd {
                if edge.var == var {
                    *zdd = edge.ptr[val].clone();
                }
            }
        })
    }

    pub fn set(&mut self, old: &Bdd, var: u32, from: usize, to: usize, size: usize) -> Bdd {
        self.apply = HashMap::new();
        Bdd::Edge(Edge {
            var,
            ptr: (0..size)
                .map(|i| {
                    if i == to {
                        self.filter(old, var, from)
                    } else {
                        Bdd::new(false)
                    }
                })
                .collect(),
        })
    }
}

impl Bdd {
    pub fn new(val: bool) -> Self {
        Bdd::Bool(val)
    }

    pub fn or(slice: Vec<Bdd>) -> Bdd {
        RcStore::default().or(slice)
    }

    pub fn nodes(&self) -> usize {
        let mut total = 0;
        RcStore::default().apply(self, &mut |_| {
            total += 1;
        });
        total
    }
    // pub fn count(&self) -> usize {
    //     let mut store = RcStore::default();
    //     store.count(self)
    // }
}

impl Not for Bdd {
    type Output = Bdd;

    fn not(self) -> Self::Output {
        RcStore::default().apply(&self, &mut |zdd| {
            if let Bdd::Bool(val) = zdd {
                *val = !*val
            }
        })
    }
}

impl Debug for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Edge").field(&self.var).finish()
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.ptr, &other.ptr) && self.var == other.var
    }
}

impl PartialOrd for Edge {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.var.partial_cmp(&other.var) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.ptr.as_ptr().partial_cmp(&other.ptr.as_ptr())
    }
}

impl Ord for Edge {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.var.cmp(&other.var) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.ptr.as_ptr().cmp(&other.ptr.as_ptr())
    }
}

impl Hash for Edge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ptr::hash(Rc::as_ptr(&self.ptr), state);
        self.var.hash(state);
    }
}
