//! Fallible AVL storage for Python objects and their comparison keys.
//!
//! Rust's [`Ord`] trait cannot represent Python comparison errors. This module
//! therefore keeps a small Python-specific AVL implementation whose operations
//! return [`PyResult`]. The public Rust [`AVLTree`](crate::AVLTree) remains
//! generic and uses ordinary infallible Rust ordering.

use std::cmp::Ordering;
use std::collections::VecDeque;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyAny;

type Link = Option<Box<Node>>;

/// A Python value paired with the key computed when it was inserted.
///
/// Caching the key avoids repeatedly calling a user key function and lets
/// traversals return the original object instead of its ordering key.
pub(crate) struct Entry {
    pub(crate) value: Py<PyAny>,
    key: Py<PyAny>,
}

impl Entry {
    pub(crate) fn new(value: Py<PyAny>, key: Py<PyAny>) -> Self {
        Self { value, key }
    }
}

/// An internal node with a cached height for constant-time balance checks.
struct Node {
    entry: Entry,
    left: Link,
    right: Link,
    height: usize,
}

impl Node {
    fn new(entry: Entry) -> Self {
        Self {
            entry,
            left: None,
            right: None,
            height: 1,
        }
    }
}

#[derive(Default)]
/// An AVL ordered set specialized for owned Python references.
///
/// Equal keys are duplicates, so `len` counts unique keys rather than object
/// identities. Every comparison happens through Python and may fail.
pub(crate) struct PythonAvlTree {
    root: Link,
    len: usize,
}

impl PythonAvlTree {
    /// Insert an entry and report whether its key was new.
    pub(crate) fn insert(&mut self, py: Python<'_>, entry: Entry) -> PyResult<bool> {
        let inserted = insert_at(py, &mut self.root, entry)?;
        self.len += usize::from(inserted);
        Ok(inserted)
    }

    /// Remove the entry equal to an already-extracted key.
    pub(crate) fn remove(&mut self, py: Python<'_>, key: &Py<PyAny>) -> PyResult<bool> {
        let removed = remove_at(py, &mut self.root, key)?;
        self.len -= usize::from(removed);
        Ok(removed)
    }

