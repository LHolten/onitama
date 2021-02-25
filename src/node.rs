use std::ptr::{self, NonNull};
use std::{alloc::Layout, cmp::max, mem::swap, rc::Rc, slice, unreachable};

use bumpalo::Bump;
use onitama_move_gen::{gen::Game, tablebase::TableBase};

#[derive(Clone, Copy, Default)]
pub struct Leaf {
    table: bool,
    value: i8,
    child: u8,
    game: Game,
}

pub struct Branch<'a> {
    lower: i8,
    upper: i8,
    depth: u8,
    child: u8,
    nodes: &'a mut [Node<'a>],
}

pub enum Node<'a> {
    Leaf(Leaf),
    Branch(Branch<'a>),
}

impl<'a> Node<'a> {
    pub fn is_child(&self, child: u8) -> bool {
        (match self {
            Node::Leaf(leaf) => leaf.child,
            Node::Branch(branch) => branch.child,
        }) == child
    }
    pub fn as_branch(&mut self) -> &mut Branch<'a> {
        match self {
            Node::Leaf(_) => unreachable!(),
            Node::Branch(branch) => branch,
        }
    }
    pub fn get_nodes(&mut self) -> &mut [Node<'a>] {
        self.as_branch().nodes
    }
    pub fn get_depth(&mut self) -> u8 {
        self.as_branch().depth
    }
    pub fn get_lower(&mut self) -> i8 {
        match self {
            Node::Leaf(leaf) => leaf.value,
            Node::Branch(branch) => branch.lower,
        }
    }
    pub fn is_table(&mut self) -> bool {
        match self {
            Node::Leaf(leaf) => leaf.table,
            Node::Branch(_) => false,
        }
    }
    pub fn piece_count(&mut self) -> usize {
        match self {
            Node::Leaf(leaf) => leaf.game.count_pieces(),
            Node::Branch(_) => unreachable!(),
        }
    }
}

