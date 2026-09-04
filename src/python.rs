//! Python bindings for arbitrary comparable objects.

use pyo3::exceptions::{PyStopIteration, PyTypeError};
use pyo3::inspect::PyStaticExpr;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyList, PyString, PyTuple};
use pyo3::{Borrowed, FromPyObject, type_hint_identifier, type_hint_subscript, type_hint_union};

use crate::python_tree::{Entry, PythonAvlTree};

/// Python bindings for `rs-avl`.
#[pymodule]
mod rs_avl {
    use super::*;

    const ANY: PyStaticExpr = type_hint_identifier!("typing", "Any");
    const CALLABLE_ARGUMENTS: PyStaticExpr = PyStaticExpr::List { elts: &[ANY] };
    const KEY_CALLABLE: PyStaticExpr = type_hint_subscript!(
        type_hint_identifier!("typing", "Callable"),
        CALLABLE_ARGUMENTS,
        ANY
    );

    struct PythonValues(Vec<Py<PyAny>>);

    impl<'a, 'py> FromPyObject<'a, 'py> for PythonValues {
        type Error = PyErr;

        const INPUT_TYPE: PyStaticExpr = type_hint_subscript!(
            type_hint_identifier!("typing", "Iterable"),
            type_hint_identifier!("typing", "Any")
        );

        fn extract(value: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
            let mut values = Vec::new();
            for value in value.try_iter()? {
                values.push(value?.unbind());
            }
            Ok(Self(values))
        }
    }

    enum KeyExtractor {
        Identity,
        Attribute(String),
        Callable(Py<PyAny>),
    }

    impl KeyExtractor {
        fn extract(&self, py: Python<'_>, value: &Py<PyAny>) -> PyResult<Py<PyAny>> {
            match self {
                Self::Identity => Ok(value.clone_ref(py)),
                Self::Attribute(name) => Ok(value.bind(py).getattr(name.as_str())?.unbind()),
                Self::Callable(callable) => {
                    Ok(callable.bind(py).call1((value.bind(py),))?.unbind())
                }
            }
        }
    }

    impl<'a, 'py> FromPyObject<'a, 'py> for KeyExtractor {
        type Error = PyErr;

        const INPUT_TYPE: PyStaticExpr =
            type_hint_union!(type_hint_identifier!("builtins", "str"), KEY_CALLABLE);