    /// Find the original object associated with an extracted key.
    pub(crate) fn search(&self, py: Python<'_>, key: &Py<PyAny>) -> PyResult<Option<Py<PyAny>>> {
        let mut node = self.root.as_deref();
        while let Some(current) = node {
            match compare(py, key, &current.entry.key)? {
                Ordering::Less => node = current.left.as_deref(),
                Ordering::Greater => node = current.right.as_deref(),
                Ordering::Equal => return Ok(Some(current.entry.value.clone_ref(py))),
            }
        }
        Ok(None)
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn height(&self) -> usize {
        height(&self.root)
    }

    pub(crate) fn clear(&mut self) {
        self.root = None;
        self.len = 0;
    }

    pub(crate) fn first(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        let mut node = self.root.as_deref()?;
        while let Some(left) = node.left.as_deref() {
            node = left;
        }
        Some(node.entry.value.clone_ref(py))
    }

    pub(crate) fn last(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        let mut node = self.root.as_deref()?;
        while let Some(right) = node.right.as_deref() {
            node = right;
        }
        Some(node.entry.value.clone_ref(py))
    }

    /// Return values in ascending key order.
    pub(crate) fn in_order(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let mut values = Vec::with_capacity(self.len);
        collect_in_order(self.root.as_deref(), py, &mut values);
        values
    }

    /// Return values in root-left-right order.
    pub(crate) fn pre_order(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let mut values = Vec::with_capacity(self.len);
        collect_pre_order(self.root.as_deref(), py, &mut values);
        values
    }

    /// Return values in left-right-root order.
    pub(crate) fn post_order(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let mut values = Vec::with_capacity(self.len);
        collect_post_order(self.root.as_deref(), py, &mut values);
        values
    }

    /// Return values breadth-first, one tree level at a time.
    pub(crate) fn level_order(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        let mut values = Vec::with_capacity(self.len);
        let mut queue = VecDeque::new();
        if let Some(root) = self.root.as_deref() {
            queue.push_back(root);
        }
        while let Some(node) = queue.pop_front() {
            values.push(node.entry.value.clone_ref(py));
            if let Some(left) = node.left.as_deref() {
                queue.push_back(left);
            }
            if let Some(right) = node.right.as_deref() {
                queue.push_back(right);
            }
        }
        values
    }

    /// Collect values whose keys satisfy the requested endpoints.
    ///
    /// Bounds are keys, not complete values. The recursive walk skips branches
    /// which cannot contain a matching key.
    pub(crate) fn range(
        &self,
        py: Python<'_>,
        start: Option<&Py<PyAny>>,
        end: Option<&Py<PyAny>>,
        include_start: bool,
        include_end: bool,
    ) -> PyResult<Vec<Py<PyAny>>> {
        if let (Some(start), Some(end)) = (start, end)
            && compare(py, start, end)? == Ordering::Greater
        {
            return Err(PyValueError::new_err("range start must not exceed end"));
        }

        let mut values = Vec::new();
        collect_range(
            self.root.as_deref(),
            py,
            start,
            end,
            include_start,
            include_end,
            &mut values,
        )?;
        Ok(values)
    }

    /// Collect at most `count` values from an inclusive lower-bound key.
    ///
    /// If `start` is absent, collection begins at the first greater key. The
    /// initial seek is logarithmic and only the returned successors are walked.
    pub(crate) fn iter_from(
        &self,
        py: Python<'_>,
        start: &Py<PyAny>,
        count: usize,
    ) -> PyResult<Vec<Py<PyAny>>> {
        let mut values = Vec::with_capacity(count.min(self.len));
        if count == 0 {
            return Ok(values);
        }

        let mut stack = Vec::new();
        let mut node = self.root.as_deref();
        while let Some(current) = node {
            if compare(py, &current.entry.key, start)? == Ordering::Less {
                node = current.right.as_deref();
            } else {
                stack.push(current);
                node = current.left.as_deref();
            }
        }

        while values.len() < count {
            let Some(current) = stack.pop() else {
                break;
            };
            values.push(current.entry.value.clone_ref(py));

            let mut successor = current.right.as_deref();
            while let Some(next) = successor {
                stack.push(next);
                successor = next.left.as_deref();
            }
        }
        Ok(values)
    }
}

/// Ask Python for a total ordering between two keys.
///
/// `PyAny::compare` tries equality, less-than, then greater-than. It propagates
/// `TypeError` for incompatible objects and rejects values such as NaN when all
/// three comparisons are false.
fn compare(py: Python<'_>, left: &Py<PyAny>, right: &Py<PyAny>) -> PyResult<Ordering> {
    left.bind(py).compare(right.bind(py))
}

fn insert_at(py: Python<'_>, link: &mut Link, entry: Entry) -> PyResult<bool> {
    let Some(node) = link.as_mut() else {
        *link = Some(Box::new(Node::new(entry)));
        return Ok(true);
    };

    // Compare before changing any links. If Python raises, `?` returns while
    // this node and every ancestor still have their original structure.
    let inserted = match compare(py, &entry.key, &node.entry.key)? {
        Ordering::Less => insert_at(py, &mut node.left, entry)?,
        Ordering::Greater => insert_at(py, &mut node.right, entry)?,
        Ordering::Equal => false,
    };
    if inserted {
        // The recursive insertion succeeded, so it is safe to take this
        // subtree temporarily and restore its AVL balance.
        *link = link.take().map(rebalance);
    }
    Ok(inserted)
}

fn remove_at(py: Python<'_>, link: &mut Link, key: &Py<PyAny>) -> PyResult<bool> {
    let Some(node) = link.as_mut() else {
        return Ok(false);
    };

    // No ownership link is taken until every fallible comparison on the search
    // path has succeeded. An exception therefore leaves the tree untouched.
    match compare(py, key, &node.entry.key)? {
        Ordering::Less => {
            let removed = remove_at(py, &mut node.left, key)?;
            if removed {
                *link = link.take().map(rebalance);
            }
            Ok(removed)
        }
        Ordering::Greater => {
            let removed = remove_at(py, &mut node.right, key)?;
            if removed {
                *link = link.take().map(rebalance);
            }
            Ok(removed)
        }
        Ordering::Equal => {
            let mut removed = link.take().expect("matched node must exist");
            *link = match (removed.left.take(), removed.right.take()) {
                (None, right) => right,
                (left, None) => left,
                (left, Some(right)) => {
                    // With two children, move the smallest node from the right
                    // subtree here. Moving the whole entry keeps value and key
                    // together without another Python comparison.
                    let (new_right, mut successor) = take_min(right);
                    successor.left = left;
                    successor.right = new_right;
                    Some(rebalance(successor))
                }
            };
            Ok(true)
        }
    }
}

fn take_min(mut node: Box<Node>) -> (Link, Box<Node>) {
    match node.left.take() {
        None => (node.right.take(), node),
        Some(left) => {
            let (new_left, minimum) = take_min(left);
            node.left = new_left;
            (Some(rebalance(node)), minimum)
        }
    }
}

fn height(link: &Link) -> usize {
    link.as_deref().map_or(0, |node| node.height)
}

fn update_height(node: &mut Node) {
    node.height = 1 + height(&node.left).max(height(&node.right));
}

fn balance_factor(node: &Node) -> isize {
    height(&node.left) as isize - height(&node.right) as isize
}

fn rebalance(mut node: Box<Node>) -> Box<Node> {
    update_height(&mut node);
    let balance = balance_factor(&node);
    if balance > 1 {
        // A left-right zig-zag needs a child rotation first. It then becomes a
        // left-left shape that one right rotation can straighten.
        if node
            .left
            .as_deref()
            .is_some_and(|left| balance_factor(left) < 0)
        {
            node.left = node.left.take().map(rotate_left);
        }
        return rotate_right(node);
    }
    if balance < -1 {
        // Mirror the process for a right-left zig-zag.
        if node
            .right
            .as_deref()
            .is_some_and(|right| balance_factor(right) > 0)
        {
            node.right = node.right.take().map(rotate_right);
        }
        return rotate_left(node);
    }
    node
}

fn rotate_right(mut root: Box<Node>) -> Box<Node> {
    // The left child rises to the root. Its former right subtree moves to the
    // old root's left side, preserving the in-order key sequence.
    let mut child = root.left.take().expect("right rotation needs a left child");
    root.left = child.right.take();
    update_height(&mut root);
    child.right = Some(root);
    update_height(&mut child);
    child
}

fn rotate_left(mut root: Box<Node>) -> Box<Node> {
    // Mirror rotation: the right child rises and its left subtree becomes the
    // old root's right subtree. Heights are updated from bottom to top.
    let mut child = root
        .right
        .take()
        .expect("left rotation needs a right child");
    root.right = child.left.take();
    update_height(&mut root);
    child.left = Some(root);
    update_height(&mut child);
    child
}

fn collect_in_order(node: Option<&Node>, py: Python<'_>, values: &mut Vec<Py<PyAny>>) {
    if let Some(node) = node {
        collect_in_order(node.left.as_deref(), py, values);
        values.push(node.entry.value.clone_ref(py));
        collect_in_order(node.right.as_deref(), py, values);
    }
}

fn collect_pre_order(node: Option<&Node>, py: Python<'_>, values: &mut Vec<Py<PyAny>>) {
    if let Some(node) = node {
        values.push(node.entry.value.clone_ref(py));
        collect_pre_order(node.left.as_deref(), py, values);
        collect_pre_order(node.right.as_deref(), py, values);
    }
}

fn collect_post_order(node: Option<&Node>, py: Python<'_>, values: &mut Vec<Py<PyAny>>) {
    if let Some(node) = node {
        collect_post_order(node.left.as_deref(), py, values);
        collect_post_order(node.right.as_deref(), py, values);
        values.push(node.entry.value.clone_ref(py));
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_range(
    node: Option<&Node>,
    py: Python<'_>,
    start: Option<&Py<PyAny>>,
    end: Option<&Py<PyAny>>,
    include_start: bool,
    include_end: bool,
    values: &mut Vec<Py<PyAny>>,
) -> PyResult<()> {
    let Some(node) = node else {
        return Ok(());
    };

    // Compare this node once per bound and reuse the results for pruning,
    // inclusion, and deciding whether to visit the other branch.
    let start_order = start
        .map(|start| compare(py, &node.entry.key, start))
        .transpose()?;
    let end_order = end
        .map(|end| compare(py, &node.entry.key, end))
        .transpose()?;

    // Visit left only when a smaller key could still meet the lower bound.
    if start_order.is_none_or(|order| order == Ordering::Greater) {
        collect_range(
            node.left.as_deref(),
            py,
            start,
            end,
            include_start,
            include_end,
            values,
        )?;
    }

    let within_start = start_order.is_none_or(|order| {
        order == Ordering::Greater || (include_start && order == Ordering::Equal)
    });
    let within_end = end_order
        .is_none_or(|order| order == Ordering::Less || (include_end && order == Ordering::Equal));
    if within_start && within_end {
        values.push(node.entry.value.clone_ref(py));
    }

    // Visit right only when a larger key could still meet the upper bound.
    if end_order.is_none_or(|order| order == Ordering::Less) {
        collect_range(
            node.right.as_deref(),
            py,
            start,
            end,
            include_start,
            include_end,
            values,
        )?;
    }
    Ok(())
}
