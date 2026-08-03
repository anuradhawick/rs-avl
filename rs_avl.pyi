"""
Python bindings for `rs-avl`.
"""

from typing import Any, Callable, Final, Iterable, final

__all__: Final[list[str]]
"""
Names exported by `from rs_avl import *`.
"""

__version__: Final[str]
"""
The installed `rs-avl` package version.
"""

@final
class AVLTree:
    """
    A height-balanced ordered set of comparable Python objects.
    
    Values are ordered directly unless `key` is an attribute name or a
    callable. Equal keys are treated as duplicate set entries.
    """
    def __bool__(self, /) -> bool:
        """
        Return `True` when the tree is non-empty.
        """
    def __contains__(self, /, value: Any) -> bool:
        """
        Implement `value in tree` using the value's extracted key.
        """
    def __iter__(self, /) -> _AVLTreeIterator:
        """
        Iterate over a snapshot of values in ascending key order.
        """
    def __len__(self, /) -> int:
        """
        Return the number of unique keys in the tree.
        """
    def __new__(cls, /, values: Iterable[Any] |None = None, *, key: str |Callable[[Any], Any] |None = None) -> AVLTree:
        """
        Create a tree from optional values and a fixed key extractor.
        
        `key` may be an attribute name, a one-argument callable, or `None`
        to compare values directly. Duplicate keys are stored only once.
        """
    def __repr__(self, /) -> str:
        """
        Return an ascending representation of the stored values.
        """
    def clear(self, /) -> None:
        """
        Remove every value from the tree.
        """
    def contains(self, /, value: Any) -> bool:
        """
        Return whether an entry matching `value`'s extracted key exists.
        """
    def contains_key(self, /, key: Any) -> bool:
        """
        Return whether an already-extracted key exists.
        """
    def first(self, /) -> Any |None:
        """
        Return the value with the smallest key, or `None` when empty.
        """
    def get(self, /, value: Any) -> Any |None:
        """
        Alias for `search`.
        """
    def has_node(self, /, value: Any) -> bool:
        """
        Compatibility alias for `contains`.
        """
    @property
    def height(self, /) -> int:
        """
        The height of the tree; an empty tree has height zero.
        """
    def in_order(self, /) -> _AVLTreeIterator:
        """
        Return a snapshot iterator in ascending key order.
        """
    def insert(self, /, value: Any) -> bool:
        """
        Insert `value` and return `True` if its key was not already present.
        """
    def is_empty(self, /) -> bool:
        """
        Return whether the tree contains no values.
        """
    def iter_from(self, /, start: Any, count: int) -> _AVLTreeIterator:
        """
        Return at most `count` values from an inclusive lower-bound key.
        `start` is an already-extracted key. If it is absent from the tree,
        iteration begins at the first greater key.
        """
    def last(self, /) -> Any |None:
        """
        Return the value with the largest key, or `None` when empty.
        """
    def level_order(self, /) -> _AVLTreeIterator:
        """
        Return a breadth-first snapshot iterator, level by level.
        """
    def max(self, /) -> Any |None:
        """
        Alias for `last`.
        """
    def min(self, /) -> Any |None:
        """
        Alias for `first`.
        """
    def post_order(self, /) -> _AVLTreeIterator:
        """
        Return a snapshot iterator in left-right-root order.
        """
    def pre_order(self, /) -> _AVLTreeIterator:
        """
        Return a snapshot iterator in root-left-right order.
        """
    def range(self, /, start: Any |None = None, end: Any |None = None, *, include_start: bool = True, include_end: bool = False) -> _AVLTreeIterator:
        """
        Return values whose extracted keys fall between optional endpoints.
        
        Endpoints are already-extracted keys. The start is inclusive and the
        end is exclusive by default. Invalid or incomparable keys raise.
        """
    def remove(self, /, value: Any) -> bool:
        """
        Remove the entry matching `value`'s extracted key.
        """
    def remove_key(self, /, key: Any) -> bool:
        """
        Remove the entry matching an already-extracted key.
        """
    def search(self, /, value: Any) -> Any |None:
        """
        Return the entry matching `value`'s extracted key, or `None`.
        """
    def search_key(self, /, key: Any) -> Any |None:
        """
        Return the entry matching an already-extracted key, or `None`.
        """

@final
class _AVLTreeIterator:
    """
    An iterator over a snapshot of tree values.
    
    Mutating or clearing the source tree does not invalidate this iterator.
    """
    def __iter__(self, /) -> _AVLTreeIterator:
        """
        Return this iterator object.
        """
    def __next__(self, /) -> Any:
        """
        Return the next value, raising `StopIteration` when exhausted.
        """
