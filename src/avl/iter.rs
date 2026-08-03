use std::borrow::Borrow;
use std::collections::VecDeque;
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::ops::{Bound, RangeBounds};

use super::node::AVLNode;

/// A double-ended iterator over values in sorted order.
pub struct Iter<'a, T> {
    front_stack: Vec<&'a AVLNode<T>>,
    back_stack: Vec<&'a AVLNode<T>>,
    remaining: usize,
}

impl<'a, T> Iter<'a, T> {
    pub(crate) fn new(root: Option<&'a AVLNode<T>>, len: usize) -> Self {
        let mut iter = Self {
            front_stack: Vec::new(),
            back_stack: Vec::new(),
            remaining: len,
        };
        iter.push_left(root);
        iter.push_right(root);
        iter
    }

    fn push_left(&mut self, mut node: Option<&'a AVLNode<T>>) {
        while let Some(current) = node {
            self.front_stack.push(current);
            node = current.left.as_deref();
        }
    }

    fn push_right(&mut self, mut node: Option<&'a AVLNode<T>>) {
        while let Some(current) = node {
            self.back_stack.push(current);
            node = current.right.as_deref();
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let node = self
            .front_stack
            .pop()
            .expect("non-empty iterator must have a front node");
        self.remaining -= 1;
        self.push_left(node.right.as_deref());
        Some(&node.value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let node = self
            .back_stack
            .pop()
            .expect("non-empty iterator must have a back node");
        self.remaining -= 1;
        self.push_right(node.left.as_deref());
        Some(&node.value)
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {}
impl<T> FusedIterator for Iter<'_, T> {}

/// An ascending iterator starting at an inclusive lower-bound key.
pub struct IterFrom<'a, T> {
    stack: Vec<&'a AVLNode<T>>,
    remaining: usize,
}

impl<'a, T> IterFrom<'a, T> {
    pub(crate) fn new<Q>(root: Option<&'a AVLNode<T>>, start: &Q, count: usize) -> Self
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut iter = Self {
            stack: Vec::new(),
            remaining: count,
        };
        if count == 0 {
            return iter;
        }

        let mut node = root;
        while let Some(current) = node {
            if current.value.borrow() < start {
                node = current.right.as_deref();
            } else {
                iter.stack.push(current);
                node = current.left.as_deref();
            }
        }
        iter
    }

    fn push_left(&mut self, mut node: Option<&'a AVLNode<T>>) {
        while let Some(current) = node {
            self.stack.push(current);
            node = current.left.as_deref();
        }
    }
}

impl<'a, T> Iterator for IterFrom<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let node = self.stack.pop()?;
        self.remaining -= 1;
        self.push_left(node.right.as_deref());
        Some(&node.value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.remaining))
    }
}

/// A descending iterator starting at an inclusive upper-bound key.
pub struct IterTo<'a, T> {
    stack: Vec<&'a AVLNode<T>>,
    remaining: usize,
}

impl<'a, T> IterTo<'a, T> {
    pub(crate) fn new<Q>(root: Option<&'a AVLNode<T>>, end: &Q, count: usize) -> Self
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut iter = Self {
            stack: Vec::new(),
            remaining: count,
        };
        if count == 0 {
            return iter;
        }

        let mut node = root;
        while let Some(current) = node {
            if current.value.borrow() > end {
                node = current.left.as_deref();
            } else {
                iter.stack.push(current);
                node = current.right.as_deref();
            }
        }
        iter
    }

    fn push_right(&mut self, mut node: Option<&'a AVLNode<T>>) {
        while let Some(current) = node {
            self.stack.push(current);
            node = current.right.as_deref();
        }
    }
}

impl<'a, T> Iterator for IterTo<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let node = self.stack.pop()?;
        self.remaining -= 1;
        self.push_right(node.left.as_deref());
        Some(&node.value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.remaining))
    }
}

