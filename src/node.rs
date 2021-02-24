use std::{
    cmp::max,
    mem::{replace, swap},
};

use arrayvec::ArrayVec;
use onitama_move_gen::{gen::Game, tablebase::TableBase};

#[derive(Default)]
pub struct Node {
    pub game: Game,
    pub guess: i8,
    pub depth: u8,
    pub nodes: Option<Box<ArrayVec<[Node; 40]>>>,
}

pub struct Agent(pub Box<TableBase>);

impl Agent {
    #[inline(always)]
    pub fn new_node(&mut self, game: Game) -> Node {
        if game.is_loss() {
            Node {
                game,
                guess: -127,
                depth: 255,
                nodes: None,
            }
        } else {
            let (done, lower) = self.0.eval(game);
            Node {
                game,
                guess: lower,
                depth: if done { 255 } else { 0 },
                nodes: None,
            }
        }
    }

    pub fn bns(&mut self, node: &mut Node) {
        if node.nodes.is_none() {
            let mut nodes = Box::new(ArrayVec::new());
            for new_game in node.game.forward() {
                unsafe { nodes.push_unchecked(self.new_node(new_game)) };
            }
            node.nodes = Some(nodes);
            node.depth = 0;
        }
        println!("depth: {}", node.depth);
        let nodes = node.nodes.as_mut().unwrap().as_mut();
        let mut ter_nodes: ArrayVec<[Node; 40]> = ArrayVec::new();
        let mut bad_nodes: ArrayVec<[Node; 40]> = ArrayVec::new();
        let mut new_nodes = replace(nodes, ArrayVec::new());
        let mut lower = -127;
        let mut upper = 127;
        loop {
            // println!("* {}", node.guess);
            let beta = max(node.guess, lower + 1);
            let mut guess = -127;
            for mut new_node in new_nodes.drain(..) {
                let eval = -self.alpha_beta(&mut new_node, -beta + 1, node.depth);
                guess = max(guess, eval);
                if eval >= beta {
                    nodes.push(new_node)
                } else {
                    bad_nodes.push(new_node)
                }
            }
            if guess < beta {
                upper = guess
            } else {
                lower = guess
            }

            if nodes.len() == 1 || lower == upper {
                nodes.extend(bad_nodes.into_iter());
                nodes.extend(ter_nodes.into_iter());
                // println!(
                //     "{:?}",
                //     node.nodes.iter().map(|n| n.guess).collect::<Vec<i8>>()
                // );
                node.depth += 1;
                break;
            } else if nodes.is_empty() {
                swap(&mut new_nodes, &mut bad_nodes);
            } else {
                swap(&mut new_nodes, nodes);
                ter_nodes.extend(bad_nodes.drain(..));
            }
            node.guess = guess;
        }
    }

    #[inline(never)]
    pub fn alpha_beta(&mut self, node: &mut Node, beta: i8, depth: u8) -> i8 {
        if depth == 0 || node.depth == 255 || node.depth == depth && node.guess >= beta {
            debug_assert!(node.guess != -128);
            return node.guess;
        }
        if node.nodes.is_none() {
            debug_assert!(!node.game.is_loss());
            let mut nodes = Box::new(ArrayVec::new());
            // for new_game in node.game.forward() {
            //     unsafe { nodes.push_unchecked(self.new_node(new_game)) };
            // }
            node.nodes = Some(nodes);
            node.depth = 0;
        }
        // let nodes = node.nodes.as_mut().unwrap().as_mut();
        // let (first, rest) = nodes.split_first_mut().unwrap();
        // node.guess = -self.alpha_beta(first, -beta + 1, depth - 1);
        // if node.guess < beta {
        //     for new_node in rest {
        //         let eval = -self.alpha_beta(new_node, -beta + 1, depth - 1);
        //         node.guess = max(node.guess, eval);
        //         if eval >= beta {
        //             swap(first, new_node);
        //             break;
        //         }
        //     }
        // }
        node.depth = depth;
        debug_assert!(node.guess != -128);
        node.guess
    }
}

#[cfg(test)]
mod test {
    use onitama_move_gen::{eval::Eval, tablebase::TableBase};

    use crate::node::Node;

    use super::Agent;

    #[test]
    fn try_forward_tie() {
        dbg!(Eval::new_tie().forward());
    }

    // #[test]
    // fn test_see_win() {
    //     let mut node = Node {
    //         depth: 1,
    //         nodes: vec![
    //             Node {
    //                 depth: 255,
    //                 guess: 0,
    //                 ..Default::default()
    //             },
    //             Node {
    //                 depth: 255,
    //                 guess: -127,
    //                 ..Default::default()
    //             },
    //         ],
    //         ..Default::default()
    //     };
    //     let mut agent = Agent(TableBase::empty());
    //     assert_eq!(agent.alpha_beta(&mut node, 1, 1), 127)
    // }

    // #[test]
    // fn test_see_win_in_two() {
    //     let mut node = Node {
    //         depth: 1,
    //         guess: -127,
    //         nodes: vec![
    //             // Node {
    //             //     depth: 255,
    //             //     ..Default::default()
    //             // },
    //             Node {
    //                 depth: 1,
    //                 guess: -127,
    //                 nodes: vec![
    //                     // Node {
    //                     //     depth: 255,
    //                     //     ..Default::default()
    //                     // },
    //                     Node {
    //                         depth: 255,
    //                         guess: 127,
    //                         ..Default::default()
    //                     },
    //                 ],
    //                 ..Default::default()
    //             },
    //         ],
    //         ..Default::default()
    //     };
    //     let mut agent = Agent(TableBase::empty());
    //     assert_eq!(agent.alpha_beta(&mut node, 1, 2), 127)
    // }
}
