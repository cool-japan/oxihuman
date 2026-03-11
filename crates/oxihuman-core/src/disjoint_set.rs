// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Union-Find (disjoint set) with path compression and union by rank.

/// A disjoint set forest with path compression and union by rank.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DisjointSet {
    parent: Vec<usize>,
    rank: Vec<u32>,
    count: usize, // number of distinct sets
}

#[allow(dead_code)]
impl DisjointSet {
    pub fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
            count: n,
        }
    }

    pub fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]]; // path halving
            x = self.parent[x];
        }
        x
    }

    pub fn union(&mut self, a: usize, b: usize) -> bool {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return false;
        }
        match self.rank[ra].cmp(&self.rank[rb]) {
            std::cmp::Ordering::Less => self.parent[ra] = rb,
            std::cmp::Ordering::Greater => self.parent[rb] = ra,
            std::cmp::Ordering::Equal => {
                self.parent[rb] = ra;
                self.rank[ra] += 1;
            }
        }
        self.count -= 1;
        true
    }

    pub fn connected(&mut self, a: usize, b: usize) -> bool {
        self.find(a) == self.find(b)
    }

    pub fn set_count(&self) -> usize {
        self.count
    }

    pub fn element_count(&self) -> usize {
        self.parent.len()
    }

    /// Size of the set containing x.
    pub fn set_size(&mut self, x: usize) -> usize {
        let root = self.find(x);
        let n = self.parent.len();
        let mut size = 0;
        for i in 0..n {
            if self.find(i) == root {
                size += 1;
            }
        }
        size
    }

    /// Returns all roots (representatives).
    pub fn roots(&mut self) -> Vec<usize> {
        let n = self.parent.len();
        let mut roots = Vec::new();
        for i in 0..n {
            if self.find(i) == i {
                roots.push(i);
            }
        }
        roots
    }
}

#[allow(dead_code)]
pub fn new_disjoint_set(n: usize) -> DisjointSet {
    DisjointSet::new(n)
}

#[allow(dead_code)]
pub fn ds_find(ds: &mut DisjointSet, x: usize) -> usize {
    ds.find(x)
}

#[allow(dead_code)]
pub fn ds_union(ds: &mut DisjointSet, x: usize, y: usize) -> bool {
    ds.union(x, y)
}

#[allow(dead_code)]
pub fn ds_connected(ds: &mut DisjointSet, x: usize, y: usize) -> bool {
    ds.connected(x, y)
}

#[allow(dead_code)]
pub fn ds_same(ds: &mut DisjointSet, x: usize, y: usize) -> bool {
    ds.connected(x, y)
}

#[allow(dead_code)]
pub fn ds_component_count(ds: &DisjointSet) -> usize {
    ds.set_count()
}

#[allow(dead_code)]
pub fn ds_size(ds: &DisjointSet) -> usize {
    ds.element_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_separate() {
        let mut ds = DisjointSet::new(5);
        assert!(!ds.connected(0, 1));
        assert_eq!(ds.set_count(), 5);
    }

    #[test]
    fn test_union() {
        let mut ds = DisjointSet::new(5);
        assert!(ds.union(0, 1));
        assert!(ds.connected(0, 1));
        assert_eq!(ds.set_count(), 4);
    }

    #[test]
    fn test_transitive() {
        let mut ds = DisjointSet::new(5);
        ds.union(0, 1);
        ds.union(1, 2);
        assert!(ds.connected(0, 2));
    }

    #[test]
    fn test_no_dup_union() {
        let mut ds = DisjointSet::new(3);
        assert!(ds.union(0, 1));
        assert!(!ds.union(0, 1)); // already same set
    }

    #[test]
    fn test_set_size() {
        let mut ds = DisjointSet::new(5);
        ds.union(0, 1);
        ds.union(0, 2);
        assert_eq!(ds.set_size(0), 3);
        assert_eq!(ds.set_size(3), 1);
    }

    #[test]
    fn test_roots() {
        let mut ds = DisjointSet::new(4);
        ds.union(0, 1);
        ds.union(2, 3);
        let roots = ds.roots();
        assert_eq!(roots.len(), 2);
    }

    #[test]
    fn test_all_union() {
        let mut ds = DisjointSet::new(4);
        ds.union(0, 1);
        ds.union(2, 3);
        ds.union(0, 2);
        assert_eq!(ds.set_count(), 1);
    }

    #[test]
    fn test_element_count() {
        let ds = DisjointSet::new(10);
        assert_eq!(ds.element_count(), 10);
    }

    #[test]
    fn test_find_self() {
        let mut ds = DisjointSet::new(3);
        assert_eq!(ds.find(2), 2);
    }

    #[test]
    fn test_large_chain() {
        let mut ds = DisjointSet::new(100);
        for i in 0..99 {
            ds.union(i, i + 1);
        }
        assert_eq!(ds.set_count(), 1);
        assert!(ds.connected(0, 99));
    }
}
