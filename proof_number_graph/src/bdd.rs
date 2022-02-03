use std::{
    array::from_fn,
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    mem::replace,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Deref, Not},
    ptr,
    rc::Rc,
};

pub type Op = fn(bool, bool) -> bool;

pub struct RcStore<S, T> {
    unique: HashMap<T, Bdd<T>>,
    compute: HashMap<[Bdd<T>; 2], Bdd<T>>,
    count: HashMap<Bdd<T>, usize>,
    next: S,
}

impl<S: Default, T> Default for RcStore<S, T> {
    fn default() -> Self {
        Self {
            unique: Default::default(),
            compute: Default::default(),
            count: Default::default(),
            next: Default::default(),
        }
    }
}

impl<S: Store<T>, T: DecisionDiagram, const N: usize> RcStore<S, [T; N]> {
    fn unique(&mut self, val: [T; N]) -> Bdd<[T; N]> {
        let rc = Bdd(Rc::new(val.clone()));
        self.unique.entry(val).or_insert(rc).clone()
    }
}

impl<S: Store<T>, T: DecisionDiagram, const N: usize> Store<Bdd<[T; N]>> for RcStore<S, [T; N]> {
    fn full(&mut self, val: bool) -> Bdd<[T; N]> {
        let next = self.next.full(val);
        self.unique(from_fn(|_| next.clone()))
    }

    fn compute(&mut self, f: Op, args: [Bdd<[T; N]>; 2]) -> Bdd<[T; N]> {
        if let Some(res) = self.compute.get(&args) {
            return res.clone();
        }
        let (a, b) = (args[0].as_ref().clone(), args[1].as_ref().clone());
        let res = a.zip(b).map(|(a, b)| self.next.compute(f, [a, b]));
        let rc = self.unique(res);
        self.compute.try_insert(args, rc).unwrap().clone()
    }

    fn set(&mut self, arg: Bdd<[T; N]>, index: usize, from: usize, to: usize) -> Bdd<[T; N]> {
        if let Some(res) = self.unique.get(arg.as_ref()) {
            return res.clone();
        }
        let mut new = arg.as_ref().clone();
        if index == 0 {
            new[to] = replace(&mut new[from], self.next.full(false));
        } else {
            new = new.map(|item| self.next.set(item, index - 1, from, to))
        }
        let rc = self.unique(new);
        self.unique.insert(arg.as_ref().clone(), rc.clone());
        rc
    }

    fn visit(&mut self, arg: Bdd<[T; N]>) {
        if self.unique.contains_key(arg.as_ref()) {
            return;
        }
        arg.iter().for_each(|t| self.next.visit(t.clone()));
        self.unique.insert(arg.as_ref().clone(), arg);
    }

    fn nodes(&self) -> usize {
        self.unique.len() + self.next.nodes()
    }

    fn count(&mut self, arg: &Bdd<[T; N]>) -> usize {
        if let Some(val) = self.count.get(arg) {
            return *val;
        }
        let val = arg.iter().map(|t| self.next.count(t)).sum();
        *self.count.try_insert(arg.clone(), val).unwrap()
    }
}

pub trait Store<T>: Default {
    fn full(&mut self, val: bool) -> T;
    fn compute(&mut self, f: Op, args: [T; 2]) -> T;
    fn set(&mut self, arg: T, index: usize, from: usize, to: usize) -> T;
    fn visit(&mut self, arg: T);
    fn nodes(&self) -> usize;
    fn count(&mut self, arg: &T) -> usize;
}

pub trait DecisionDiagram: Sized + Eq + Hash + Clone + Debug {
    type S: Store<Self>;

    fn full(val: bool) -> Self {
        Self::S::default().full(val)
    }
    fn set(self, index: usize, from: usize, to: usize) -> Self {
        Self::S::default().set(self, index, from, to)
    }
    fn nodes(&self) -> usize {
        let mut store = Self::S::default();
        store.visit(self.clone());
        store.nodes()
    }
    fn count(&self) -> usize {
        let mut store = Self::S::default();
        store.count(self)
    }
}

impl<T: DecisionDiagram, const C: usize> DecisionDiagram for Bdd<[T; C]> {
    type S = RcStore<T::S, [T; C]>;
}

#[derive(Default)]
pub struct BoolStore {}

impl DecisionDiagram for bool {
    type S = BoolStore;
}

impl Store<bool> for BoolStore {
    fn full(&mut self, val: bool) -> bool {
        val
    }

    fn compute(&mut self, f: Op, args: [bool; 2]) -> bool {
        f(args[0], args[1])
    }

    fn set(&mut self, _arg: bool, _index: usize, _from: usize, _to: usize) -> bool {
        unreachable!()
    }

    fn visit(&mut self, _arg: bool) {}

    fn nodes(&self) -> usize {
        0
    }

    fn count(&mut self, arg: &bool) -> usize {
        *arg as usize
    }
}

#[derive(Clone)]
pub struct Bdd<T>(Rc<T>);
impl<T> Debug for Bdd<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Bdd").finish()
    }
}

impl<T> Deref for Bdd<T> {
    type Target = Rc<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> PartialEq for Bdd<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(self, other)
    }
}

impl<T> Eq for Bdd<T> {}

impl<T> Hash for Bdd<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ptr::hash(Rc::as_ptr(self), state)
    }
}

impl<T> BitOr for Bdd<T>
where
    Self: DecisionDiagram,
{
    type Output = Bdd<T>;

    fn bitor(self, rhs: Self) -> Self::Output {
        let mut store = <Self as DecisionDiagram>::S::default();
        store.compute(BitOr::bitor, [self, rhs])
    }
}

impl<T> BitAnd for Bdd<T>
where
    Self: DecisionDiagram,
{
    type Output = Bdd<T>;

    fn bitand(self, rhs: Self) -> Self::Output {
        let mut store = <Self as DecisionDiagram>::S::default();
        store.compute(BitAnd::bitand, [self, rhs])
    }
}

impl<T> BitXor for Bdd<T>
where
    Self: DecisionDiagram,
{
    type Output = Bdd<T>;

    fn bitxor(self, rhs: Self) -> Self::Output {
        let mut store = <Self as DecisionDiagram>::S::default();
        store.compute(BitXor::bitxor, [self, rhs])
    }
}

impl<T> BitAndAssign for Bdd<T>
where
    Self: DecisionDiagram,
{
    fn bitand_assign(&mut self, other: Self) {
        *self = self.clone() & other
    }
}

impl<T> BitOrAssign for Bdd<T>
where
    Self: DecisionDiagram,
{
    fn bitor_assign(&mut self, other: Self) {
        *self = self.clone() | other
    }
}

impl<T> BitXorAssign for Bdd<T>
where
    Self: DecisionDiagram,
{
    fn bitxor_assign(&mut self, other: Self) {
        *self = self.clone() ^ other
    }
}

impl<T> Not for Bdd<T>
where
    Self: DecisionDiagram,
{
    type Output = Bdd<T>;

    fn not(self) -> Self::Output {
        self ^ Self::full(true)
    }
}
