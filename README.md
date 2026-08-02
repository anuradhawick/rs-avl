# rs-avl

`rs-avl` is a height-balanced ordered set for Rust and Python. The Rust crate
provides a generic `AvlTree<T: Ord>`; the Python extension stores arbitrary
comparable objects and supports attribute-name or callable key extraction.

- Rust package: `rs-avl` on crates.io
- Python package: `rs-avl` on PyPI, imported as `rs_avl`
- Rust API documentation: [docs.rs/rs-avl](https://docs.rs/rs-avl)

## Highlights

- Logarithmic insertion, removal, and search
- Unique-value ordered-set semantics
- Inclusive and exclusive range queries
- In-order, pre-order, post-order, and level-order traversals
- Generic borrowed-key lookup in Rust
- Arbitrary comparable Python objects with `key="attribute"` or `key=callable`
- Python type stubs generated from the compiled PyO3 API

See [rust_readme.md](rust_readme.md) for the Rust API and
[python_readme.md](python_readme.md) for the Python API.
