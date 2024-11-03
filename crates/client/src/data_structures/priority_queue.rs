// /!\ This priority queue implementation is made for A* search algorithm and not for general purpose /!\

use crate::maze::Cell;
use std::cmp::{Ord, Ordering};
use std::collections::BinaryHeap;

#[derive(PartialEq, Eq)]
pub struct Node {
    f: i32,
    g: i32,
    h: i32,
    cell: Cell,
}

impl Ord for Node {
    fn cmp(&self, other: &Node) -> Ordering {
        self.f
            .cmp(&other.f)
            .then_with(|| self.g.cmp(&other.g))
            .then_with(|| self.h.cmp(&other.h))
            .reverse()
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
}

impl PriorityQueue {
    pub fn new() -> Self {
        Self { heap: BinaryHeap::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn enqueue(&mut self, node: Node) {
        self.heap.push(node);
    }

    pub fn dequeue(&mut self) -> Node {
        self.heap.pop().expect("empty queue")
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

        let node = Node { f: 1, g: 2, h: 3, cell: Cell { row: 0, column: 0 } };

        priority_queue.enqueue(node);

        assert_eq!(priority_queue.is_empty(), false);

        let dequeued_node = priority_queue.dequeue();
        assert_eq!(dequeued_node.f, 1);
        assert_eq!(dequeued_node.g, 2);
        assert_eq!(dequeued_node.h, 3);
    }

    #[test]
    fn test_enqueue_and_dequeue_multiple_nodes() {
        let mut priority_queue = PriorityQueue::new();

        let node1 = Node { f: 3, g: 2, h: 1, cell: Cell { row: 1, column: 1 } };
        let node2 = Node { f: 1, g: 4, h: 3, cell: Cell { row: 2, column: 2 } };
        let node3 = Node { f: 2, g: 3, h: 4, cell: Cell { row: 3, column: 3 } };

        priority_queue.enqueue(node1);
        priority_queue.enqueue(node2);
        priority_queue.enqueue(node3);

        let dequeued_node = priority_queue.dequeue();
        assert_eq!(dequeued_node.f, 1);
        assert_eq!(dequeued_node.g, 4);
        assert_eq!(dequeued_node.h, 3);

        let dequeued_node = priority_queue.dequeue();
        assert_eq!(dequeued_node.f, 2);

        let dequeued_node = priority_queue.dequeue();
        assert_eq!(dequeued_node.f, 3);

        assert_eq!(priority_queue.is_empty(), true);
    }

    #[test]
    fn test_priority_with_same_values() {
        let mut priority_queue = PriorityQueue::new();

        let node1 = Node { f: 3, g: 2, h: 1, cell: Cell { row: 1, column: 1 } };
        let node2 = Node { f: 3, g: 4, h: 1, cell: Cell { row: 2, column: 2 } };
        let node3 = Node { f: 3, g: 4, h: 2, cell: Cell { row: 3, column: 3 } };

        priority_queue.enqueue(node2);
        priority_queue.enqueue(node3);
        priority_queue.enqueue(node1);

        let dequeued_node = priority_queue.dequeue();
        assert_eq!(dequeued_node.cell, Cell { row: 1, column: 1 });
        let dequeued_node = priority_queue.dequeue();
        assert_eq!(dequeued_node.cell, Cell { row: 2, column: 2 });
        let dequeued_node = priority_queue.dequeue();
        assert_eq!(dequeued_node.cell, Cell { row: 3, column: 3 });

        assert_eq!(priority_queue.is_empty(), true);
    }

    #[test]
    #[should_panic(expected = "empty queue")]
    fn test_dequeue_from_empty_queue() {
        let mut priority_queue = PriorityQueue::new();

        priority_queue.dequeue();
    }
}
