use core::panic;

use onitama_move_gen::{
    gen::{Game, PIECE_MASK},
    ops::{BitIter, CardIter},
};
use onitama_move_gen::{CARD_HASH, KING_HASH, NAMES, PIECE_HASH};

#[derive(Debug, Deserialize)]
#[serde(tag = "messageType")]
pub enum LitamaMsg {
    #[serde(rename = "create")]
    Create(CreateMsg),
    #[serde(rename = "join")]
    Join(JoinMsg),
    #[serde(rename = "state")]
    State(StateMsg),
    #[serde(rename = "move")]
    Move,
    #[serde(rename = "spectate")]
    Spectate,
    #[serde(rename = "error")]
    Error(ErrorMsg),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMsg {
    pub match_id: String,
    pub token: String,
    pub index: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinMsg {
    pub token: String,
    pub index: u8,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "gameState")]
pub enum StateMsg {
    #[serde(rename = "waiting for player")]
    WaitingForPlayers,
    #[serde(rename = "in progress")]
    InProgress(StateObj),
    #[serde(rename = "ended")]
    Ended,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StateObj {
    pub current_turn: String,
    pub cards: CardsObj,
    pub indices: IndicesObj,
    pub board: String,
}

impl StateObj {
    pub fn index(&self) -> u8 {
        if self.current_turn == "red" {
            self.indices.red
        } else {
            self.indices.blue
        }
    }

    pub fn game(&self) -> Game {
        let mut red = 0;
        let mut blue = 0;
        for (i, c) in self.board.chars().enumerate() {
            let p = (i + 4 - 2 * (i % 5)) as u32;
            match c {
                '0' => {}
                '1' => blue |= 1 << p,
                '2' => blue |= 1 << p | p << 25,
                '3' => red |= 1 << (24 - p),
                '4' => red |= 1 << (24 - p) | (24 - p) << 25,
                u => panic!(&format!("unexpected char: {}", u)),
            }
        }
        let red_cards = Self::cards(&self.cards.red);
        let blue_cards = Self::cards(&self.cards.blue);
        let table = Self::card_id(&self.cards.side);
        let red_hash = Self::hash(red, red_cards);
        let blue_hash = Self::hash(blue, blue_cards);
        if self.current_turn == "red" {
            Game {
                my: red,
                other: blue,
                cards: red_cards | blue_cards << 16,
                table,
                hash: red_hash ^ blue_hash.swap_bytes(),
            }
        } else {
            Game {
                my: blue,
                other: red,
                cards: blue_cards | red_cards << 16,
                table,
                hash: blue_hash ^ red_hash.swap_bytes(),
            }
        }
    }

    pub fn all_cards(&self) -> [u32; 5] {
        [
            Self::card_id(&self.cards.blue[0]),
            Self::card_id(&self.cards.blue[1]),
            Self::card_id(&self.cards.red[0]),
            Self::card_id(&self.cards.red[1]),
            Self::card_id(&self.cards.side),
        ]
    }

    fn cards(val: &[String; 2]) -> u32 {
        let mut cards = 0;
        for name in val {
            cards |= 1 << Self::card_id(name);
        }
        cards
    }

    fn card_id(card: &str) -> u32 {
        for (i, name) in NAMES.iter().enumerate() {
            if name == &card {
                return i as u32;
            }
        }
        panic!(&format!("card not found: {}", card))
    }

    fn hash(board: u32, cards: u32) -> u32 {
        let mut hash = 0;
        for pos in BitIter(board & PIECE_MASK) {
            hash ^= PIECE_HASH[pos as usize];
        }
        hash ^= KING_HASH[board.wrapping_shr(25) as usize];
        for card in CardIter::new(cards) {
            hash ^= CARD_HASH[card as usize];
        }
        hash
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CardsObj {
    pub red: [String; 2],
    pub blue: [String; 2],
    pub side: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IndicesObj {
    pub red: u8,
    pub blue: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorMsg {
    pub error: String,
    pub command: String,
}

pub fn translate_pos(pos: usize, flip: bool) -> String {
    let pos = if flip { 24 - pos } else { pos };
    let row = pos / 5;
    let col = pos % 5;
    [
        "edcba".chars().nth(col).unwrap(),
        "12345".chars().nth(row).unwrap(),
    ]
    .iter()
    .collect::<String>()
}

pub fn move_to_command(
    game: Game,
    new_game: Game,
    match_id: &str,
    token: &str,
    flip: bool,
) -> String {
    let mut command = String::from("move ");
    // match id
    command.push_str(match_id);
    command.push(' ');
    // token
    command.push_str(token);
    command.push(' ');
    // card
    command.push_str(NAMES[new_game.table as usize]);
    command.push(' ');
    // from:to
    let from = game.my & !new_game.other;
    let to = new_game.other & !game.my;
    command.push_str(&translate_pos(from.trailing_zeros() as usize, flip));
    command.push_str(&translate_pos(to.trailing_zeros() as usize, flip));
    println!("{}", command);
    command
}
