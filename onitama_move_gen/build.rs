extern crate build_const;
use build_const::ConstWriter;

#[allow(clippy::unusual_byte_groupings)]
const CARDS: [(&str, u32, u8); 16] = [
    ("ox", 0b00000_00100_00010_00100_00000, 0),
    ("boar", 0b00000_00100_01010_00000_00000, 1),
    ("horse", 0b00000_00100_01000_00100_00000, 1),
    ("elephant", 0b00000_01010_01010_00000_00000, 1),
    ("crab", 0b00000_00100_10001_00000_00000, 0),
    ("tiger", 0b00100_00000_00000_00100_00000, 0),
    ("monkey", 0b00000_01010_00000_01010_00000, 0),
    ("crane", 0b00000_00100_00000_01010_00000, 0),
    ("dragon", 0b00000_10001_00000_01010_00000, 1),
    ("mantis", 0b00000_01010_00000_00100_00000, 1),
    ("frog", 0b00000_01000_10000_00010_00000, 1),
    ("rabbit", 0b00000_00010_00001_01000_00000, 0),
    ("goose", 0b00000_01000_01010_00010_00000, 0),
    ("rooster", 0b00000_00010_01010_01000_00000, 1),
    ("eel", 0b00000_01000_00010_01000_00000, 0),
    ("cobra", 0b00000_00010_01000_00010_00000, 1),
];

fn shift(m: u32, pos: usize) -> u32 {
    #[allow(clippy::unusual_byte_groupings)]
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

fn main() {
    let mut shifted = [[0; 25]; 16];
    let mut shifted_r = [[0; 25]; 16];
    let mut shifted_l = [[0; 25]; 16];
    let mut shifted_u = [[0; 25]; 16];
    for card in 0..16 {
        let m = CARDS[card].1;
        let r = CARDS[card].1.reverse_bits() >> 7;
        for pos in 0..25 {
            let m = shift(m, pos);
            let r = shift(r, pos);
            shifted[card][pos] = m;
            shifted_r[card][pos] = r;
            shifted_l[card][pos] = m as u64;
            shifted_u[card][pos] = (m as u64) << 32;
        }
    }

    let consts = ConstWriter::for_build("lut").unwrap();
    let mut consts = consts.finish_dependencies();
    consts.add_value("SHIFTED", "[[u32; 25]; 16]", shifted);
    consts.add_value("SHIFTED_R", "[[u32; 25]; 16]", shifted_r);
    consts.add_value("SHIFTED_L", "[[u64; 25]; 16]", shifted_l);
    consts.add_value("SHIFTED_U", "[[u64; 25]; 16]", shifted_u);
    consts.finish();
}
