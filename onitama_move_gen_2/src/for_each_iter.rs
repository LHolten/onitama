use std::ops::ControlFlow;

pub trait ForEachIter {
    type Item<'a>;

    #[inline(always)]
    fn for_each<F>(mut self, mut f: F)
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
        Self: Sized,
        F: for<'a> FnMut(Self::Item<'a>) -> R,
        R: std::ops::Try<Output = ()>;

    #[inline(always)]
    fn fold<B, F>(self, init: B, mut f: F) -> B
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

    #[inline(always)]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
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

    fn std_iter(self) -> ForEachIterNewType<Self>
    where
        Self: Sized,
    {
        ForEachIterNewType(self)
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
    fn for_each<F>(self, f: F)
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
    fn fold<B, F>(self, init: B, f: F) -> B
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
