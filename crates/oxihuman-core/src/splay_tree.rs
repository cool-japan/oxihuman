// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Splay tree — self-adjusting BST using array-based nodes.

/// A node in the splay tree.
#[derive(Debug, Clone, Default)]
struct SplayNode {
    key: i64,
    value: i64,
    left: Option<usize>,
    right: Option<usize>,
    parent: Option<usize>,
}

/// A splay tree (self-adjusting BST).
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct SplayTree {
    nodes: Vec<SplayNode>,
    root: Option<usize>,
    count: usize,
}

impl SplayTree {
    /// Create a new empty splay tree.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    fn alloc_node(&mut self, key: i64, value: i64) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(SplayNode {
            key,
            value,
            left: None,
            right: None,
            parent: None,
        });
        idx
    }

    fn rotate_right(&mut self, x: usize) {
        let y = match self.nodes[x].parent {
            Some(p) => p,
            None => return,
        };
        let z = self.nodes[y].parent;
        let xl = self.nodes[x].left;
        let xr = self.nodes[x].right;

        self.nodes[x].right = Some(y);
        self.nodes[y].parent = Some(x);
        self.nodes[y].left = xr;
        if let Some(xrn) = xr {
            self.nodes[xrn].parent = Some(y);
        }
        self.nodes[x].parent = z;
        if let Some(zn) = z {
            if self.nodes[zn].left == Some(y) {
                self.nodes[zn].left = Some(x);
            } else {
                self.nodes[zn].right = Some(x);
            }
        } else {
            self.root = Some(x);
        }
        let _ = xl;
    }

    fn rotate_left(&mut self, x: usize) {
        let y = match self.nodes[x].parent {
            Some(p) => p,
            None => return,
        };
        let z = self.nodes[y].parent;
        // Save x's left subtree before overwriting
        let xl = self.nodes[x].left;

        // x moves up, y becomes x's left child
        self.nodes[x].left = Some(y);
        self.nodes[y].parent = Some(x);
        // y inherits x's original left subtree as its right child
        self.nodes[y].right = xl;
        if let Some(xln) = xl {
            self.nodes[xln].parent = Some(y);
        }
        self.nodes[x].parent = z;
        if let Some(zn) = z {
            if self.nodes[zn].left == Some(y) {
                self.nodes[zn].left = Some(x);
            } else {
                self.nodes[zn].right = Some(x);
            }
        } else {
            self.root = Some(x);
        }
    }

    fn splay(&mut self, x: usize) {
        for _ in 0..self.nodes.len() + 1 {
            let parent = self.nodes[x].parent;
            match parent {
                None => break,
                Some(p) => {
                    let gp = self.nodes[p].parent;
                    if let Some(g) = gp {
                        if self.nodes[g].left == Some(p) && self.nodes[p].left == Some(x) {
                            self.rotate_right(p);
                            self.rotate_right(x);
                        } else if self.nodes[g].right == Some(p) && self.nodes[p].right == Some(x) {
                            self.rotate_left(p);
                            self.rotate_left(x);
                        } else if self.nodes[p].left == Some(x) {
                            self.rotate_right(x);
                            self.rotate_left(x);
                        } else {
                            self.rotate_left(x);
                            self.rotate_right(x);
                        }
                    } else {
                        // zig step (p is root)
                        if self.nodes[p].left == Some(x) {
                            self.rotate_right(x);
                        } else {
                            self.rotate_left(x);
                        }
                    }
                }
            }
        }
    }

    /// Insert a key-value pair.
    #[allow(dead_code)]
    pub fn insert(&mut self, key: i64, value: i64) {
        if let Some(idx) = self.find_node(key) {
            self.nodes[idx].value = value;
            return;
        }
        let idx = self.alloc_node(key, value);
        if self.root.is_none() {
            self.root = Some(idx);
            self.count += 1;
            return;
        }
        let Some(cur) = self.root else { return };
        let mut cur = cur;
        loop {
            if key < self.nodes[cur].key {
                match self.nodes[cur].left {
                    None => {
                        self.nodes[cur].left = Some(idx);
                        self.nodes[idx].parent = Some(cur);
                        break;
                    }
                    Some(l) => cur = l,
                }
            } else {
                match self.nodes[cur].right {
                    None => {
                        self.nodes[cur].right = Some(idx);
                        self.nodes[idx].parent = Some(cur);
                        break;
                    }
                    Some(r) => cur = r,
                }
            }
        }
        self.splay(idx);
        self.count += 1;
    }

    fn find_node(&self, key: i64) -> Option<usize> {
        let mut cur = self.root?;
        loop {
            if key == self.nodes[cur].key {
                return Some(cur);
            } else if key < self.nodes[cur].key {
                cur = self.nodes[cur].left?;
            } else {
                cur = self.nodes[cur].right?;
            }
        }
    }

    /// Look up a key.
    #[allow(dead_code)]
    pub fn get(&self, key: i64) -> Option<i64> {
        self.find_node(key).map(|i| self.nodes[i].value)
    }

    /// Check if a key exists.
    #[allow(dead_code)]
    pub fn contains(&self, key: i64) -> bool {
        self.find_node(key).is_some()
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut t = SplayTree::new();
        t.insert(5, 50);
        assert_eq!(t.get(5), Some(50));
    }

    #[test]
    fn get_missing_returns_none() {
        let t = SplayTree::new();
        assert!(t.get(1).is_none());
    }

    #[test]
    fn contains_after_insert() {
        let mut t = SplayTree::new();
        t.insert(3, 0);
        assert!(t.contains(3));
    }

    #[test]
    fn len_tracks_inserts() {
        let mut t = SplayTree::new();
        t.insert(1, 1);
        t.insert(2, 2);
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn is_empty_initially() {
        let t = SplayTree::new();
        assert!(t.is_empty());
    }

    #[test]
    fn overwrite_existing_key() {
        let mut t = SplayTree::new();
        t.insert(7, 70);
        t.insert(7, 77);
        assert_eq!(t.get(7), Some(77));
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn multiple_inserts() {
        let mut t = SplayTree::new();
        for i in 0..5i64 {
            t.insert(i, i * 10);
        }
        for i in 0..5i64 {
            assert_eq!(t.get(i), Some(i * 10));
        }
    }

    #[test]
    fn insert_descending_order() {
        let mut t = SplayTree::new();
        t.insert(5, 5);
        t.insert(3, 3);
        t.insert(1, 1);
        assert!(t.contains(3));
    }

    #[test]
    fn get_after_many_inserts() {
        let mut t = SplayTree::new();
        for i in [10i64, 5, 15, 2, 7, 12, 20].iter() {
            t.insert(*i, *i);
        }
        assert_eq!(t.get(15), Some(15));
    }

    #[test]
    fn is_empty_false_after_insert() {
        let mut t = SplayTree::new();
        t.insert(1, 1);
        assert!(!t.is_empty());
    }
}
