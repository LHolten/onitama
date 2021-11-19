mod map;
mod wrap;

use std::ops::ControlFlow;

use crate::for_each_iter::{map::Map, wrap::Wrap};

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
    fn map<F>(self, f: F) -> Wrap<Map<Self, F>>
    where
        Self: Sized,
        F: for<'a> FnMut<(Self::Item<'a>,)>,
    {
        Map { iter: self, f }.wrap()
    }

    #[inline]
    fn all<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: for<'a> FnMut(Self::Item<'a>) -> bool,
    {
        self.try_fold((), |(), x| {
            if f(x) {
                ControlFlow::CONTINUE
            } else {
                ControlFlow::BREAK
            }
        }) == ControlFlow::CONTINUE
    }

    #[inline]
    fn any<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: for<'a> FnMut(Self::Item<'a>) -> bool,
    {
        self.try_fold((), |(), x| {
            if f(x) {
                ControlFlow::BREAK
            } else {
                ControlFlow::CONTINUE
            }
        }) == ControlFlow::BREAK
    }

    #[inline]
    fn wrap(self) -> Wrap<Self>
    where
        Self: Sized,
    {
        Wrap(self)
    }
}
