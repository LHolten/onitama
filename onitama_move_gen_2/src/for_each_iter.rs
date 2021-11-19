use std::ops::ControlFlow;

pub trait ForEachIter {
    type Item<'a>;

    #[inline]
    fn for_each<F>(&mut self, mut f: F)
    where
        Self: Sized,
        F: for<'a> FnMut(Self::Item<'a>),
    {
        self.try_for_each(|x| {
            f(x);
            Some(())
        });
    }

    fn try_for_each<F, R>(&mut self, f: F) -> R
    where
        F: for<'a> FnMut(Self::Item<'a>) -> R,
        R: std::ops::Try<Output = ()>;

    #[inline]
    fn fold<B, F>(&mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: for<'a> FnMut(B, Self::Item<'a>) -> B,
    {
        let mut accum = Some(init);
        self.for_each(|x| {
            accum = Some(f(accum.take().unwrap(), x));
        });
        accum.unwrap()
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        F: for<'a> FnMut(B, Self::Item<'a>) -> R,
        R: std::ops::Try<Output = B>,
    {
        let mut accum = Some(init);
        match self.try_for_each(|x| {
            accum = Some(f(accum.take().unwrap(), x).branch()?);
            ControlFlow::Continue(())
        }) {
            ControlFlow::Break(r) => R::from_residual(r),
            _ => R::from_output(accum.unwrap()),
        }
    }

    #[inline]
    fn map<F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: for<'a> FnMut<(Self::Item<'a>,)>,
    {
        Map { iter: self, f }
    }

    #[inline]
    fn std_iter(self) -> ForEachIterNewType<Self>
    where
        Self: Sized,
    {
        ForEachIterNewType(self)
    }
}

pub struct Map<I, F>
where
    I: ForEachIter,
    F: for<'a> FnMut<(I::Item<'a>,)>,
{
    iter: I,
    f: F,
}

impl<I, F> ForEachIter for Map<I, F>
where
    I: ForEachIter,
    F: for<'a> FnMut<(I::Item<'a>,)>,
{
    type Item<'a> = <F as FnOnce<(I::Item<'a>,)>>::Output;

    #[inline]
    fn try_for_each<G, R>(&mut self, mut f: G) -> R
    where
        Self: Sized,
        G: for<'a> FnMut(Self::Item<'a>) -> R,
        R: std::ops::Try<Output = ()>,
    {
        self.iter.try_for_each(|v| f((self.f)(v)))
    }
}

pub struct ForEachIterNewType<T>(T);

impl<T, I> Iterator for ForEachIterNewType<T>
where
    for<'a> T: ForEachIter<Item<'a> = I>,
{
    type Item = I;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        // also try_for_each might not remove items on break so this is impossible to implement
        unimplemented!("please use one of the specialised iterator methods")
    }

    #[inline(always)]
    fn for_each<F>(mut self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        self.0.for_each(f)
    }

    #[inline(always)]
    fn try_for_each<F, R>(&mut self, f: F) -> R
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: std::ops::Try<Output = ()>,
    {
        self.0.try_for_each(f)
    }

    #[inline(always)]
    fn fold<B, F>(mut self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.0.fold(init, f)
    }

    #[inline(always)]
    fn try_fold<B, F, R>(&mut self, init: B, f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> R,
        R: std::ops::Try<Output = B>,
    {
        self.0.try_fold(init, f)
    }
}

mod tests {
    use super::ForEachIter;
    use crate::{side::Left, state::State};

    #[test]
    pub fn test() {
        let id: for<'a> fn(&'a mut _) -> &'a mut _ = |x| x;
        State::<Left>::default().map(id).for_each(|s| {
            dbg!(s);
        })
    }
}
