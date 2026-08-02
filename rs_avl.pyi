"""
Python bindings for `rs-avl`.
"""

from typing import Any, Final, Iterable, final

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
    A height-balanced ordered set of signed 64-bit integers.
    
    Values are unique, iteration is ascending, and search, insertion, and
    removal take logarithmic time while the tree remains balanced.
    """
    def __bool__(self, /) -> bool:
        """
        Return `True` when the tree is non-empty.
        """
    def __contains__(self, /, value: Any) -> bool:
        """
        Implement the `value in tree` membership operation.
        """
    def __iter__(self, /) -> _AVLTreeIterator:
        """
        Iterate over a snapshot of the values in ascending order.
        """
    def __len__(self, /) -> int:
        """
        Return the number of unique values in the tree.
        """
    def __new__(cls, /, values: Iterable[int] |None = None) -> AVLTree:
        """
        Create a tree from an optional iterable of integers.
        
        Duplicate values from the iterable are stored only once.
        """
    def __repr__(self, /) -> str:
        """
        Return an unambiguous ascending representation of the tree.
        """
    def clear(self, /) -> None:
        """
        Remove every value from the tree.
        """
    def contains(self, /, value: int) -> bool:
        """
        Return whether `value` is present in the tree.
        """
    def first(self, /) -> int |None:
        """
        Return the smallest value, or `None` when empty.
        """
    def get(self, /, value: int) -> int |None:
        """
        Alias for `search`.
        """
    def has_node(self, /, value: int) -> bool:
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
        Return a snapshot iterator in ascending order.
        """
    def insert(self, /, value: int) -> bool:
        """
        Insert `value` and return `True` if it was not already present.
        """
    def is_empty(self, /) -> bool:
        """
        Return whether the tree contains no values.
        """
    def last(self, /) -> int |None:
        """
        Return the largest value, or `None` when empty.
        """
    def level_order(self, /) -> _AVLTreeIterator:
        """
        Return a breadth-first snapshot iterator, level by level.
        """
    def max(self, /) -> int |None:
        """
        Alias for `last`.
        """
    def min(self, /) -> int |None:
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
    def range(self, /, start: int |None = None, end: int |None = None, *, include_start: bool = True, include_end: bool = False) -> _AVLTreeIterator:
        """
        Return an ascending snapshot iterator between optional endpoints.
        
        The start is inclusive and the end is exclusive by default. Use
        `include_start` and `include_end` to change either boundary.
        Raises `ValueError` when `start` is greater than `end`.
        """
    def remove(self, /, value: int) -> bool:
        """
        Remove `value` and return `True` if it was present.
        """
    def search(self, /, value: int) -> int |None:
        """
        Return the stored value equal to `value`, or `None` when absent.
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
    def __next__(self, /) -> int:
        """
        Return the next value, raising `StopIteration` when exhausted.
        """
