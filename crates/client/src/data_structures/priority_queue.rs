// /!\ This priority queue implementation is made for A* search algorithm and not for general purpose /!\

use shared::maze::Cell;
use std::cmp::{Ord, Ordering};
use std::collections::{BinaryHeap, HashSet};

#[derive(PartialEq, Eq)]
pub struct Node {
    pub priority_f: i32,
    pub cell: Cell,
}

impl Ord for Node {
    fn cmp(&self, other: &Node) -> Ordering {
        self.priority_f.cmp(&other.priority_f).reverse()
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
#[derive(Default)]
pub struct PriorityQueue {
    heap: BinaryHeap<Node>,
    cells: HashSet<Cell>,
}

impl PriorityQueue {
    pub fn new() -> Self {
        Self { heap: BinaryHeap::new(), cells: HashSet::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn enqueue(&mut self, node: Node) {
        self.cells.insert(node.cell);
        self.heap.push(node);
    }

    pub fn dequeue(&mut self) -> Node {
        let n: Node = self.heap.pop().expect("empty queue");
        self.cells.remove(&n.cell);
        n
    }

    pub fn contains(&self, target_cell: &Cell) -> bool {
        self.cells.contains(target_cell)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_priority_queue_empty() {
        let priority_queue = PriorityQueue::new();

        assert_eq!(priority_queue.is_empty(), true);
    }

    #[test]
    fn test_enqueue_and_dequeue_single_node() {
        let mut priority_queue = PriorityQueue::new();

        let node = Node { priority_f: 1, cell: Cell { row: 0, column: 0 } };

        priority_queue.enqueue(node);

        assert_eq!(priority_queue.is_empty(), false);

        let dequeued_node = priority_queue.dequeue();
        assert_eq!(dequeued_node.priority_f, 1);
    }

    #[test]
    fn test_enqueue_and_dequeue_multiple_nodes() {
        let mut priority_queue = PriorityQueue::new();

        let node1 = Node { priority_f: 3, cell: Cell { row: 1, column: 1 } };
        let node2 = Node { priority_f: 1, cell: Cell { row: 2, column: 2 } };
        let node3 = Node { priority_f: 2, cell: Cell { row: 3, column: 3 } };

        priority_queue.enqueue(node1);
        priority_queue.enqueue(node2);
        priority_queue.enqueue(node3);

        let dequeued_node = priority_queue.dequeue();
        assert_eq!(dequeued_node.priority_f, 1);

        let dequeued_node = priority_queue.dequeue();
        assert_eq!(dequeued_node.priority_f, 2);

        let dequeued_node = priority_queue.dequeue();
        assert_eq!(dequeued_node.priority_f, 3);

        assert_eq!(priority_queue.is_empty(), true);
    }

    #[test]
    fn test_priority_with_same_values() {
        let mut priority_queue = PriorityQueue::new();

        let node1 = Node { priority_f: 3, cell: Cell { row: 1, column: 1 } };
        let node2 = Node { priority_f: 3, cell: Cell { row: 2, column: 2 } };
        let node3 = Node { priority_f: 2, cell: Cell { row: 3, column: 3 } };

        priority_queue.enqueue(node2);
        priority_queue.enqueue(node3);
        priority_queue.enqueue(node1);

        let dequeued_node = priority_queue.dequeue();
        assert_eq!(dequeued_node.priority_f, 2);
        let dequeued_node = priority_queue.dequeue();
        assert_eq!(dequeued_node.priority_f, 3);
        let dequeued_node = priority_queue.dequeue();
        assert_eq!(dequeued_node.priority_f, 3);

        assert_eq!(priority_queue.is_empty(), true);
    }

    #[test]
    #[should_panic(expected = "empty queue")]
    fn test_dequeue_from_empty_queue() {
        let mut priority_queue = PriorityQueue::new();

        priority_queue.dequeue();
    }

    #[test]
    fn test_contain_cell() {
        let mut priority_queue = PriorityQueue::new();

        let node1 = Node { priority_f: 3, cell: Cell { row: 43, column: 12 } };
        priority_queue.enqueue(node1);

        assert_eq!(priority_queue.contains(&Cell { row: 43, column: 12 }), true);

        priority_queue.dequeue();

        assert_eq!(priority_queue.contains(&Cell { row: 43, column: 12 }), false);

        assert_eq!(priority_queue.contains(&Cell { row: 4, column: 4 }), false);
    }
}
