import rs_avl
import pytest


def test_python_tree_api() -> None:
    tree = rs_avl.AVLTree(value for value in [4, 2, 6, 1, 3, 5, 7, 4])

    assert list(tree) == [1, 2, 3, 4, 5, 6, 7]
    assert len(tree) == 7
    assert tree.height == 3
    assert tree.search(5) == 5
    assert tree.get(99) is None
    assert tree.has_node(3)
    assert 6 in tree
    with pytest.raises(TypeError):
        assert "6" not in tree
    assert tree.first() == tree.min() == 1
    assert tree.last() == tree.max() == 7
    assert repr(tree) == "AVLTree([1, 2, 3, 4, 5, 6, 7])"

    assert list(tree.range(2, 6)) == [2, 3, 4, 5]
    assert list(tree.range(2, 6, include_start=False, include_end=True)) == [3, 4, 5, 6]
    assert list(tree.iter_from(3, 3)) == [3, 4, 5]
    assert list(tree.iter_from(3.5, 10)) == [4, 5, 6, 7]
    assert list(tree.iter_from(8, 3)) == []
    assert list(tree.iter_from("not compared", 0)) == []
    assert list(tree.iter_to(5, 3)) == [5, 4, 3]
    assert list(tree.iter_to(4.5, 10)) == [4, 3, 2, 1]
    assert list(tree.iter_to(0, 3)) == []
    assert list(tree.iter_to("not compared", 0)) == []
    assert list(tree.descending()) == [7, 6, 5, 4, 3, 2, 1]
    assert list(tree.pre_order()) == [4, 2, 1, 3, 6, 5, 7]
    assert list(tree.post_order()) == [1, 3, 2, 5, 7, 6, 4]
    assert list(tree.level_order()) == [4, 2, 6, 1, 3, 5, 7]

    iterator = iter(tree)
    cursor = tree.iter_from(3, 3)
    reverse_cursor = tree.iter_to(5, 3)
    descending = tree.descending()
    tree.clear()
    assert list(iterator) == [1, 2, 3, 4, 5, 6, 7]
    assert list(cursor) == [3, 4, 5]
    assert list(reverse_cursor) == [5, 4, 3]
    assert list(descending) == [7, 6, 5, 4, 3, 2, 1]
    assert not tree


def test_invalid_python_range() -> None:
    tree = rs_avl.AVLTree([1, 2, 3])

    try:
        list(tree.range(3, 1))
    except ValueError as error:
        assert str(error) == "range start must not exceed end"
    else:
        raise AssertionError("expected a ValueError")


class Task:
    def __init__(self, name: str, priority: int) -> None:
        self.name = name
        self.priority = priority

    def __repr__(self) -> str:
        return f"Task({self.name!r}, {self.priority})"


def test_attribute_key_supports_arbitrary_objects() -> None:
    low = Task("documentation", 3)
    urgent = Task("release", 1)
    normal = Task("testing", 2)
    duplicate = Task("duplicate", 2)
    tree = rs_avl.AVLTree([low, urgent, normal], key="priority")

    assert list(tree) == [urgent, normal, low]
    assert not tree.insert(duplicate)
    assert tree.search(Task("probe", 2)) is normal
    assert tree.search_key(3) is low
    assert tree.contains_key(1)
    assert list(tree.range(1, 3)) == [urgent, normal]
    assert list(tree.iter_from(2, 2)) == [normal, low]
    assert list(tree.iter_to(2, 2)) == [normal, urgent]
    assert tree.remove_key(2)
    assert list(tree) == [urgent, low]


def test_callable_key_can_return_composite_keys() -> None:
    first = Task("beta", 1)
    second = Task("alpha", 1)
    third = Task("gamma", 2)
    tree = rs_avl.AVLTree(
        [first, second, third],
        key=lambda task: (task.priority, task.name),
    )

    assert list(tree) == [second, first, third]
    assert tree.search_key((1, "beta")) is first
    assert tree.remove(Task("beta", 1))
    assert list(tree) == [second, third]


