use std::{
    cmp::max,
    mem::{swap, take},
    rc::Rc,
};

use bumpalo::Bump;
use onitama_move_gen::{gen::Game, tablebase::TableBase};

#[derive(Default)]
pub struct Node<'a> {
    pub game: Game,
    pub guess: i8,
    pub depth: u8,
    pub nodes: Option<&'a mut [Node<'a>]>,
}

pub struct Agent {
    tablebase: Rc<TableBase>,
    bump: Bump,
}

impl Agent {
    pub fn new(tablebase: Rc<TableBase>) -> Self {
        Self {
            tablebase,
            bump: Bump::new(),
        }
    }

    pub fn copy<'a>(&'a self, node: &Node) -> Node<'a> {
        Node {
            game: node.game,
            guess: node.guess,
            depth: node.depth,
            nodes: node.nodes.as_ref().map(|nodes| {
                self.bump
                    .alloc_slice_fill_iter(nodes.iter().map(|new_node| self.copy(new_node)))
            }),
        }
    }

    #[inline(always)]
    pub fn new_node(&self, game: Game) -> Node {
        if game.is_loss() {
            Node {
                game,
                guess: -127,
                depth: 255,
                nodes: None,
            }
        } else {
            let (done, lower) = self.tablebase.eval(game);
            Node {
                game,
                guess: lower,
                depth: if done { 255 } else { 0 },
                nodes: None,
            }
        }
    }

    pub fn expand<'a>(&'a self, node: &mut Node<'a>) {
        node.nodes = Some(
            self.bump
                .alloc_slice_fill_iter(node.game.forward().map(|new_game| self.new_node(new_game))),
        );
    }

    pub fn bns<'a>(&'a self, node: &mut Node<'a>) {
        if node.nodes.is_none() {
            self.expand(node);
            node.depth = 0;
        }
        println!("depth: {}", node.depth);
        let mut ter_nodes = Vec::new();
        let mut bad_nodes = Vec::new();
        let mut new_nodes: Vec<Node> = node.nodes.as_mut().unwrap().iter_mut().map(take).collect();
        let mut lower = -127;
        let mut upper = 127;
        loop {
            // println!("* {}", node.guess);
            let beta = max(node.guess, lower + 1);
            let mut guess = -127;
            for mut new_node in take(&mut new_nodes) {
                let eval = -self.alpha_beta(&mut new_node, -beta + 1, node.depth);
                guess = max(guess, eval);
                if eval >= beta {
                    new_nodes.push(new_node)
                } else {
                    bad_nodes.push(new_node)
                }
            }
            if guess < beta {
                upper = guess
            } else {
                lower = guess
            }

            if new_nodes.len() == 1 || lower == upper {
                new_nodes.extend(bad_nodes.into_iter());
                new_nodes.extend(ter_nodes.into_iter());
                node.depth += 1;
                node.nodes = Some(self.bump.alloc_slice_fill_iter(new_nodes.into_iter()));
                break;
            } else if new_nodes.is_empty() {
                swap(&mut new_nodes, &mut bad_nodes);
            } else {
                ter_nodes.extend(bad_nodes.drain(..));
            }
            node.guess = guess;
        }
    }

    pub fn alpha_beta<'a>(&'a self, node: &mut Node<'a>, beta: i8, depth: u8) -> i8 {
        if depth == 0 || node.depth == 255 || node.depth == depth && node.guess >= beta {
            debug_assert!(node.guess != -128);
            return node.guess;
        }
        if node.nodes.is_none() {
            self.expand(node);
        }
        let nodes = node.nodes.as_mut().unwrap();
        let (first, rest) = nodes.split_first_mut().unwrap();
        node.guess = -self.alpha_beta(first, -beta + 1, depth - 1);
        if node.guess < beta {
            for new_node in rest {
                let eval = -self.alpha_beta(new_node, -beta + 1, depth - 1);
                node.guess = max(node.guess, eval);
                if eval >= beta {
                    swap(first, new_node);
                    break;
                }
            }
        }
        node.depth = depth;
        debug_assert!(node.guess != -128);
        node.guess
    }
}

#[cfg(test)]
mod test {
    use std::mem::size_of;

    use bumpalo::Bump;
    use onitama_move_gen::{eval::Eval, gen::Game, tablebase::TableBase};

    use crate::node::Node;

    #[test]
    fn try_forward_tie() {
        dbg!(Eval::new_tie().forward());
    }

    #[test]
    fn try_bump() {
        assert_eq!(size_of::<Game>(), 16);
        // let bump = Bump::new();
        // let my = bump.alloc_with(|| {
        //     let nodes = bump.alloc_slice_fill_default(1);
        //     Test { nodes: Some(nodes) }
        // });
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
