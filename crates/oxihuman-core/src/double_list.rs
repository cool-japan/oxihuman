// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A simple doubly-linked list implemented with Vec-based nodes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Node<T> {
    value: T,
    prev: Option<usize>,
    next: Option<usize>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DoubleList<T> {
    nodes: Vec<Option<Node<T>>>,
    head: Option<usize>,
    tail: Option<usize>,
    len: usize,
    free: Vec<usize>,
}

#[allow(dead_code)]
impl<T: Clone> DoubleList<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            head: None,
            tail: None,
            len: 0,
            free: Vec::new(),
        }
    }

    fn alloc_node(&mut self, value: T) -> usize {
        let node = Node {
            value,
            prev: None,
            next: None,
        };
        if let Some(idx) = self.free.pop() {
            self.nodes[idx] = Some(node);
            idx
        } else {
            let idx = self.nodes.len();
            self.nodes.push(Some(node));
            idx
        }
    }

    pub fn push_back(&mut self, value: T) -> usize {
        let idx = self.alloc_node(value);
        if let Some(tail) = self.tail {
            if let Some(n) = self.nodes[tail].as_mut() {
                n.next = Some(idx);
            }
            if let Some(n) = self.nodes[idx].as_mut() {
                n.prev = Some(tail);
            }
        } else {
            self.head = Some(idx);
        }
        self.tail = Some(idx);
        self.len += 1;
        idx
    }

    pub fn push_front(&mut self, value: T) -> usize {
        let idx = self.alloc_node(value);
        if let Some(head) = self.head {
            if let Some(n) = self.nodes[head].as_mut() {
                n.prev = Some(idx);
            }
            if let Some(n) = self.nodes[idx].as_mut() {
                n.next = Some(head);
            }
        } else {
            self.tail = Some(idx);
        }
        self.head = Some(idx);
        self.len += 1;
        idx
    }

    pub fn pop_front(&mut self) -> Option<T> {
        let head = self.head?;
        let node = self.nodes[head].take()?;
        self.head = node.next;
        if let Some(new_head) = self.head {
            if let Some(n) = self.nodes[new_head].as_mut() {
                n.prev = None;
            }
        } else {
            self.tail = None;
        }
        self.free.push(head);
        self.len -= 1;
        Some(node.value)
    }

    pub fn pop_back(&mut self) -> Option<T> {
        let tail = self.tail?;
        let node = self.nodes[tail].take()?;
        self.tail = node.prev;
        if let Some(new_tail) = self.tail {
            if let Some(n) = self.nodes[new_tail].as_mut() {
                n.next = None;
            }
        } else {
            self.head = None;
        }
        self.free.push(tail);
        self.len -= 1;
        Some(node.value)
    }

    pub fn front(&self) -> Option<&T> {
        self.head
            .and_then(|h| self.nodes[h].as_ref().map(|n| &n.value))
    }

    pub fn back(&self) -> Option<&T> {
        self.tail
            .and_then(|t| self.nodes[t].as_ref().map(|n| &n.value))
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        self.nodes
            .get(idx)
            .and_then(|n| n.as_ref().map(|n| &n.value))
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn to_vec(&self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.len);
        let mut cur = self.head;
        while let Some(idx) = cur {
            if let Some(node) = &self.nodes[idx] {
                result.push(node.value.clone());
                cur = node.next;
            } else {
                break;
            }
        }
        result
    }
}

impl<T: Clone> Default for DoubleList<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let list: DoubleList<i32> = DoubleList::new();
        assert!(list.is_empty());
    }

    #[test]
    fn test_push_back() {
        let mut list = DoubleList::new();
        list.push_back(1);
        list.push_back(2);
        assert_eq!(list.len(), 2);
        assert_eq!(list.front(), Some(&1));
        assert_eq!(list.back(), Some(&2));
    }

    #[test]
    fn test_push_front() {
        let mut list = DoubleList::new();
        list.push_front(1);
        list.push_front(2);
        assert_eq!(list.front(), Some(&2));
        assert_eq!(list.back(), Some(&1));
    }

    #[test]
    fn test_pop_front() {
        let mut list = DoubleList::new();
        list.push_back(10);
        list.push_back(20);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_pop_back() {
        let mut list = DoubleList::new();
        list.push_back(10);
        list.push_back(20);
        assert_eq!(list.pop_back(), Some(20));
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_pop_empty() {
        let mut list: DoubleList<i32> = DoubleList::new();
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.pop_back(), None);
    }

    #[test]
    fn test_to_vec() {
        let mut list = DoubleList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        assert_eq!(list.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_get() {
        let mut list = DoubleList::new();
        let idx = list.push_back(42);
        assert_eq!(list.get(idx), Some(&42));
    }

    #[test]
    fn test_mixed_ops() {
        let mut list = DoubleList::new();
        list.push_back(1);
        list.push_front(0);
        list.push_back(2);
        assert_eq!(list.to_vec(), vec![0, 1, 2]);
    }

    #[test]
    fn test_single_element() {
        let mut list = DoubleList::new();
        list.push_back(99);
        assert_eq!(list.pop_front(), Some(99));
        assert!(list.is_empty());
        assert_eq!(list.front(), None);
        assert_eq!(list.back(), None);
    }
}
