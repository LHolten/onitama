extern crate build_const;
use build_const::ConstWriter;

const CARDS: [(&str, u32, u8); 16] = [
    ("tiger", 0b00100_00000_00000_00100_00000, 0),
    ("crab", 0b00000_00100_10001_00000_00000, 0),
    ("monkey", 0b00000_01010_00000_01010_00000, 0),
    ("crane", 0b00000_00100_00000_01010_00000, 0),
    ("dragon", 0b00000_10001_00000_01010_00000, 1),
    ("elephant", 0b00000_01010_01010_00000_00000, 1),
    ("mantis", 0b00000_01010_00000_00100_00000, 1),
    ("boar", 0b00000_00100_01010_00000_00000, 1),
    ("frog", 0b00000_01000_10000_00010_00000, 1),
    ("rabbit", 0b00000_00010_00001_01000_00000, 0),
    ("goose", 0b00000_01000_01010_00010_00000, 0),
    ("rooster", 0b00000_00010_01010_01000_00000, 1),
    ("horse", 0b00000_00100_01000_00100_00000, 1),
    ("ox", 0b00000_00100_00010_00100_00000, 0),
    ("eel", 0b00000_01000_00010_01000_00000, 0),
    ("cobra", 0b00000_00010_01000_00010_00000, 1),
];

fn shift(m: u32, pos: usize) -> u32 {
    const MASK: [u32; 5] = [
        0b00111_00111_00111_00111_00111,
        0b01111_01111_01111_01111_01111,
        0b11111_11111_11111_11111_11111,
        0b11110_11110_11110_11110_11110,
        0b11100_11100_11100_11100_11100,
    ];

    let shifted = (m as u64).wrapping_shl(pos as u32).wrapping_shr(12) as u32;
    shifted & MASK[pos % 5]
}

fn reverse(card: u32) -> u32 {
    card.reverse_bits() >> (32 - 25)
}

fn main() {
    let mut shifted = [[[0; 25]; 2]; 16];
    for card in 0..16 {
        let m = CARDS[card].1;
        for player in 0..2 {
            let m = if player == 0 { m } else { reverse(m) };
            for pos in 0..25 {
                let m = shift(m, pos);
                shifted[card][player][pos] = m;
            }
        }
    }

    let consts = ConstWriter::for_build("lut").unwrap();
    let mut consts = consts.finish_dependencies();
    consts.add_value("SHIFTED", "[[[u32; 25]; 2]; 16]", shifted);
    consts.finish();
}