class Comparable:
    def __init__(self, rank: int) -> None:
        self.rank = rank

    def __eq__(self, other: object) -> bool:
        return isinstance(other, Comparable) and self.rank == other.rank

    def __lt__(self, other: object) -> bool:
        if not isinstance(other, Comparable):
            return NotImplemented
        return self.rank < other.rank


def test_direct_python_rich_comparison() -> None:
    values = [Comparable(3), Comparable(1), Comparable(2)]
    tree = rs_avl.AVLTree(values)

    assert [value.rank for value in tree] == [1, 2, 3]
    assert tree.search(Comparable(2)) is values[2]


def test_comparison_and_key_errors_leave_existing_tree_unchanged() -> None:
    tree = rs_avl.AVLTree([1, 2, 3])

    with pytest.raises(TypeError):
        tree.insert("incomparable")
    assert list(tree) == [1, 2, 3]

    with pytest.raises(TypeError):
        tree.iter_from("incomparable", 1)
    assert list(tree) == [1, 2, 3]

    with pytest.raises(TypeError):
        tree.iter_to("incomparable", 1)
    assert list(tree) == [1, 2, 3]

    with pytest.raises(TypeError, match="key must be"):
        rs_avl.AVLTree(key=42)
    with pytest.raises(AttributeError):
        rs_avl.AVLTree([Task("missing", 1)], key="unknown")


def test_pickle_round_trip_identity_key() -> None:
    import pickle

    tree = rs_avl.AVLTree([3, 1, 4, 1, 5, 9, 2, 6])
    restored = pickle.loads(pickle.dumps(tree))

    assert list(restored) == list(tree)
    assert len(restored) == len(tree)


def test_pickle_round_trip_attribute_key() -> None:
    import pickle

    low = Task("documentation", 3)
    urgent = Task("release", 1)
    normal = Task("testing", 2)
    tree = rs_avl.AVLTree([low, urgent, normal], key="priority")
    restored = pickle.loads(pickle.dumps(tree))

    assert [t.name for t in restored] == [t.name for t in tree]
    assert restored.first().name == "release"


def priority_key(task: Task) -> int:
    return task.priority


def test_pickle_round_trip_callable_key() -> None:
    import pickle

    low = Task("documentation", 3)
    urgent = Task("release", 1)
    normal = Task("testing", 2)
    # Use a module-level function so it is picklable
    tree = rs_avl.AVLTree([low, urgent, normal], key=priority_key)
    restored = pickle.loads(pickle.dumps(tree))

    assert [t.name for t in restored] == [t.name for t in tree]


def test_pickle_round_trip_empty_tree() -> None:
    import pickle

    tree: rs_avl.AVLTree = rs_avl.AVLTree()
    restored = pickle.loads(pickle.dumps(tree))

    assert list(restored) == []
    assert len(restored) == 0
    assert restored.is_empty()


def test_pickle_fast_path_bypasses_avl_insertion() -> None:
    """Unpickling must not call insert() or perform AVL comparisons.

    We verify this by patching AVLTree.insert: if the fast path is taken,
    insert is never invoked during reconstruction.
    """
    import pickle

    tree = rs_avl.AVLTree([1, 2, 3, 4, 5])

    insert_call_count = 0
    original_insert = rs_avl.AVLTree.insert

    def counting_insert(self: rs_avl.AVLTree, value: object) -> bool:
        nonlocal insert_call_count
        insert_call_count += 1
        return original_insert(self, value)

    rs_avl.AVLTree.insert = counting_insert  # type: ignore[method-assign]
    try:
        restored = pickle.loads(pickle.dumps(tree))
    finally:
        rs_avl.AVLTree.insert = original_insert  # type: ignore[method-assign]

    # Reconstruction must use the fast builder, not insert().
    assert insert_call_count == 0
    assert list(restored) == [1, 2, 3, 4, 5]
