# rs-avl

[![crates.io](https://img.shields.io/crates/v/rs-avl.svg)](https://crates.io/crates/rs-avl)
[![docs.rs](https://docs.rs/rs-avl/badge.svg)](https://docs.rs/rs-avl)
[![Publish crates.io](https://github.com/anuradhawick/rs-avl/actions/workflows/crates.yml/badge.svg)](https://github.com/anuradhawick/rs-avl/actions/workflows/crates.yml)

A compact, generic ordered set powered by an AVL tree. It keeps itself
height-balanced after every insertion and removal, giving predictable
logarithmic lookup while retaining simple, sorted iteration.

Use it for primitive values, strings, domain records, or any type for which
you can define a meaningful [`Ord`](https://doc.rust-lang.org/std/cmp/trait.Ord.html)
implementation.

## Highlights

- `O(log n)` insertion, removal, and search
- `O(log n + k)` bounded ranges returning `k` values
- Unique-value set semantics
- No `Clone` requirement on stored values
- Borrowed-key lookup with `Borrow`
- In-order, pre-order, post-order, and level-order traversals
- `FromIterator`, `Extend`, and iteration by reference

## Installation

```toml
[dependencies]
rs-avl = "0.1"
```

## Quick start

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

## Store your own structs

The tree is not limited to numbers. Deriving `Ord` gives an arbitrary Rust
struct a lexicographic ordering based on its field order. Here, releases sort
by semantic version first, then by channel and title:

```rust
use rs_avl::AvlTree;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Release {
    version: (u16, u16, u16),
    channel: &'static str,
    title: String,
}

let mut releases = AvlTree::new();

releases.insert(Release {
    version: (2, 0, 0),
    channel: "stable",
    title: "Balanced Horizon".into(),
});
releases.insert(Release {
    version: (1, 5, 0),
    channel: "stable",
    title: "Range Finder".into(),
});
releases.insert(Release {
    version: (2, 1, 0),
    channel: "beta",
    title: "Traversal Preview".into(),
});

let versions = releases
    .iter()
    .map(|release| release.version)
    .collect::<Vec<_>>();

assert_eq!(versions, [(1, 5, 0), (2, 0, 0), (2, 1, 0)]);
assert_eq!(releases.first().unwrap().title, "Range Finder");
assert_eq!(releases.last().unwrap().channel, "beta");
```

Notice that `Release` does not implement `Clone`: the tree takes ownership of
each value and its iterators yield shared references. For domain-specific
ordering—such as comparing releases by version only—you can implement `Ord`
manually and the AVL tree will follow those rules everywhere.

## Traversal and inspection

`iter()` and `in_order()` yield ascending values. `pre_order()`,
`post_order()`, and `level_order()` expose the tree's current balanced shape,
which is useful for visualization and teaching. `root()` provides read-only
node inspection without allowing callers to break ordering or height
invariants.

Full API documentation is available on [docs.rs](https://docs.rs/rs-avl).

## License

Dual-licensed under your choice of GPL-3.0-only or Apache-2.0.