/// A root-left-right traversal iterator.
pub struct PreOrder<'a, T> {
    stack: Vec<&'a AVLNode<T>>,
}

impl<'a, T> PreOrder<'a, T> {
    pub(crate) fn new(root: Option<&'a AVLNode<T>>) -> Self {
        Self {
            stack: root.into_iter().collect(),
        }
    }
}

impl<'a, T> Iterator for PreOrder<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        if let Some(right) = node.right.as_deref() {
            self.stack.push(right);
        }
        if let Some(left) = node.left.as_deref() {
            self.stack.push(left);
        }
        Some(&node.value)
    }
}

/// A left-right-root traversal iterator.
pub struct PostOrder<'a, T> {
    stack: Vec<(&'a AVLNode<T>, bool)>,
}

impl<'a, T> PostOrder<'a, T> {
    pub(crate) fn new(root: Option<&'a AVLNode<T>>) -> Self {
        Self {
            stack: root.into_iter().map(|node| (node, false)).collect(),
        }
    }
}

impl<'a, T> Iterator for PostOrder<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, visited)) = self.stack.pop() {
            if visited {
                return Some(&node.value);
            }
            self.stack.push((node, true));
            if let Some(right) = node.right.as_deref() {
                self.stack.push((right, false));
            }
            if let Some(left) = node.left.as_deref() {
                self.stack.push((left, false));
            }
        }
        None
    }
}

/// A breadth-first traversal iterator.
pub struct LevelOrder<'a, T> {
    queue: VecDeque<&'a AVLNode<T>>,
}

impl<'a, T> LevelOrder<'a, T> {
    pub(crate) fn new(root: Option<&'a AVLNode<T>>) -> Self {
        Self {
            queue: root.into_iter().collect(),
        }
    }
}

impl<'a, T> Iterator for LevelOrder<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.queue.pop_front()?;
        if let Some(left) = node.left.as_deref() {
            self.queue.push_back(left);
        }
        if let Some(right) = node.right.as_deref() {
            self.queue.push_back(right);
        }
        Some(&node.value)
    }
}

/// An ascending iterator limited to a key range.
pub struct Range<'a, T, Q: ?Sized, R> {
    stack: Vec<&'a AVLNode<T>>,
    bounds: R,
    finished: bool,
    marker: PhantomData<&'a Q>,
}

impl<'a, T, Q, R> Range<'a, T, Q, R>
where
    T: Borrow<Q>,
    Q: Ord + ?Sized,
    R: RangeBounds<Q>,
{
    pub(crate) fn new(root: Option<&'a AVLNode<T>>, bounds: R) -> Self {
        let mut range = Self {
            stack: Vec::new(),
            bounds,
            finished: false,
            marker: PhantomData,
        };
        range.push_candidates(root);
        range
    }

    fn below_start(&self, value: &Q) -> bool {
        match self.bounds.start_bound() {
            Bound::Included(start) => value < start,
            Bound::Excluded(start) => value <= start,
            Bound::Unbounded => false,
        }
    }

    fn beyond_end(&self, value: &Q) -> bool {
        match self.bounds.end_bound() {
            Bound::Included(end) => value > end,
            Bound::Excluded(end) => value >= end,
            Bound::Unbounded => false,
        }
    }

    fn push_candidates(&mut self, mut node: Option<&'a AVLNode<T>>) {
        while let Some(current) = node {
            if self.below_start(current.value.borrow()) {
                node = current.right.as_deref();
            } else {
                self.stack.push(current);
                node = current.left.as_deref();
            }
        }
    }
}

impl<'a, T, Q, R> Iterator for Range<'a, T, Q, R>
where
    T: Borrow<Q>,
    Q: Ord + ?Sized,
    R: RangeBounds<Q>,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let node = self.stack.pop()?;
        if self.beyond_end(node.value.borrow()) {
            self.finished = true;
            self.stack.clear();
            return None;
        }
        self.push_candidates(node.right.as_deref());
        Some(&node.value)
    }
}
