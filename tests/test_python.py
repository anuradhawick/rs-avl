import rs_avl


def test_python_tree_api() -> None:
    tree = rs_avl.AVLTree(value for value in [4, 2, 6, 1, 3, 5, 7, 4])

    assert list(tree) == [1, 2, 3, 4, 5, 6, 7]
    assert len(tree) == 7
    assert tree.height == 3
    assert tree.search(5) == 5
    assert tree.get(99) is None
    assert tree.has_node(3)
    assert 6 in tree
    assert "6" not in tree
    assert tree.first() == tree.min() == 1
    assert tree.last() == tree.max() == 7
    assert repr(tree) == "AVLTree([1, 2, 3, 4, 5, 6, 7])"

    assert list(tree.range(2, 6)) == [2, 3, 4, 5]
    assert list(tree.range(2, 6, include_start=False, include_end=True)) == [3, 4, 5, 6]
    assert list(tree.pre_order()) == [4, 2, 1, 3, 6, 5, 7]
    assert list(tree.post_order()) == [1, 3, 2, 5, 7, 6, 4]
    assert list(tree.level_order()) == [4, 2, 6, 1, 3, 5, 7]

    iterator = iter(tree)
    tree.clear()
    assert list(iterator) == [1, 2, 3, 4, 5, 6, 7]
    assert not tree


def test_invalid_python_range() -> None:
    tree = rs_avl.AVLTree([1, 2, 3])

    try:
        list(tree.range(3, 1))
    except ValueError as error:
        assert str(error) == "range start must not exceed end"
    else:
        raise AssertionError("expected a ValueError")
