use std::{
    mem::replace,
    ops::{ControlFlow, Deref, DerefMut},
};

use crate::for_each_iter::ForEachIter;

pub struct Wrap<T>(pub(crate) T);

impl<T> Deref for Wrap<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Wrap<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, I> Iterator for Wrap<T>
where
    for<'a> T: ForEachIter<Item<'a> = I>,
{
    type Item = I;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.try_for_each(ControlFlow::Break).break_value()
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
    fn try_for_each<F, R>(&mut self, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: std::ops::Try<Output = ()>,
    {
        let mut tmp = R::from_output(());
        self.0.try_for_each(|x| replace(&mut tmp, f(x)))
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
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> R,
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
}
