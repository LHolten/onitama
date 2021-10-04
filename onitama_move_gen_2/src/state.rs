use crate::player::Player;

#[derive(Clone, Copy)]
pub struct Pieces(u32, u32);

#[derive(Clone, Copy)]
pub struct Cards(u16, u16, u16);

impl Pieces {
    const PAWN: u32 = (1 << 25) - 1;

    fn get_mut<P: Player>(&mut self) -> &mut u32 {
        P::choose(&mut self.0, &mut self.1)
    }

    fn get<P: Player>(self) -> u32 {
        P::choose(self.0, self.1)
    }

    // returns a mask
    pub fn all<P: Player>(self) -> u32 {
        self.get::<P>() & Self::PAWN
    }

    // returns a coordinate
    pub fn king<P: Player>(self) -> u32 {
        self.get::<P>().wrapping_shr(25)
    }

    pub fn move_all<P: Player>(&mut self, from: u32, to: u32) {
        *self.get_mut::<P::Other>() &= !(1 << to);
        *self.get_mut::<P>() ^= (1 << from) ^ (1 << to);
    }

    pub fn move_king<P: Player>(&mut self, to: u32) {
        *self.get_mut::<P::Other>() &= !(1 << to);
        *self.get_mut::<P>() &= Self::PAWN;
        *self.get_mut::<P>() ^= to << 25;
    }

    // returns a coordinate
    pub fn temple<P: Player>() -> u32 {
        P::choose(2, 22)
    }
}

impl Cards {
    pub fn get<P: Player>(self) -> u32 {
        P::choose(self.0, self.1) as u32
    }

    pub fn table<P: Player>(self) -> u16 {
        self.2
    }

    pub fn swap<P: Player>(&mut self, card: u32) {
        self.2 ^= 1 << card;
        *P::choose(&mut self.0, &mut self.1) ^= self.2
    }

    // use combined cards but flipped
    pub fn attacked_by<P: Player>(self, target: u32) -> u32 {
        todo!()
    }

    pub fn card<P: Player>(from: u32, card: u32) -> u32 {
        todo!()
    }
}

#[derive(Clone, Copy)]
pub struct State {
    pub pieces: Pieces,
    pub cards: Cards,
}

impl State {
    pub fn king_is_attacked<P: Player>(self) -> bool {
        let mask = self.cards.attacked_by::<P>(self.pieces.king::<P>());
        mask & self.pieces.all::<P::Other>() != 0
    }

    pub fn temple_is_attacked<P: Player>(self) -> bool {
        let mask = self.cards.attacked_by::<P>(Pieces::temple::<P>());
        mask & (1 << self.pieces.king::<P::Other>()) != 0
    }
}
