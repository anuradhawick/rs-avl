//! A generic AVL ordered set with logarithmic search and updates.
//!
//! [`AVLTree`] stores unique ordered values and provides insertion, removal,
//! borrowed-key search, lazy traversals, and bounded range iteration. Tree
//! height is maintained automatically through AVL rotations.
//!
//! # Example
//!
//! ```
//! use rs_avl::AVLTree;
//!
//! let mut tree = AVLTree::new();
//! tree.extend([4, 2, 6, 1, 3, 5, 7]);
//!
//! assert!(tree.contains(&5));
//! assert_eq!(tree.range(2..=5).copied().collect::<Vec<_>>(), [2, 3, 4, 5]);
//! assert!(tree.remove(&4));
//! ```

pub mod avl;
mod python;
mod python_tree;

pub use avl::{AVLNode, AVLTree};
