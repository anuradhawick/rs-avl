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
    assert list(tree.pre_order()) == [4, 2, 1, 3, 6, 5, 7]
    assert list(tree.post_order()) == [1, 3, 2, 5, 7, 6, 4]
    assert list(tree.level_order()) == [4, 2, 6, 1, 3, 5, 7]

    iterator = iter(tree)
    cursor = tree.iter_from(3, 3)
    tree.clear()
    assert list(iterator) == [1, 2, 3, 4, 5, 6, 7]
    assert list(cursor) == [3, 4, 5]
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

    with pytest.raises(TypeError, match="key must be"):
        rs_avl.AVLTree(key=42)
    with pytest.raises(AttributeError):
        rs_avl.AVLTree([Task("missing", 1)], key="unknown")
