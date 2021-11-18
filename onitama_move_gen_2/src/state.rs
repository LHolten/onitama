use std::{hint::unreachable_unchecked, marker::PhantomData, ops::BitOr};

use seq_macro::seq;

use crate::{
    for_each_iter::{ForEachIter, ForEachIterNewType},
    forward::BitIter,
    side::Side,
};

#[derive(Default)]
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

pub fn offset_pieces(from: u32, offset: u32) -> u32 {
    let mut new = if offset > 14 {
        from << (offset - 14)
    } else {
        from >> (14 - offset)
    };
    if offset == 16 {
        new &= BOARD_MASK << 1;
    } else if offset == 12 {
        new &= BOARD_MASK >> 1;
    }
    new
}

pub fn single_mask<S: Side>(card: u32, from: u32) -> u32 {
    let bitmap = get_bitmap::<S>(card);
    let mask = ((bitmap as u64) << from) >> 14;
    todo!("fix this for tiger somehow");
    mask as u32 & BOARD_MASK
}

#[allow(clippy::unusual_byte_groupings)]
const BOARD_MASK: u32 = 0b11111_0_11111_0_11111_0_11111_0_11111;

fn all_mask_const<S: Side, const CARD: u32>(from: u32) -> u32 {
    let bitmap = get_bitmap::<S>(CARD);

    let mut mask = 0;
    for offset in BitIter(bitmap).std_iter() {
        mask |= offset_pieces(from, offset);
    }
    mask & BOARD_MASK
}

pub fn all_mask<S: Side>(from: u32, card: u32) -> u32 {
    seq!(C in 0..16 {
        match card {
            #(C => all_mask_const::<S, C>(from),)*
            _ => unsafe { unreachable_unchecked() }
        }
    })
}

pub fn get_bitmap<S: Side>(card: u32) -> u32 {
    #[allow(clippy::unusual_byte_groupings)]
    const CARD_MAP: [u32; 16] = [
        0b00000_0_00000_0_01001_0_00000_0_00000,
        0b00100_0_00000_0_00010_0_00000_0_00100,
        0b00000_0_01010_0_00000_0_01010_0_00000,
        0b00000_0_01000_0_00010_0_01000_0_00000,
        0b00010_0_01000_0_00000_0_01000_0_00010,
        0b00000_0_00110_0_00000_0_00110_0_00000,
        0b00000_0_00010_0_01000_0_00010_0_00000,
        0b00000_0_00100_0_00010_0_00100_0_00000,
        0b00100_0_00010_0_00000_0_01000_0_00000,
        0b00000_0_00110_0_00000_0_01100_0_00000,
        0b00000_0_00100_0_01010_0_00000_0_00000,
        0b00000_0_01010_0_00000_0_00100_0_00000,
        0b00000_0_01000_0_00000_0_00010_0_00100,
        0b00000_0_01100_0_00000_0_00110_0_00000,
        0b00000_0_00000_0_01010_0_00100_0_00000,
        0b00000_0_00100_0_00000_0_01010_0_00000,
    ];
    let bitmap = CARD_MAP[card as usize];
    S::get((bitmap, reverse_bitmap(bitmap)))
}

pub fn reverse_bitmap(board: u32) -> u32 {
    board.reverse_bits() >> 3
}
