//! Node storage and AVL balancing primitives.

pub(crate) type Link<T> = Option<Box<AVLNode<T>>>;

/// A read-only view of a node in an [`AVLTree`](super::AVLTree).
///
/// Tree mutations are deliberately kept on `AVLTree` so callers cannot break
/// the ordering or balance invariants.
#[derive(Debug)]
pub struct AVLNode<T> {
    pub(crate) value: T,
    pub(crate) left: Link<T>,
    pub(crate) right: Link<T>,
    pub(crate) height: usize,
}

impl<T> AVLNode<T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            value,
            left: None,
            right: None,
            height: 1,
        }
    }

    /// Returns the value stored at this node.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns the cached height of this node.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns this node's left child, if present.
    pub fn left(&self) -> Option<&Self> {
        self.left.as_deref()
    }

    /// Returns this node's right child, if present.
    pub fn right(&self) -> Option<&Self> {
        self.right.as_deref()
    }
}

pub(crate) fn height<T>(node: &Link<T>) -> usize {
    node.as_deref().map_or(0, |node| node.height)
}

fn update_height<T>(node: &mut AVLNode<T>) {
    node.height = 1 + height(&node.left).max(height(&node.right));
}

fn balance_factor<T>(node: &AVLNode<T>) -> isize {
    height(&node.left) as isize - height(&node.right) as isize
}

/// Rotates a left-heavy subtree to the right.
///
/// Before: `y` owns `x`, and `x` owns the middle subtree `b`.
/// After: `x` becomes the root, `y` becomes its right child, and `b` moves
/// to `y.left`. This preserves the in-order sequence while reducing height.
fn rotate_right<T>(mut y: Box<AVLNode<T>>) -> Box<AVLNode<T>> {
    let mut x = y.left.take().expect("right rotation requires a left child");
    let middle = x.right.take();

    y.left = middle;
    update_height(&mut y);
    x.right = Some(y);
    update_height(&mut x);
    x
}

/// Rotates a right-heavy subtree to the left.
///
/// This is the mirror of `rotate_right`: the new root is the old right child,
/// and its middle subtree moves to the old root's right side. No values are
/// reordered; only ownership links and cached heights change.
fn rotate_left<T>(mut x: Box<AVLNode<T>>) -> Box<AVLNode<T>> {
    let mut y = x
        .right
        .take()
        .expect("left rotation requires a right child");
    let middle = y.left.take();

    x.right = middle;
    update_height(&mut x);
    y.left = Some(x);
    update_height(&mut y);
    y
}

pub(crate) fn rebalance<T>(mut node: Box<AVLNode<T>>) -> Box<AVLNode<T>> {
    update_height(&mut node);
    let balance = balance_factor(&node);

    if balance > 1 {
        // A left-right shape first needs its child straightened into a
        // left-left shape; the final right rotation can then lift it safely.
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
        // Mirror case: turn a right-left zig-zag into a right-right line before
        // rotating the unbalanced root to the left.
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
