use std::collections::BTreeSet;
use std::ops::Bound::{Excluded, Included};

use rs_avl::{AVLNode, AVLTree};

fn assert_invariants<T: Ord + std::fmt::Debug>(tree: &AVLTree<T>) {
    fn visit<T: Ord + std::fmt::Debug>(
        node: Option<&AVLNode<T>>,
        lower: Option<&T>,
        upper: Option<&T>,
    ) -> (usize, usize) {
        let Some(node) = node else {
            return (0, 0);
        };

        if let Some(lower) = lower {
            assert!(node.value() > lower);
        }
        if let Some(upper) = upper {
            assert!(node.value() < upper);
        }

        let (left_height, left_len) = visit(node.left(), lower, Some(node.value()));
        let (right_height, right_len) = visit(node.right(), Some(node.value()), upper);
        let expected_height = 1 + left_height.max(right_height);

        assert_eq!(node.height(), expected_height);
        assert!(left_height.abs_diff(right_height) <= 1);
        (expected_height, left_len + right_len + 1)
    }

    let (height, len) = visit(tree.root(), None, None);
    assert_eq!(tree.height(), height);
    assert_eq!(tree.len(), len);
}

#[test]
fn insertion_handles_all_four_rotation_shapes() {
    for values in [[30, 20, 10], [30, 10, 20], [10, 20, 30], [10, 30, 20]] {
        let tree: AVLTree<_> = values.into_iter().collect();
        assert_eq!(tree.root().map(AVLNode::value), Some(&20));
        assert_eq!(tree.iter().copied().collect::<Vec<_>>(), [10, 20, 30]);
        assert_invariants(&tree);
    }
}

#[test]
fn insert_search_and_duplicate_semantics_are_set_like() {
    let mut tree = AVLTree::new();
    assert!(tree.is_empty());
    assert!(tree.insert(String::from("pear")));
    assert!(tree.insert(String::from("apple")));
    assert!(tree.insert(String::from("orange")));
    assert!(!tree.insert(String::from("pear")));

    assert_eq!(tree.len(), 3);
    assert_eq!(tree.search("orange").map(String::as_str), Some("orange"));
    assert_eq!(tree.get("missing"), None);
    assert!(tree.contains("apple"));
    assert!(tree.has_node("pear"));
    assert_eq!(tree.min().map(String::as_str), Some("apple"));
    assert_eq!(tree.max().map(String::as_str), Some("pear"));
    assert_eq!(
        tree.range::<str, _>((Included("apple"), Included("orange")))
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["apple", "orange"]
    );
    assert_invariants(&tree);
}

#[test]
fn removal_rebalances_leaf_single_child_and_two_child_cases() {
    let mut tree: AVLTree<_> = (0..100).collect();

    for value in [99, 98, 50, 0, 64, 32, 31, 63, 10, 20, 40] {
        assert!(tree.remove(&value));
        assert!(!tree.contains(&value));
        assert_invariants(&tree);
    }
    assert!(!tree.remove(&1_000));

    let values = tree.iter().copied().collect::<Vec<_>>();
    assert!(values.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(values.len(), tree.len());
}

#[test]
fn ranges_observe_inclusive_exclusive_and_unbounded_limits() {
    let tree: AVLTree<_> = (0..10).collect();

    assert_eq!(tree.range(3..7).copied().collect::<Vec<_>>(), [3, 4, 5, 6]);
    assert_eq!(
        tree.range(3..=7).copied().collect::<Vec<_>>(),
        [3, 4, 5, 6, 7]
    );
    assert_eq!(tree.range(..3).copied().collect::<Vec<_>>(), [0, 1, 2]);
    assert_eq!(tree.range(8..).copied().collect::<Vec<_>>(), [8, 9]);
    assert_eq!(
        tree.range((Excluded(3), Included(6)))
            .copied()
            .collect::<Vec<_>>(),
        [4, 5, 6]
    );
}

#[test]
fn iter_from_seeks_to_an_inclusive_lower_bound_and_limits_results() {
    let numbers: AVLTree<_> = [10, 20, 30, 40, 50].into_iter().collect();

    assert_eq!(
        numbers.iter_from(&20, 3).copied().collect::<Vec<_>>(),
        [20, 30, 40]
    );
    assert_eq!(
        numbers.iter_from(&25, 3).copied().collect::<Vec<_>>(),
        [30, 40, 50]
    );
    assert_eq!(
        numbers.iter_from(&50, 10).copied().collect::<Vec<_>>(),
        [50]
    );
    assert!(numbers.iter_from(&60, 3).next().is_none());
    assert!(numbers.iter_from(&10, 0).next().is_none());

    let words: AVLTree<_> = ["apple".to_owned(), "pear".to_owned()]
        .into_iter()
        .collect();
    assert_eq!(
        words
            .iter_from("banana", 1)
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["pear"]
    );
}

#[test]
fn traversal_iterators_have_their_documented_order() {
    let tree: AVLTree<_> = [4, 2, 6, 1, 3, 5, 7].into_iter().collect();

    assert_eq!(
        tree.in_order().copied().collect::<Vec<_>>(),
        [1, 2, 3, 4, 5, 6, 7]
    );
    assert_eq!(
        tree.pre_order().copied().collect::<Vec<_>>(),
        [4, 2, 1, 3, 6, 5, 7]
    );
    assert_eq!(
        tree.post_order().copied().collect::<Vec<_>>(),
        [1, 3, 2, 5, 7, 6, 4]
    );
    assert_eq!(
        tree.level_order().copied().collect::<Vec<_>>(),
        [4, 2, 6, 1, 3, 5, 7]
    );
    assert_eq!(
        (&tree).into_iter().copied().collect::<Vec<_>>(),
        [1, 2, 3, 4, 5, 6, 7]
    );
}

#[test]
fn values_do_not_need_to_implement_clone() {
    #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
    struct NotClone(i32);

    let mut tree = AVLTree::new();
    for value in [2, 1, 4, 3, 5] {
        tree.insert(NotClone(value));
    }
    assert!(tree.remove(&NotClone(2)));
    assert_eq!(
        tree.iter().map(|value| value.0).collect::<Vec<_>>(),
        [1, 3, 4, 5]
    );
    assert_invariants(&tree);
}

#[test]
fn clear_restores_the_empty_state() {
    let mut tree: AVLTree<_> = (1..=5).collect();
    tree.clear();

    assert!(tree.is_empty());
    assert_eq!(tree.len(), 0);
    assert_eq!(tree.height(), 0);
    assert_eq!(tree.first(), None);
    assert_eq!(tree.last(), None);
    assert_eq!(format!("{tree:?}"), "{}");
}

#[test]
fn mixed_updates_match_the_standard_ordered_set() {
    let mut tree = AVLTree::new();
    let mut expected = BTreeSet::new();
    let mut state = 0x1234_5678_u32;

    for step in 0..2_000 {
        // A fixed linear-congruential sequence keeps this stress test repeatable.
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let value = (state % 250) as i32;
        if state & 1 == 0 {
            assert_eq!(tree.insert(value), expected.insert(value));
        } else {
            assert_eq!(tree.remove(&value), expected.remove(&value));
        }

        assert_eq!(
            tree.iter().copied().collect::<Vec<_>>(),
            expected.iter().copied().collect::<Vec<_>>()
        );
        if step % 25 == 0 {
            assert_invariants(&tree);
        }
    }
    assert_invariants(&tree);
}