        fn extract(value: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
            if let Ok(name) = value.cast::<PyString>() {
                return Ok(Self::Attribute(name.to_str()?.to_owned()));
            }
            if value.is_callable() {
                return Ok(Self::Callable(value.to_owned().unbind()));
            }
            Err(PyTypeError::new_err(
                "key must be an attribute name, a callable, or None",
            ))
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

    /// A height-balanced ordered set of comparable Python objects.
    ///
    /// Values are ordered directly unless `key` is an attribute name or a
    /// callable. Equal keys are treated as duplicate set entries.
    #[pyclass(name = "AVLTree", module = "rs_avl")]
    struct PyAVLTree {
        inner: PythonAvlTree,
        key: KeyExtractor,
    }

    #[pymethods]
    impl PyAVLTree {
        /// Create a tree from optional values and a fixed key extractor.
        ///
        /// `key` may be an attribute name, a one-argument callable, or `None`
        /// to compare values directly. Duplicate keys are stored only once.
        #[new]
        #[pyo3(signature = (values = None, *, key = None))]
        fn new(
            py: Python<'_>,
            values: Option<PythonValues>,
            key: Option<KeyExtractor>,
        ) -> PyResult<Self> {
            let key = key.unwrap_or(KeyExtractor::Identity);
            let mut inner = PythonAvlTree::default();
            if let Some(values) = values {
                for value in values.0 {
                    let extracted = key.extract(py, &value)?;
                    inner.insert(py, Entry::new(value, extracted))?;
                }
            }
            Ok(Self { inner, key })
        }

        /// Insert `value` and return `True` if its key was not already present.
        fn insert(&mut self, py: Python<'_>, value: Py<PyAny>) -> PyResult<bool> {
            let key = self.key.extract(py, &value)?;
            self.inner.insert(py, Entry::new(value, key))
        }

        /// Remove the entry matching `value`'s extracted key.
        fn remove(&mut self, py: Python<'_>, value: Py<PyAny>) -> PyResult<bool> {
            let key = self.key.extract(py, &value)?;
            self.inner.remove(py, &key)
        }

        /// Remove the entry matching an already-extracted key.
        fn remove_key(&mut self, py: Python<'_>, key: Py<PyAny>) -> PyResult<bool> {
            self.inner.remove(py, &key)
        }

        /// Return the entry matching `value`'s extracted key, or `None`.
        fn search(&self, py: Python<'_>, value: Py<PyAny>) -> PyResult<Option<Py<PyAny>>> {
            let key = self.key.extract(py, &value)?;
            self.inner.search(py, &key)
        }

        /// Alias for `search`.
        fn get(&self, py: Python<'_>, value: Py<PyAny>) -> PyResult<Option<Py<PyAny>>> {
            self.search(py, value)
        }

        /// Return the entry matching an already-extracted key, or `None`.
        fn search_key(&self, py: Python<'_>, key: Py<PyAny>) -> PyResult<Option<Py<PyAny>>> {
            self.inner.search(py, &key)
        }

        /// Return whether an entry matching `value`'s extracted key exists.
        fn contains(&self, py: Python<'_>, value: Py<PyAny>) -> PyResult<bool> {
            Ok(self.search(py, value)?.is_some())
        }

        /// Compatibility alias for `contains`.
        fn has_node(&self, py: Python<'_>, value: Py<PyAny>) -> PyResult<bool> {
            self.contains(py, value)
        }

        /// Return whether an already-extracted key exists.
        fn contains_key(&self, py: Python<'_>, key: Py<PyAny>) -> PyResult<bool> {
            Ok(self.inner.search(py, &key)?.is_some())
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

        /// Return the value with the smallest key, or `None` when empty.
        fn first(&self, py: Python<'_>) -> Option<Py<PyAny>> {
            self.inner.first(py)
        }

        /// Alias for `first`.
        fn min(&self, py: Python<'_>) -> Option<Py<PyAny>> {
            self.first(py)
        }

        /// Return the value with the largest key, or `None` when empty.
        fn last(&self, py: Python<'_>) -> Option<Py<PyAny>> {
            self.inner.last(py)
        }

        /// Alias for `last`.
        fn max(&self, py: Python<'_>) -> Option<Py<PyAny>> {
            self.last(py)
        }

        /// Return values whose extracted keys fall between optional endpoints.
        ///
        /// Endpoints are already-extracted keys. The start is inclusive and the
        /// end is exclusive by default. Invalid or incomparable keys raise.
        #[pyo3(signature = (start = None, end = None, *, include_start = true, include_end = false))]
        fn range(
            &self,
            py: Python<'_>,
            start: Option<Py<PyAny>>,
            end: Option<Py<PyAny>>,
            include_start: bool,
            include_end: bool,
        ) -> PyResult<PyAVLTreeIterator> {
            Ok(PyAVLTreeIterator::new(self.inner.range(
                py,
                start.as_ref(),
                end.as_ref(),
                include_start,
                include_end,
            )?))
        }

        /// Return at most `count` values from an inclusive lower-bound key.
        /// `start` is an already-extracted key. If it is absent from the tree,
        /// iteration begins at the first greater key.
        fn iter_from(
            &self,
            py: Python<'_>,
            start: Py<PyAny>,
            count: usize,
        ) -> PyResult<PyAVLTreeIterator> {
            Ok(PyAVLTreeIterator::new(
                self.inner.iter_from(py, &start, count)?,
            ))
        }

        /// Return at most `count` values from an inclusive upper-bound key.
        /// Values are returned in descending order. `end` is an already-
        /// extracted key; if absent, iteration begins at the first smaller key.
        fn iter_to(
            &self,
            py: Python<'_>,
            end: Py<PyAny>,
            count: usize,
        ) -> PyResult<PyAVLTreeIterator> {
            Ok(PyAVLTreeIterator::new(self.inner.iter_to(py, &end, count)?))
        }

        /// Return a snapshot iterator in ascending key order.
        fn in_order(&self, py: Python<'_>) -> PyAVLTreeIterator {
            PyAVLTreeIterator::new(self.inner.in_order(py))
        }

        /// Return a snapshot iterator in descending key order.
        fn descending(&self, py: Python<'_>) -> PyAVLTreeIterator {
            PyAVLTreeIterator::new(self.inner.descending(py))
        }

        /// Return a snapshot iterator in root-left-right order.
        fn pre_order(&self, py: Python<'_>) -> PyAVLTreeIterator {
            PyAVLTreeIterator::new(self.inner.pre_order(py))
        }

        /// Return a snapshot iterator in left-right-root order.
        fn post_order(&self, py: Python<'_>) -> PyAVLTreeIterator {
            PyAVLTreeIterator::new(self.inner.post_order(py))
        }

        /// Return a breadth-first snapshot iterator, level by level.
        fn level_order(&self, py: Python<'_>) -> PyAVLTreeIterator {
            PyAVLTreeIterator::new(self.inner.level_order(py))
        }

        /// Return the number of unique keys in the tree.
        fn __len__(&self) -> usize {
            self.inner.len()
        }

        /// Return `True` when the tree is non-empty.
        fn __bool__(&self) -> bool {
            !self.inner.is_empty()
        }

        /// Implement `value in tree` using the value's extracted key.
        fn __contains__(&self, py: Python<'_>, value: Py<PyAny>) -> PyResult<bool> {
            self.contains(py, value)
        }

        /// Iterate over a snapshot of values in ascending key order.
        fn __iter__(&self, py: Python<'_>) -> PyAVLTreeIterator {
            self.in_order(py)
        }

        /// Return an ascending representation of the stored values.
        fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
            let representations = self
                .inner
                .in_order(py)
                .into_iter()
                .map(|value| Ok(value.bind(py).repr()?.to_str()?.to_owned()))
                .collect::<PyResult<Vec<_>>>()?;
            Ok(format!("AVLTree([{}])", representations.join(", ")))
        }

        /// Return `(callable, args)` for pickle reconstruction.
        ///
        /// The tree is reduced to a sorted list of `(value, key)` pairs plus
        /// the key extractor. Reconstruction uses the fast O(n) sorted builder
        /// so unpickling does not perform AVL insertion or Python comparisons.
        fn __reduce__(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
            // Collect sorted (value, key) pairs — keys are pre-computed and
            // stored on each node, so no key extractor call is needed here.
            let pairs_data = self.inner.in_order_entries(py);
            let pairs: Vec<Py<PyAny>> = pairs_data
                .into_iter()
                .map(|(value, key)| {
                    PyTuple::new(py, [value.into_bound(py), key.into_bound(py)])
                        .map(|t| t.into_any().unbind())
                })
                .collect::<PyResult<_>>()?;
            let pairs_list = PyList::new(py, pairs)?.unbind();

            // Encode the key extractor as None | str | callable so it can be
            // stored alongside the values and restored without ambiguity.
            let key_spec: Py<PyAny> = match &self.key {
                KeyExtractor::Identity => py.None(),
                KeyExtractor::Attribute(name) => PyString::new(py, name).unbind().into_any(),
                KeyExtractor::Callable(callable) => callable.clone_ref(py),
            };

            // Retrieve the reconstruction helper from the installed module so
            // pickle can locate it by qualified name on deserialization.
            let module = py.import("rs_avl.rs_avl")?;
            let reconstruct = module.getattr("_avltree_from_sorted_entries")?;

            // Return (reconstruct_fn, (pairs_list, key_spec))
            let inner_args = PyTuple::new(py, [pairs_list.bind(py).as_any(), key_spec.bind(py)])?;
            PyTuple::new(py, [reconstruct.as_any(), inner_args.as_any()]).map(|t| t.unbind())
        }
    }

    /// An iterator over a snapshot of tree values.
    ///
    /// Mutating or clearing the source tree does not invalidate this iterator.
    #[pyclass(name = "_AVLTreeIterator", module = "rs_avl")]
    struct PyAVLTreeIterator {
        values: std::vec::IntoIter<Py<PyAny>>,
    }

    impl PyAVLTreeIterator {
        fn new(values: Vec<Py<PyAny>>) -> Self {
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
        fn __next__(&mut self) -> PyResult<Py<PyAny>> {
            self.values
                .next()
                .ok_or_else(|| PyStopIteration::new_err(()))
        }
    }

    /// Private reconstruction helper used by pickle.
    ///
    /// Accepts a list of `(value, key)` pairs in **strictly ascending key
    /// order** and the original key extractor. Nodes are allocated directly
    /// via the O(n) balanced builder — no AVL insertion or Python comparison
    /// work is performed during unpickling.
    ///
    /// This function is an implementation detail; its name and signature may
    /// change across versions without notice.
    #[pyfunction]
    fn _avltree_from_sorted_entries(
        _py: Python<'_>,
        pairs: Vec<(Py<PyAny>, Py<PyAny>)>,
        key_spec: Option<KeyExtractor>,
    ) -> PyResult<PyAVLTree> {
        let key = key_spec.unwrap_or(KeyExtractor::Identity);
        let entries: Vec<Entry> = pairs
            .into_iter()
            .map(|(value, extracted_key)| Entry::new(value, extracted_key))
            .collect();
        let inner = PythonAvlTree::from_sorted(entries);
        Ok(PyAVLTree { inner, key })
    }
}
