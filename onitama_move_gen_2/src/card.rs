use std::hint::unreachable_unchecked;

use seq_macro::seq;

use crate::{for_each_iter::ForEachIter, forward::BitIter, side::Side};

#[inline]
pub(crate) fn offset_pieces(from: u32, offset: u32) -> u32 {
    let mut new = ((from as u64) << offset >> 14) as u32;
    if offset == 16 {
        new &= BOARD_MASK << 1;
    } else if offset == 12 {
        new &= BOARD_MASK >> 1;
    }
    new & BOARD_MASK
}

#[inline]
#[allow(clippy::erasing_op, clippy::eq_op)]
pub(crate) fn single_mask<S: Side>(card: u32, from: u32) -> u32 {
    fn inner<S: Side>(card: u32, from: u32) -> u32 {
        let bitmap = get_bitmap::<S>(card);
        let mut mask = (((bitmap as u64) << from) >> 14) as u32;
        if card == 0 && from % 6 >= 3 {
            mask &= BOARD_MASK << 1
        } else if card == 0 {
            mask &= BOARD_MASK >> 1
        }
        mask & BOARD_MASK
    }

    seq!(V in 0..464 {
        match (from << 4) + card {
            #(V => inner::<S>(V & 0b1111, V >> 4),)*
            _ => unsafe { unreachable_unchecked() }
        }
    })
}

#[allow(clippy::unusual_byte_groupings)]
const BOARD_MASK: u32 = 0b11111_0_11111_0_11111_0_11111_0_11111;

pub fn all_mask<S: Side>(from: u32, card: u32) -> u32 {
    #[inline]
    fn inner<S: Side, const CARD: u32>(from: u32) -> u32 {
        let bitmap = get_bitmap::<S>(CARD);

        let mut mask = 0;
        BitIter(bitmap).for_each(|offset| {
            mask |= offset_pieces(from, offset);
        });
        mask
    }

    seq!(C in 0..16 {
        match card {
            #(C => inner::<S, C>,)*
            _ => unsafe { unreachable_unchecked() }
        }
    })(from)
}

pub(crate) fn get_bitmap<S: Side>(card: u32) -> u32 {
    #[allow(clippy::unusual_byte_groupings)]
    const CARD_MAP_LEFT: [u32; 16] = [
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
    const CARD_MAP_RIGHT: [u32; 16] = seq!(C in 0..16 {
        [
            #(reverse_bitmap(CARD_MAP_LEFT[C]),)*
        ]
    });
    S::get((CARD_MAP_LEFT, CARD_MAP_RIGHT))[card as usize]
}

#[inline]
const fn reverse_bitmap(board: u32) -> u32 {
    board.reverse_bits() >> 3
}
