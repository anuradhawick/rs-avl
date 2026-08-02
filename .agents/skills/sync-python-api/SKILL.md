---
name: sync-python-api
description: Keep this repository's PyO3 Python API aligned with changes to the generic Rust AVL API. Use when modifying public methods, names, semantics, traversals, ranges, node behavior, or documentation under src/avl; when reviewing whether src/python.rs and src/python_tree.rs still mirror the Rust surface; or when regenerating rs_avl.pyi after binding changes.
---

# Sync Python API

Preserve intentional differences between the generic Rust ordered set and the
fallible Python-object implementation while keeping their public behavior
aligned.

## Workflow

1. Inspect the Rust change in `src/avl/`, including its tests and public docs.
2. Inventory the corresponding Python surface in `src/python.rs` and storage
   behavior in `src/python_tree.rs`.
3. Update Python behavior, PyO3 signatures, and Rust doc comments together.
4. Preserve these intentional Python differences:
   - Store owned `Py<PyAny>` values and cached extracted keys.
   - Accept `key=None`, an attribute name, or a callable.
   - Return `PyResult` wherever Python extraction or comparison can fail.
   - Leave the tree unchanged when a comparison raises.
   - Keep `search_key`, `contains_key`, `remove_key`, and key-bounded `range`.
5. Add or update black-box cases in `tests/test_python.py`. Cover direct
   comparison, attribute keys, callable/composite keys, duplicates, exceptions,
   traversal order, and snapshot iterators as relevant.
6. Regenerate the checked-in stub; never hand-edit it:

   ```bash
   maturin generate-stubs --out .
   ```

7. Inspect `rs_avl.pyi` for accurate types and docstrings. Add PyO3 inspection
   metadata in Rust when inference produces `Incomplete` or an invalid type.
8. Validate the complete dual API:

   ```bash
   cargo fmt --all --check
   cargo clippy --all-targets -- -D warnings
   cargo test --all-targets
   maturin develop --generate-stubs
   python -m pytest -q tests/test_python.py
   ```

## Guardrails

- Do not make the generic Rust tree depend on Python.
- Do not implement Python comparison through Rust `Ord`; Python ordering is
  fallible and can raise `TypeError` or reject non-total values such as NaN.
- Do not expose mutable nodes that could violate ordering or height invariants.
- Treat equal keys as duplicates unless the requested API explicitly changes
  ordered-set semantics in both implementations.
- Update `python_readme.md` and `examples/python_avl.py` when user-facing Python
  behavior changes.
