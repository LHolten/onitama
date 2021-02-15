use bitintr::Popcnt;

use crate::{
    eval::Eval,
    gen::{Game, PIECE_MASK},
    ops::{BitIter, CardIter},
};

#[derive(Default)]
struct OppTable {
    eval: Option<Eval>,
    data: [Option<Box<OppTable>>; 25],
}

#[derive(Default)]
struct MyTable {
    opp_table: OppTable,
    data: [Option<Box<MyTable>>; 25],
}

#[derive(Default)]
struct TableBase {
    data: [[[MyTable; 25]; 25]; 30],
}

impl TableBase {
    fn lookup(&self, game: &Game) -> Option<Eval> {
        let my_king = game.my.wrapping_shr(25);
        let other_king = game.other.wrapping_shr(25); // not rotated
        let cards = compress_cards(game.my_cards, game.other_cards);
        let mut my_table = &self.data[cards as usize][my_king as usize][other_king as usize];
        for pos in BitIter(game.my & PIECE_MASK ^ 1 << my_king) {
            my_table = my_table.data[pos as usize].as_ref()?
        }
        let mut opp_table = &my_table.opp_table;
        for pos in BitIter(game.other & PIECE_MASK ^ 1 << other_king) {
            opp_table = opp_table.data[pos as usize].as_ref()?
        }
        opp_table.eval
    }

    fn store(&mut self, game: &Game, eval: Eval) {
        let my_king = game.my.wrapping_shr(25);
        let other_king = game.other.wrapping_shr(25); // not rotated
        let cards = compress_cards(game.my_cards, game.other_cards);
        let mut my_table = &mut self.data[cards as usize][my_king as usize][other_king as usize];
        for pos in BitIter(game.my & PIECE_MASK ^ 1 << my_king) {
            if my_table.data[pos as usize].is_none() {
                my_table.data[pos as usize] = Some(Box::new(MyTable::default()))
            }
            my_table = my_table.data[pos as usize].as_mut().unwrap()
        }
        let mut opp_table = &mut my_table.opp_table;
        for pos in BitIter(game.other & PIECE_MASK ^ 1 << other_king) {
            if opp_table.data[pos as usize].is_none() {
                opp_table.data[pos as usize] = Some(Box::new(OppTable::default()))
            }
            opp_table = opp_table.data[pos as usize].as_mut().unwrap()
        }
        opp_table.eval = Some(eval);
    }

    fn new() -> Self {
        let mut table = TableBase::default();

        for pos in 0..25u32 {
            if pos == 22 || pos == 2 {
                continue;
            };
            for (my_cards, other_cards) in card_config() {
                let game = Game {
                    my: 1 << pos | pos.wrapping_shl(25),
                    other: 1 << 22 | 22u32.wrapping_shl(25),
                    my_cards,
                    other_cards,
                };
                for (prev, prev2) in game.backwards() {
                    table.update(prev);
                    table.update(prev2);
                }
            }
        }
        table
    }

    fn update(&mut self, game: Game) {
        if self.lookup(&game).is_some() {
            todo!()
        }
    }
}

fn compress_cards(my_cards: u8, other_cards: u8) -> u8 {
    let mut total = 0;
    for card in CardIter::new(my_cards) {
        total = total * 5 + card
    }
    if total >= 10 {
        total = 19 - total
    };
    total + 10 * ((!(my_cards | other_cards) - 1) & other_cards).popcnt()
}

pub fn card_config() -> impl Iterator<Item = (u8, u8)> {
    (0..5)
        .flat_map(|center| {
            (0..4).flat_map(move |my1| ((my1 + 1)..5).map(move |my2| (center, my1, my2)))
        })
        .filter_map(|(center, my1, my2)| {
            if center != my1 && center != my2 {
                let my_cards = 1 << my1 | 1 << my2;
                let other_cards = !my_cards ^ (1 << center);
                Some((my_cards, other_cards))
            } else {
                None
            }
        })
}

// #[cfg(test)]
// mod tests {
//     // Note this useful idiom: importing names from outer (for mod tests) scope.
//     use std::thread;

//     use super::*;

//     #[test]
//     fn test_eval() {
//         thread::Builder::new()
//             .stack_size(1024 * 1024 * 10)
//             .spawn(|| {
//                 let mut tie = 0;
//                 let mut loss = 0;
//                 let mut win = 0;

//                 let mut table = Box::new([None; 18750]);
//                 for game in game_config() {
//                     let new_index = SmallTable::compress(&game);
//                     table[new_index] = Some(Eval::Tie);
//                     let new_eval = table.eval(&game);
//                     table[new_index] = Some(new_eval);
//                     match new_eval {
//                         Eval::Win => win += 1,
//                         Eval::Loss => loss += 1,
//                         Eval::Tie => tie += 1,
//                     }
//                 }

//                 println!("win: {}, loss: {}, tie: {}", win, loss, tie);

//                 for game in game_config() {
//                     let new_index = SmallTable::compress(&game);
//                     if let Some(eval) = table[new_index] {
//                         assert_eq!(eval, table.eval(&game))
//                     } else {
//                         unreachable!()
//                     }
//                 }
//             })
//             .unwrap()
//             .join()
//             .unwrap();
//     }

//     #[test]
//     fn test_compress() {
//         let mut table = Box::new([false; 18750]);
//         let mut num = 0;
//         for game in game_config() {
//             let new_index = SmallTable::compress(&game);
//             assert!(!table[new_index]);
//             table[new_index] = true;
//             num += 1;
//         }
//         println!("{}", num)
//     }
// }
