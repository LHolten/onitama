use std::{hint::unreachable_unchecked, mem::swap, ops::BitXorAssign};

use num::PrimInt;
use seq_macro::seq;

use crate::{
    card::{get_bitmap, offset_pieces, single_mask},
    for_each_iter::ForEachIter,
    side::Side,
    state::State,
};

#[derive(Clone, Copy, Default)]
pub struct BitIter<T>(pub T);

impl<T: PrimInt + BitXorAssign> ForEachIter for BitIter<T> {
    type Item<'a> = u32;

    // this implementation keeps values that were not succesfully processed
    // which is not "correct", but it is conventient when nesting loops
    fn try_for_each<F, R>(&mut self, mut f: F) -> R
    where
        Self: Sized,
        F: for<'a> FnMut(Self::Item<'a>) -> R,
        R: std::ops::Try<Output = ()>,
    {
        while self.0 != T::zero() {
            let index = self.0.trailing_zeros();
            f(index)?;
            self.0 ^= T::one() << index as usize;
        }
        R::from_output(())
    }
}

impl<S: Side> State<S> {
    // second param is threats
    fn go_all_pawns<F, R, const CARD: u32>(&mut self, mut f: F) -> R
    where
        F: for<'a> FnMut(&mut State<S::Other>) -> R,
        R: std::ops::Try<Output = ()>,
    {
        BitIter(get_bitmap::<S>(CARD)).try_for_each(|offset| {
            let mut piece_mask = offset_pieces(self.my_pawns(), offset);
            piece_mask &= !self.my_pawns() & !(1 << self.my_king());
            BitIter(piece_mask).try_for_each(|to| self.go_pawn(to + 14 - offset, to, CARD, &mut f))
        })
    }

    // all parameters are indices
    fn go_pawn<F, R>(&mut self, from: u32, to: u32, mut card: u32, mut f: F) -> R
    where
        F: for<'a> FnMut(&mut State<S::Other>) -> R,
        R: std::ops::Try<Output = ()>,
    {
        let opp_pawn_change = self.opp_pawns() & 1 << to;
        let my_pawn_change = 1 << from | 1 << to;
        let my_card_change = 1 << self.table | 1 << card;

        *S::Other::get_mut(&mut self.pawns) ^= opp_pawn_change;
        *S::get_mut(&mut self.pawns) ^= my_pawn_change;
        *S::get_mut(&mut self.cards) ^= my_card_change;
        swap(&mut self.table, &mut card);

        let res = f(self.flip());

        *S::Other::get_mut(&mut self.pawns) ^= opp_pawn_change;
        *S::get_mut(&mut self.pawns) ^= my_pawn_change;
        *S::get_mut(&mut self.cards) ^= my_card_change;
        swap(&mut self.table, &mut card);

        res
    }

    fn go_king<F, R>(&mut self, mut to: u32, mut card: u32, mut f: F) -> R
    where
        F: for<'a> FnMut(&mut State<S::Other>) -> R,
        R: std::ops::Try<Output = ()>,
    {
        let opp_pawn_change = self.opp_pawns() & 1 << to;

        *S::Other::get_mut(&mut self.pawns) ^= opp_pawn_change;
        swap(S::get_mut(&mut self.kings), &mut to);
        swap(&mut self.table, &mut card);

        let res = f(self.flip());

        *S::Other::get_mut(&mut self.pawns) ^= opp_pawn_change;
        swap(S::get_mut(&mut self.kings), &mut to);
        swap(&mut self.table, &mut card);

        res
    }
}

// we do not care about resuming!
impl<S: Side> ForEachIter for State<S> {
    type Item<'a> = &'a mut State<S::Other>;

    // this assumes it is not a win in 1
    fn try_for_each<F, R>(&mut self, mut f: F) -> R
    where
        F: for<'a> FnMut(Self::Item<'a>) -> R,
        R: std::ops::Try<Output = ()>,
    {
        if self.temple_threatened() {
            // this is not a win in one, so it is impossible to take the king
            return R::from_output(());
        }

        let threats = self.king_threats();
        self.my_cards().try_for_each(|card| {
            let mut king_mask = single_mask::<S>(card, self.my_king());
            king_mask &= !self.my_pawns() & !self.opp_attack();
            BitIter(king_mask).try_for_each(|to| self.go_king(to, card, &mut f))?;

            match threats.count_ones() {
                0 => {
                    seq!(C in 0..16 {
                        match card {
                            #(C => self.go_all_pawns::<_, R, C>(&mut f)?,)*
                            _ => unsafe { unreachable_unchecked() }
                        }
                    })
                }
                1 => self
                    .from_which_pawns(threats.trailing_zeros(), card)
                    .try_for_each(|from| {
                        self.go_pawn(from, threats.trailing_zeros(), card, &mut f)
                    })?,
                2.. => {}
            }

            R::from_output(())
        })?;

        R::from_output(())
    }
}