impl Default for Node<'_> {
    fn default() -> Self {
        Node::Leaf(Default::default())
    }
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
        match node {
            Node::Leaf(leaf) => Node::Leaf(*leaf),
            Node::Branch(branch) => Node::Branch(Branch {
                nodes: self
                    .bump
                    .alloc_slice_fill_iter(branch.nodes.iter().map(|new_node| self.copy(new_node))),
                lower: branch.lower,
                upper: branch.upper,
                depth: branch.depth,
                child: branch.child,
            }),
        }
    }

    #[inline(always)]
    pub fn new_node(&self, game: Game, child: u8) -> Node {
        let (table, value) = if game.is_loss() {
            (true, -127)
        } else {
            self.tablebase.eval(game)
        };
        Node::Leaf(Leaf {
            game,
            table,
            value,
            child,
        })
    }

    pub fn expand<'a>(&'a self, node: &mut Node<'a>) -> Option<()> {
        if let Node::Leaf(leaf) = node {
            let mut iter = leaf
                .game
                .forward()
                .enumerate()
                .map(|(new_child, new_game)| self.new_node(new_game, new_child as u8));

            let layout = Layout::array::<Node>(iter.len()).unwrap();
            let dst = self.bump.try_alloc_layout(layout).ok()?.cast::<Node>();

            let nodes = unsafe {
                for i in 0..iter.len() {
                    ptr::write(dst.as_ptr().add(i), iter.next().unwrap());
                }
                let result = slice::from_raw_parts_mut(dst.as_ptr(), iter.len());
                debug_assert_eq!(Layout::for_value(result), layout);
                result
            };
            *node = Node::Branch(Branch {
                lower: leaf.value,
                upper: 127,
                depth: 0,
                child: leaf.child,
                nodes,
            });
        };
        Some(())
    }

    pub fn bns<'a>(&'a self, node: &mut Node<'a>) -> Option<()> {
        self.expand(node)?;
        let depth = node.as_branch().depth + 1;
        let mut guess = node.as_branch().lower;
        println!("depth: {}", depth);
        while node.as_branch().depth != depth || node.as_branch().lower != node.as_branch().upper {
            let beta = max(node.as_branch().lower.saturating_add(1), guess);
            guess = self.alpha_beta(node, beta, depth)?;
        }
        assert!(node.as_branch().depth == depth);
        Some(())

        // node.lower = -127;
        // node.upper = 127;
        // node.depth += 1;
        // loop {
        //     let mut beta = max(guess, node.lower + 1);
        //     assert!(beta <= node.upper);
        //     assert!(beta > node.lower);
        //     // println!("* {}, {}, {}", node.lower, beta, node.upper);

        //     let mut count = false;
        //     let (first, rest) = node.nodes.split_first_mut().unwrap();
        //     guess = -self.alpha_beta(first, -beta + 1, node.depth - 1);
        //     if guess >= beta {
        //         node.lower = guess;
        //         if node.lower == node.upper {
        //             return node.lower;
        //         }
        //         beta = node.lower + 1;
        //         count = true;
        //     }
        //     for new_node in rest {
        //         let eval = -self.alpha_beta(new_node, -beta + 1, node.depth - 1);
        //         guess = max(guess, eval);
        //         if eval >= beta {
        //             swap(first, new_node);
        //             node.lower = eval;
        //             if node.lower == node.upper {
        //                 return node.lower;
        //             }
        //             if count {
        //                 continue;
        //             }
        //             beta = node.lower + 1;
        //             count = true;
        //         }
        //     }
        //     if count {
        //         return node.lower;
        //     }
        //     node.upper = guess;
        //     if node.lower == node.upper {
        //         return node.lower;
        //     }
        // }
    }

    pub fn alpha_beta<'a>(&'a self, node: &mut Node<'a>, beta: i8, depth: u8) -> Option<i8> {
        if depth == 0 {
            return self.quiescence(node, beta);
        }
        if node.is_table() {
            return Some(node.get_lower());
        }
        self.expand(node)?;
        let node = node.as_branch();
        if node.depth == depth {
            if node.lower >= beta {
                return Some(node.lower);
            }
            if node.upper < beta {
                return Some(node.upper);
            }
        } else {
            node.lower = -127;
            node.upper = 127;
            node.depth = depth;
        }
        let (first, rest) = node.nodes.split_first_mut().unwrap();
        let mut guess = -self.alpha_beta(first, -beta + 1, depth - 1)?;
        if guess >= beta {
            node.lower = guess;
            debug_assert!(node.lower <= node.upper);
            return Some(guess);
        }
        for new_node in rest {
            let eval = -self.alpha_beta(new_node, -beta + 1, depth - 1)?;
            guess = max(guess, eval);
            if eval >= beta {
                swap(first, new_node);
                node.lower = eval;
                debug_assert!(node.lower <= node.upper);
                return Some(eval);
            }
        }
        node.upper = guess;
        debug_assert!(node.lower <= node.upper);
        Some(guess)
    }

    pub fn quiescence<'a>(&'a self, node: &mut Node<'a>, beta: i8) -> Option<i8> {
        match node {
            Node::Branch(branch) => {
                debug_assert_eq!(branch.depth, 0);
                debug_assert_eq!(branch.lower, branch.upper);
                return Some(branch.lower);
            }
            Node::Leaf(leaf) => {
                if leaf.table {
                    return Some(leaf.value);
                }
            }
        }
        let pieces = node.piece_count();
        self.expand(node)?;
        let node = node.as_branch();
        let (first, rest) = node.nodes.split_first_mut().unwrap();
        if first.piece_count() < pieces {
            let eval = -self.quiescence(first, -beta + 1)?;
            node.lower = max(node.lower, eval);
        }
        for new_node in rest {
            if new_node.piece_count() < pieces {
                let eval = -self.quiescence(new_node, -beta + 1)?;
                node.lower = max(node.lower, eval);
                swap(first, new_node);
            }
        }
        node.upper = node.lower;
        Some(node.lower)
    }
}

#[cfg(test)]
mod test {
    use std::mem::size_of;

    use bumpalo::Bump;
    use onitama_move_gen::{eval::Eval, gen::Game, tablebase::TableBase};

    use crate::node::{Branch, Leaf, Node};

    #[test]
    fn try_forward_tie() {
        dbg!(Eval::new_tie().forward());
    }

    #[test]
    fn try_bump() {
        assert_eq!(size_of::<Branch>(), 24);
        // let bump = Bump::new();
        // let my = bump.alloc_with(|| {
        //     let nodes = bump.alloc_slice_fill_default(1);
        //     Test { nodes: Some(nodes) }
        // });
    }
}
