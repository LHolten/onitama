use std::ops::{Deref, DerefMut};

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
