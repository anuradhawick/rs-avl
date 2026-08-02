//! Python bindings for the integer-specialized AVL tree.

use std::ops::Bound as RangeBound;

use pyo3::exceptions::{PyStopIteration, PyValueError};
use pyo3::inspect::PyStaticExpr;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::{Borrowed, FromPyObject, type_hint_identifier, type_hint_subscript};

use crate::AvlTree;

/// Python bindings for `rs-avl`.
#[pymodule]
mod rs_avl {
    use super::*;

    /// Extracts any Python iterable while advertising its precise input type.
    struct IntIterable(Vec<i64>);

    impl<'a, 'py> FromPyObject<'a, 'py> for IntIterable {
        type Error = PyErr;

        const INPUT_TYPE: PyStaticExpr = type_hint_subscript!(
            type_hint_identifier!("typing", "Iterable"),
            type_hint_identifier!("builtins", "int")
        );

        fn extract(value: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
            let mut values = Vec::new();
            for value in value.try_iter()? {
                values.push(value?.extract()?);
            }
            Ok(Self(values))
        }
    }

    #[pymodule_export]
    #[expect(non_upper_case_globals)]
    /// The installed `rs-avl` package version.
    pub const __version__: &str = env!("CARGO_PKG_VERSION");

    #[pymodule_export]
    #[expect(non_upper_case_globals)]
    /// Names exported by `from rs_avl import *`.
    pub const __all__: [&str; 2] = ["AVLTree", "__version__"];

    /// A height-balanced ordered set of signed 64-bit integers.
    ///
    /// Values are unique, iteration is ascending, and search, insertion, and
    /// removal take logarithmic time while the tree remains balanced.
    #[pyclass(name = "AVLTree", module = "rs_avl")]
    struct PyAVLTree {
        inner: AvlTree<i64>,
    }

    #[pymethods]
    impl PyAVLTree {
        /// Create a tree from an optional iterable of integers.
        ///
        /// Duplicate values from the iterable are stored only once.
        #[new]
        #[pyo3(signature = (values = None))]
        fn new(values: Option<IntIterable>) -> Self {
            let mut inner = AvlTree::new();
            if let Some(values) = values {
                for value in values.0 {
                    inner.insert(value);
                }
            }
            Self { inner }
        }

        /// Insert `value` and return `True` if it was not already present.
        fn insert(&mut self, value: i64) -> bool {
            self.inner.insert(value)
        }

        /// Remove `value` and return `True` if it was present.
        fn remove(&mut self, value: i64) -> bool {
            self.inner.remove(&value)
        }

        /// Return the stored value equal to `value`, or `None` when absent.
        fn search(&self, value: i64) -> Option<i64> {
            self.inner.search(&value).copied()
        }

        /// Alias for `search`.
        fn get(&self, value: i64) -> Option<i64> {
            self.search(value)
        }

        /// Return whether `value` is present in the tree.
        fn contains(&self, value: i64) -> bool {
            self.inner.contains(&value)
        }

        /// Compatibility alias for `contains`.
        fn has_node(&self, value: i64) -> bool {
            self.contains(value)
        }

        /// Remove every value from the tree.
        fn clear(&mut self) {
            self.inner.clear();
        }

        /// Return whether the tree contains no values.
        fn is_empty(&self) -> bool {
            self.inner.is_empty()
        }

        #[getter]
        /// The height of the tree; an empty tree has height zero.
        fn height(&self) -> usize {
            self.inner.height()
        }

        /// Return the smallest value, or `None` when empty.
        fn first(&self) -> Option<i64> {
            self.inner.first().copied()
        }

        /// Alias for `first`.
        fn min(&self) -> Option<i64> {
            self.first()
        }

        /// Return the largest value, or `None` when empty.
        fn last(&self) -> Option<i64> {
            self.inner.last().copied()
        }

        /// Alias for `last`.
        fn max(&self) -> Option<i64> {
            self.last()
        }

        /// Return an ascending snapshot iterator between optional endpoints.
        ///
        /// The start is inclusive and the end is exclusive by default. Use
        /// `include_start` and `include_end` to change either boundary.
        /// Raises `ValueError` when `start` is greater than `end`.
        #[pyo3(signature = (start = None, end = None, *, include_start = true, include_end = false))]
        fn range(
            &self,
            start: Option<i64>,
            end: Option<i64>,
            include_start: bool,
            include_end: bool,
        ) -> PyResult<PyAVLTreeIterator> {
            if start.zip(end).is_some_and(|(start, end)| start > end) {
                return Err(PyValueError::new_err("range start must not exceed end"));
            }

            let start = match (start, include_start) {
                (Some(value), true) => RangeBound::Included(value),
                (Some(value), false) => RangeBound::Excluded(value),
                (None, _) => RangeBound::Unbounded,
            };
            let end = match (end, include_end) {
                (Some(value), true) => RangeBound::Included(value),
                (Some(value), false) => RangeBound::Excluded(value),
                (None, _) => RangeBound::Unbounded,
            };

            Ok(PyAVLTreeIterator::new(
                self.inner.range((start, end)).copied().collect(),
            ))
        }

        /// Return a snapshot iterator in ascending order.
        fn in_order(&self) -> PyAVLTreeIterator {
            PyAVLTreeIterator::new(self.inner.in_order().copied().collect())
        }

        /// Return a snapshot iterator in root-left-right order.
        fn pre_order(&self) -> PyAVLTreeIterator {
            PyAVLTreeIterator::new(self.inner.pre_order().copied().collect())
        }

        /// Return a snapshot iterator in left-right-root order.
        fn post_order(&self) -> PyAVLTreeIterator {
            PyAVLTreeIterator::new(self.inner.post_order().copied().collect())
        }

        /// Return a breadth-first snapshot iterator, level by level.
        fn level_order(&self) -> PyAVLTreeIterator {
            PyAVLTreeIterator::new(self.inner.level_order().copied().collect())
        }

        /// Return the number of unique values in the tree.
        fn __len__(&self) -> usize {
            self.inner.len()
        }

        /// Return `True` when the tree is non-empty.
        fn __bool__(&self) -> bool {
            !self.inner.is_empty()
        }

        /// Implement the `value in tree` membership operation.
        fn __contains__(&self, value: &Bound<'_, PyAny>) -> bool {
            value
                .extract::<i64>()
                .is_ok_and(|value| self.inner.contains(&value))
        }

        /// Iterate over a snapshot of the values in ascending order.
        fn __iter__(&self) -> PyAVLTreeIterator {
            self.in_order()
        }

        /// Return an unambiguous ascending representation of the tree.
        fn __repr__(&self) -> String {
            let values = self.inner.iter().copied().collect::<Vec<_>>();
            format!("AVLTree({values:?})")
        }
    }

    /// An iterator over a snapshot of tree values.
    ///
    /// Mutating or clearing the source tree does not invalidate this iterator.
    #[pyclass(name = "_AVLTreeIterator", module = "rs_avl")]
    struct PyAVLTreeIterator {
        values: std::vec::IntoIter<i64>,
    }

    impl PyAVLTreeIterator {
        fn new(values: Vec<i64>) -> Self {
            Self {
                values: values.into_iter(),
            }
        }
    }

    #[pymethods]
    impl PyAVLTreeIterator {
        /// Return this iterator object.
        fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
            slf
        }

        /// Return the next value, raising `StopIteration` when exhausted.
        fn __next__(&mut self) -> PyResult<i64> {
            self.values
                .next()
                .ok_or_else(|| PyStopIteration::new_err(()))
        }
    }
}
