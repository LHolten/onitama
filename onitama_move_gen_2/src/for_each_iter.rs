use std::ops::ControlFlow;

pub trait ForEachIter {
    type Item;

    fn try_for_each<F, R>(&mut self, f: F) -> R
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: std::ops::Try<Output = ()>;

    fn std_iter(self) -> ForEachIterNewType<Self>
    where
        Self: Sized,
    {
        ForEachIterNewType(self)
    }
}

pub struct ForEachIterNewType<T>(T);

impl<T: ForEachIter> Iterator for ForEachIterNewType<T> {
    type Item = T::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.try_for_each(ControlFlow::Break).break_value()
    }

    #[inline(always)]
    fn for_each<F>(mut self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        self.try_for_each(|x| {
            f(x);
            Some(())
        });
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
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
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
