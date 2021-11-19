use crate::for_each_iter::ForEachIter;

pub struct Map<I, F>
where
    I: ForEachIter,
    F: for<'a> FnMut<(I::Item<'a>,)>,
{
    pub(crate) iter: I,
    pub(crate) f: F,
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

mod tests {
    use super::*;
    use crate::{side::Left, state::State};

    #[test]
    pub fn test() {
        let id: for<'a> fn(&'a mut _) -> &'a mut _ = |x| x;
        State::<Left>::default().map(id).for_each(|s| {
            dbg!(s);
        })
    }
}
