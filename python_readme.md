# rs-avl for Python

A fast AVL ordered set backed by Rust, with support for arbitrary comparable
Python objects.

## Installation

```bash
pip install rs-avl
```

## Comparable values

```python
from rs_avl import AVLTree

tree = AVLTree([4, 2, 6, 1, 3, 5])
tree.insert(7)

assert list(tree) == [1, 2, 3, 4, 5, 6, 7]
assert tree.search(3) == 3
assert list(tree.range(2, 6)) == [2, 3, 4, 5]
```

## Objects ordered by an attribute

```python
from dataclasses import dataclass
from rs_avl import AVLTree

@dataclass
class Task:
    name: str
    priority: int

tasks = AVLTree(
    [Task("document", 2), Task("release", 1)],
    key="priority",
)

assert tasks.first().name == "release"
assert tasks.search_key(2).name == "document"
```

`key` can also be a callable, including a lambda returning a composite key:

```python
tasks = AVLTree(key=lambda task: (task.priority, task.name))
```

Equal keys are treated as duplicates. Extracted keys should remain comparable
for as long as their values are stored. Type information is included through a
generated `.pyi` file and `py.typed` marker.
