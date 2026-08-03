use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt;
use std::iter::FromIterator;
use std::ops::RangeBounds;

use super::iter::{Iter, LevelOrder, PostOrder, PreOrder, Range};
use super::node::{AVLNode, Link, height, rebalance};

/// A height-balanced binary search tree storing unique values.
pub struct AVLTree<T> {
    root: Link<T>,
    len: usize,
}

impl<T> AVLTree<T> {
    /// Creates an empty tree.
    pub const fn new() -> Self {
        Self { root: None, len: 0 }
    }

    /// Returns the number of values in the tree.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` when the tree contains no values.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the tree height, where an empty tree has height zero.
    pub fn height(&self) -> usize {
        height(&self.root)
    }

    /// Removes all values.
    pub fn clear(&mut self) {
        self.root = None;
        self.len = 0;
    }

    /// Returns a read-only view of the root node.
    pub fn root(&self) -> Option<&AVLNode<T>> {
        self.root.as_deref()
    }

    /// Iterates over values in ascending (in-order) sequence.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(self.root.as_deref())
    }

    /// Alias for [`iter`](Self::iter).
    pub fn in_order(&self) -> Iter<'_, T> {
        self.iter()
    }

    /// Traverses values in root-left-right order.
    pub fn pre_order(&self) -> PreOrder<'_, T> {
        PreOrder::new(self.root.as_deref())
    }

    /// Traverses values in left-right-root order.
    pub fn post_order(&self) -> PostOrder<'_, T> {
        PostOrder::new(self.root.as_deref())
    }

    /// Traverses values one level at a time.
    pub fn level_order(&self) -> LevelOrder<'_, T> {
        LevelOrder::new(self.root.as_deref())
    }
}

impl<T: Ord> AVLTree<T> {
    /// Inserts `value`, returning `false` if it was already present.
    pub fn insert(&mut self, value: T) -> bool {
        let (root, inserted) = insert_node(self.root.take(), value);
        self.root = root;
        self.len += usize::from(inserted);
        inserted
    }

    /// Finds and returns the stored value equal to `key`.
    pub fn search<Q>(&self, key: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut node = self.root.as_deref();
        while let Some(current) = node {
            match key.cmp(current.value.borrow()) {
                Ordering::Less => node = current.left.as_deref(),
                Ordering::Greater => node = current.right.as_deref(),
                Ordering::Equal => return Some(&current.value),
            }
        }
        None
    }

    /// Conventional alias for [`search`](Self::search).
    pub fn get<Q>(&self, key: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.search(key)
    }

    /// Returns whether `key` is present.
    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.search(key).is_some()
    }

    /// Compatibility alias for [`contains`](Self::contains).
    pub fn has_node<Q>(&self, key: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.contains(key)
    }

    /// Removes `key`, returning whether a value was removed.
    pub fn remove<Q>(&mut self, key: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let (root, removed) = remove_node(self.root.take(), key);
        self.root = root;
        self.len -= usize::from(removed);
        removed
    }

    /// Returns the smallest value.
    pub fn first(&self) -> Option<&T> {
        let mut node = self.root.as_deref()?;
        while let Some(left) = node.left.as_deref() {
            node = left;
        }
        Some(&node.value)
    }

    /// Alias for [`first`](Self::first).
    pub fn min(&self) -> Option<&T> {
        self.first()
    }

    /// Returns the largest value.
    pub fn last(&self) -> Option<&T> {
        let mut node = self.root.as_deref()?;
        while let Some(right) = node.right.as_deref() {
            node = right;
        }
        Some(&node.value)
    }

    /// Alias for [`last`](Self::last).
    pub fn max(&self) -> Option<&T> {
        self.last()
    }

    /// Iterates over values inside `bounds` in ascending order.
    ///
    /// The search skips subtrees below the lower bound and stops at the upper
    /// bound, giving `O(log n + k)` work for `k` returned values.
    pub fn range<Q, R>(&self, bounds: R) -> Range<'_, T, Q, R>
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        Range::new(self.root.as_deref(), bounds)
    }
}

fn insert_node<T: Ord>(node: Link<T>, value: T) -> (Link<T>, bool) {
    let Some(mut node) = node else {
        return (Some(Box::new(AVLNode::new(value))), true);
    };

    let inserted = match value.cmp(&node.value) {
        Ordering::Less => {
            let (left, inserted) = insert_node(node.left.take(), value);
            node.left = left;
            inserted
        }
        Ordering::Greater => {
            let (right, inserted) = insert_node(node.right.take(), value);
            node.right = right;
            inserted
        }
        Ordering::Equal => return (Some(node), false),
    };
    (Some(rebalance(node)), inserted)
}

fn remove_node<T, Q>(node: Link<T>, key: &Q) -> (Link<T>, bool)
where
    T: Borrow<Q> + Ord,
    Q: Ord + ?Sized,
{
    let Some(mut node) = node else {
        return (None, false);
    };

    match key.cmp(node.value.borrow()) {
        Ordering::Less => {
            let (left, removed) = remove_node(node.left.take(), key);
            node.left = left;
            if removed {
                (Some(rebalance(node)), true)
            } else {
                (Some(node), false)
            }
        }
        Ordering::Greater => {
            let (right, removed) = remove_node(node.right.take(), key);
            node.right = right;
            if removed {
                (Some(rebalance(node)), true)
            } else {
                (Some(node), false)
            }
        }
        Ordering::Equal => match (node.left.take(), node.right.take()) {
            (None, right) => (right, true),
            (left, None) => (left, true),
            (left, Some(right)) => {
                let (new_right, mut successor) = take_min(right);
                successor.left = left;
                successor.right = new_right;
                (Some(rebalance(successor)), true)
            }
        },
    }
}

fn take_min<T>(mut node: Box<AVLNode<T>>) -> (Link<T>, Box<AVLNode<T>>) {
    match node.left.take() {
        None => (node.right.take(), node),
        Some(left) => {
            let (new_left, minimum) = take_min(left);
            node.left = new_left;
            (Some(rebalance(node)), minimum)
        }
    }
}

impl<T> Default for AVLTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> Extend<T> for AVLTree<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.insert(value);
        }
    }
}

impl<T: Ord> FromIterator<T> for AVLTree<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut tree = Self::new();
        tree.extend(iter);
        tree
    }
}

impl<'a, T> IntoIterator for &'a AVLTree<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: fmt::Debug> fmt::Debug for AVLTree<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_set().entries(self.iter()).finish()
    }
}
