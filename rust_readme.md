# rs-avl for Rust

A generic AVL ordered set with logarithmic insertion, removal, and lookup.

## Installation

```toml
[dependencies]
rs-avl = "0.1"
```

## Example

```rust
use rs_avl::AvlTree;

let mut tree: AvlTree<_> = [4, 2, 6, 1, 3, 5, 7].into_iter().collect();

assert!(tree.insert(8));
assert!(!tree.insert(8));
assert_eq!(tree.search(&3), Some(&3));
assert_eq!(tree.range(2..=5).copied().collect::<Vec<_>>(), [2, 3, 4, 5]);
assert_eq!(tree.iter().copied().collect::<Vec<_>>(), [1, 2, 3, 4, 5, 6, 7, 8]);
assert!(tree.remove(&4));
```

`AvlTree<T>` does not require `Clone`. It supports borrowed-key search and
removal, `FromIterator`, `Extend`, iteration by reference, read-only node
inspection, and in-order, pre-order, post-order, and level-order traversal.

Full API documentation is available on [docs.rs](https://docs.rs/rs-avl).
