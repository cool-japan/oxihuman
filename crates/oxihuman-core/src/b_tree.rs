// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! B-tree with configurable order for range scans.

/// A B-tree node.
#[derive(Debug, Clone)]
struct BNode<K: Ord + Clone, V: Clone> {
    keys: Vec<K>,
    values: Vec<V>,
    children: Vec<BNode<K, V>>,
    is_leaf: bool,
}

impl<K: Ord + Clone, V: Clone> BNode<K, V> {
    fn new_leaf() -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
            is_leaf: true,
        }
    }

    fn new_internal() -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
            is_leaf: false,
        }
    }
}

/// A B-tree.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BTree<K: Ord + Clone, V: Clone> {
    root: BNode<K, V>,
    order: usize,
    count: usize,
}

impl<K: Ord + Clone, V: Clone> BTree<K, V> {
    /// Create a new B-tree with the given `order` (minimum degree).
    /// `order` must be >= 2.
    #[allow(dead_code)]
    pub fn new(order: usize) -> Self {
        let order = order.max(2);
        Self {
            root: BNode::new_leaf(),
            order,
            count: 0,
        }
    }

    fn search<'a>(node: &'a BNode<K, V>, key: &K) -> Option<&'a V> {
        let pos = node.keys.partition_point(|k| k < key);
        if pos < node.keys.len() && &node.keys[pos] == key {
            return Some(&node.values[pos]);
        }
        if node.is_leaf {
            return None;
        }
        if pos < node.children.len() {
            Self::search(&node.children[pos], key)
        } else {
            None
        }
    }

    /// Look up a key.
    #[allow(dead_code)]
    pub fn get(&self, key: &K) -> Option<&V> {
        Self::search(&self.root, key)
    }

    /// Check if a key exists.
    #[allow(dead_code)]
    pub fn contains(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    fn insert_non_full(node: &mut BNode<K, V>, key: K, value: V, order: usize) -> bool {
        let pos = node.keys.partition_point(|k| k < &key);
        if pos < node.keys.len() && node.keys[pos] == key {
            node.values[pos] = value;
            return false; // overwrite, no new key
        }
        if node.is_leaf {
            node.keys.insert(pos, key);
            node.values.insert(pos, value);
            true
        } else {
            if pos >= node.children.len() {
                return false;
            }
            let needs_split = node.children[pos].keys.len() == 2 * order - 1;
            if needs_split {
                Self::split_child(node, pos, order);
                if node.keys[pos] == key {
                    node.values[pos] = value;
                    return false;
                }
                let new_pos = if key > node.keys[pos] { pos + 1 } else { pos };
                if new_pos < node.children.len() {
                    Self::insert_non_full(&mut node.children[new_pos], key, value, order)
                } else {
                    false
                }
            } else {
                Self::insert_non_full(&mut node.children[pos], key, value, order)
            }
        }
    }

    fn split_child(parent: &mut BNode<K, V>, i: usize, order: usize) {
        if i >= parent.children.len() {
            return;
        }
        let mid = order - 1;
        if parent.children[i].keys.len() < mid + 1 {
            return;
        }
        let mut new_node = if parent.children[i].is_leaf {
            BNode::new_leaf()
        } else {
            BNode::new_internal()
        };

        let child = &mut parent.children[i];
        new_node.keys = child.keys.split_off(mid + 1);
        new_node.values = child.values.split_off(mid + 1);
        if !child.is_leaf {
            new_node.children = child.children.split_off(mid + 1);
        }
        let Some(up_key) = child.keys.pop() else {
            return;
        };
        let Some(up_val) = child.values.pop() else {
            return;
        };

        parent.keys.insert(i, up_key);
        parent.values.insert(i, up_val);
        parent.children.insert(i + 1, new_node);
    }

    /// Insert a key-value pair.
    #[allow(dead_code)]
    pub fn insert(&mut self, key: K, value: V) {
        if self.root.keys.len() == 2 * self.order - 1 {
            let mut new_root = BNode::new_internal();
            let old_root = std::mem::replace(&mut self.root, BNode::new_internal());
            new_root.children.push(old_root);
            Self::split_child(&mut new_root, 0, self.order);
            self.root = new_root;
        }
        let is_new = Self::insert_non_full(&mut self.root, key, value, self.order);
        if is_new {
            self.count += 1;
        }
    }

    /// Number of entries.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Collect all (key, value) pairs in sorted order.
    #[allow(dead_code)]
    pub fn range_scan(&self) -> Vec<(K, V)> {
        let mut out = Vec::new();
        Self::collect(&self.root, &mut out);
        out
    }

    fn collect(node: &BNode<K, V>, out: &mut Vec<(K, V)>) {
        if node.is_leaf {
            for (k, v) in node.keys.iter().zip(node.values.iter()) {
                out.push((k.clone(), v.clone()));
            }
            return;
        }
        for (i, (k, v)) in node.keys.iter().zip(node.values.iter()).enumerate() {
            if i < node.children.len() {
                Self::collect(&node.children[i], out);
            }
            out.push((k.clone(), v.clone()));
        }
        if let Some(last) = node.children.last() {
            Self::collect(last, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filled_tree() -> BTree<i32, i32> {
        let mut t = BTree::new(2);
        for i in [5, 3, 8, 1, 4, 7, 9, 2, 6].iter() {
            t.insert(*i, *i * 10);
        }
        t
    }

    #[test]
    fn insert_and_get() {
        let mut t: BTree<i32, i32> = BTree::new(2);
        t.insert(5, 50);
        assert_eq!(t.get(&5), Some(&50));
    }

    #[test]
    fn get_missing_is_none() {
        let t: BTree<i32, i32> = BTree::new(2);
        assert!(t.get(&1).is_none());
    }

    #[test]
    fn contains_after_insert() {
        let t = filled_tree();
        assert!(t.contains(&7));
    }

    #[test]
    fn len_correct() {
        let t = filled_tree();
        assert_eq!(t.len(), 9);
    }

    #[test]
    fn is_empty_initially() {
        let t: BTree<i32, i32> = BTree::new(2);
        assert!(t.is_empty());
    }

    #[test]
    fn range_scan_sorted() {
        let t = filled_tree();
        let pairs = t.range_scan();
        for w in pairs.windows(2) {
            assert!(w[0].0 <= w[1].0);
        }
    }

    #[test]
    fn overwrite_existing_key() {
        let mut t: BTree<i32, i32> = BTree::new(2);
        t.insert(1, 10);
        t.insert(1, 20);
        assert_eq!(t.get(&1), Some(&20));
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn many_inserts_searchable() {
        let mut t: BTree<i32, i32> = BTree::new(3);
        for i in 0..20i32 {
            t.insert(i, i);
        }
        assert_eq!(t.get(&15), Some(&15));
    }

    #[test]
    fn order_3_tree() {
        let mut t: BTree<i32, i32> = BTree::new(3);
        for i in 0..10i32 {
            t.insert(i, i);
        }
        assert_eq!(t.len(), 10);
    }

    #[test]
    fn is_empty_false_after_insert() {
        let mut t: BTree<i32, i32> = BTree::new(2);
        t.insert(1, 1);
        assert!(!t.is_empty());
    }
}
