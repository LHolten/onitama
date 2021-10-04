struct Black;
struct White;

pub trait Player {
    type Other: Player;
    fn choose<P>(black: P, white: P) -> P;
    fn temple() -> u32;
}

impl Player for Black {
    type Other = White;
    fn choose<P>(black: P, _white: P) -> P {
        black
    }

    fn temple() -> u32 {
        1 << 22
    }
}

impl Player for White {
    type Other = Black;
    fn choose<P>(_black: P, white: P) -> P {
        white
    }

    fn temple() -> u32 {
        1 << 2
    }
}
