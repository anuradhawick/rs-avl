from dataclasses import dataclass

from rs_avl import AVLTree


@dataclass
class Task:
    name: str
    priority: int


tasks = AVLTree(
    [Task("documentation", 2), Task("release", 1), Task("cleanup", 3)],
    key="priority",
)

print("two from priority 2:", list(tasks.iter_from(2, 2)))

print("priority order:", list(tasks))
print("priority 2:", tasks.search_key(2))
print("priorities [1, 3):", list(tasks.range(1, 3)))
