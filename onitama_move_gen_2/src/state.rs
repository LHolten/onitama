use std::fmt::Debug;
use std::{marker::PhantomData, ops::BitOr};

use crate::{
    card::{all_mask, single_mask},
    for_each_iter::{ForEachIter, ForEachIterNewType},
    forward::BitIter,
    side::Side,
};

pub struct State<S> {
    pub pawns: (u32, u32),
    pub kings: (u32, u32), // kings are stored as indices
    pub cards: (u16, u16),
    pub table: u32, // this is also an index
    pub side: PhantomData<S>,
}

impl<S: Side> State<S> {
    pub fn my_king(&self) -> u32 {
        S::get(self.kings)
    }

    pub fn opp_king(&self) -> u32 {
        S::Other::get(self.kings)
    }

    pub fn my_pawns(&self) -> u32 {
        S::get(self.pawns)
    }

    pub fn opp_pawns(&self) -> u32 {
        S::Other::get(self.pawns)
    }

    pub fn my_cards(&self) -> BitIter<u16> {
        BitIter(S::get(self.cards))
    }

    pub fn opp_cards(&self) -> ForEachIterNewType<BitIter<u16>> {
        BitIter(S::Other::get(self.cards)).std_iter()
    }

    pub fn opp_attack(&self) -> u32 {
        let opp_all = self.opp_pawns() | 1 << self.opp_king();
        self.opp_cards()
            .map(|card| all_mask::<S::Other>(opp_all, card))
            .fold(0, BitOr::bitor)
    }

    pub fn king_threats(&self) -> u32 {
        let mut mask = 0;
        self.opp_cards()
            .for_each(|card| mask |= single_mask::<S>(card, self.my_king()));
        mask & (self.opp_pawns() | 1 << self.opp_king())
    }

    pub fn temple_threatened(&self) -> bool {
        (self.opp_pawns() & S::temple() == 0)
            && self
                .opp_cards()
                .any(|card| single_mask::<S::Other>(card, self.opp_king()) & S::temple() != 0)
    }

    pub fn from_which_pawns(&self, to: u32, card: u32) -> BitIter<u32> {
        let mask = single_mask::<S::Other>(card, to);
        BitIter(mask & self.my_pawns())
    }

    pub fn flip(&mut self) -> &mut State<S::Other> {
        unsafe { &mut *(self as *mut Self as *mut State<S::Other>) }
    }
}

impl<S> Debug for State<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..5 {
            f.write_str("\n")?;
            for x in 0..5 {
                let pos = y * 6 + x;
                if self.pawns.0 & 1 << pos != 0 {
                    f.write_str("x")?;
                } else if self.pawns.1 & 1 << pos != 0 {
                    f.write_str("o")?;
                } else if pos == self.kings.0 {
                    f.write_str("X")?;
                } else if pos == self.kings.1 {
                    f.write_str("O")?;
                } else {
                    f.write_str(".")?;
                }
            }
        }
        Ok(())
    }
}

impl<S> Default for State<S> {
    fn default() -> Self {
        #[allow(clippy::unusual_byte_groupings)]
        Self {
            pawns: (
                0b10000_0_10000_0_00000_0_10000_0_10000,
                0b00001_0_00001_0_00000_0_00001_0_00001,
            ),
            kings: (16, 12),
            cards: (0b11, 0b1100),
            table: 4,
            side: Default::default(),
        }
    }
}
