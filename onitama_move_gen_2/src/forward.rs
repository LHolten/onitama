use std::{marker::PhantomData, ops::ControlFlow};

use crate::{
    for_each_iter::ForEachIter,
    player::Player,
    state::{Cards, State},
};

#[derive(Clone, Copy)]
struct BitIter(pub u32);

impl ForEachIter for BitIter {
    type Item = u32;

    fn try_for_each<F, R>(&mut self, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: std::ops::Try<Output = ()>,
    {
        while self.0 != 0 {
            let index = self.0.trailing_zeros();
            self.0 ^= 1 << index;
            f(index)?;
        }
        R::from_output(())
    }
}

impl BitIter {
    fn try_for_each_recycle<F, R>(&mut self, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(u32) -> R,
        R: std::ops::Try<Output = ()>,
    {
        match self.try_for_each(|index| f(index).branch().map_break(|r| (r, index))) {
            ControlFlow::Break((r, index)) => {
                self.0 ^= 1 << index; // add back index, we need it again
                R::from_residual(r)
            }
            _ => R::from_output(()),
        }
    }
}

struct Forward<P> {
    state: State,
    cards: BitIter,
    from: BitIter,
    to: BitIter,
    _player: PhantomData<P>,
}

impl<P: Player> Forward<P> {
    pub fn new(state: State) -> Self {
        Self {
            state,
            cards: if state.temple_is_attacked::<P>() {
                BitIter(0) // if the temple is attacked then all moves are losing
            } else {
                BitIter(state.cards.get::<P>())
            },
            from: BitIter(0),
            to: BitIter(0),
            _player: PhantomData,
        }
    }
}

impl<P: Player> ForEachIter for Forward<P> {
    type Item = State;

    fn try_for_each<F, R>(&mut self, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: std::ops::Try<Output = ()>,
    {
        let mut virtual_call = |state: State| {
            if state.king_is_attacked::<P>() {
                R::from_output(()) // we ignore states that have an attacked king
            } else {
                f(state)
            }
        };

        let king = self.state.pieces.king::<P>();
        self.cards.try_for_each_recycle(|card| {
            let mut state = self.state;
            state.cards.swap::<P>(card); // can be reused for all moves with this card

            reset(&mut self.from, || self.state.pieces.all::<P>());
            if self.from.0 & (1 << king) != 0 {
                reset(&mut self.to, || Cards::card::<P>(king, card));
                self.to.try_for_each(|to| {
                    let mut state = state;
                    state.pieces.move_all::<P>(king, to);
                    state.pieces.move_king::<P>(to);
                    virtual_call(state)
                })?;
                self.from.0 ^= 1 << king; // we do not want to move the king again
            }
            self.from.try_for_each_recycle(|from| {
                reset(&mut self.to, || Cards::card::<P>(from, card));
                self.to.try_for_each(|to| {
                    let mut state = state;
                    state.pieces.move_all::<P>(from, to);
                    virtual_call(state)
                })
            })
        })
    }
}

fn reset<F>(iter: &mut BitIter, f: F)
where
    F: FnOnce() -> u32,
{
    if iter.0 == 0 {
        *iter = BitIter(f())
    }
}
