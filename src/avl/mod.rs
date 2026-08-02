//! Generic AVL tree implementation.

mod iter;
mod node;
mod tree;

pub use iter::{Iter, LevelOrder, PostOrder, PreOrder, Range};
pub use node::AVLNode;
pub use tree::AvlTree;

/// Compatibility spelling for code that uses the acronym in capitals.
pub type AVLTree<T> = AvlTree<T>;
