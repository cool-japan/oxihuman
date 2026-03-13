// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Union-Find (disjoint set) with path compression and union-by-rank.

/// A union-find data structure over integer elements `0..n`.
#[derive(Debug, Clone)]
pub struct UnionFindV3 {
    parent: Vec<usize>,
    rank: Vec<u32>,
    size: Vec<usize>,
    num_components: usize,
}

impl UnionFindV3 {
    /// Create a new union-find with `n` singletons.
    pub fn new(n: usize) -> Self {
        UnionFindV3 {
            parent: (0..n).collect(),
            rank: vec![0; n],
            size: vec![1; n],
            num_components: n,
        }
    }

    /// Find the root of `x` with path compression.
    pub fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    /// Union the sets containing `a` and `b`.
    /// Returns true if they were in different sets.
    pub fn union(&mut self, a: usize, b: usize) -> bool {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return false;
        }
        if self.rank[ra] < self.rank[rb] {
            self.parent[ra] = rb;
            self.size[rb] += self.size[ra];
        } else if self.rank[ra] > self.rank[rb] {
            self.parent[rb] = ra;
            self.size[ra] += self.size[rb];
        } else {
            self.parent[rb] = ra;
            self.size[ra] += self.size[rb];
            self.rank[ra] += 1;
        }
        self.num_components -= 1;
        true
    }

    /// True if `a` and `b` are in the same set.
    pub fn connected(&mut self, a: usize, b: usize) -> bool {
        self.find(a) == self.find(b)
    }

    /// Size of the component containing `x`.
    pub fn component_size(&mut self, x: usize) -> usize {
        let root = self.find(x);
        self.size[root]
    }

    /// Number of disjoint components.
    pub fn num_components(&self) -> usize {
        self.num_components
    }

    /// Total number of elements.
    pub fn len(&self) -> usize {
        self.parent.len()
    }

    /// True if no elements.
    pub fn is_empty(&self) -> bool {
        self.parent.is_empty()
    }
}

/// Create a new union-find with `n` elements.
pub fn new_union_find_v3(n: usize) -> UnionFindV3 {
    UnionFindV3::new(n)
}

/// Union two elements.
pub fn uf3_union(uf: &mut UnionFindV3, a: usize, b: usize) -> bool {
    uf.union(a, b)
}

/// Find root.
pub fn uf3_find(uf: &mut UnionFindV3, x: usize) -> usize {
    uf.find(x)
}

/// Check connectivity.
pub fn uf3_connected(uf: &mut UnionFindV3, a: usize, b: usize) -> bool {
    uf.connected(a, b)
}

/// Number of components.
pub fn uf3_num_components(uf: &UnionFindV3) -> usize {
    uf.num_components()
}

/// Component size.
pub fn uf3_component_size(uf: &mut UnionFindV3, x: usize) -> usize {
    uf.component_size(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_components() {
        let uf = new_union_find_v3(5);
        assert_eq!(uf3_num_components(&uf), 5 /* five singletons */);
    }

    #[test]
    fn test_union_reduces_components() {
        let mut uf = new_union_find_v3(4);
        uf3_union(&mut uf, 0, 1);
        assert_eq!(uf3_num_components(&uf), 3 /* one merge */);
    }

    #[test]
    fn test_connected() {
        let mut uf = new_union_find_v3(5);
        uf3_union(&mut uf, 2, 3);
        assert!(uf3_connected(&mut uf, 2, 3) /* merged */);
        assert!(!uf3_connected(&mut uf, 0, 3));
    }

    #[test]
    fn test_transitive() {
        let mut uf = new_union_find_v3(5);
        uf3_union(&mut uf, 0, 1);
        uf3_union(&mut uf, 1, 2);
        assert!(uf3_connected(&mut uf, 0, 2) /* transitive */);
    }

    #[test]
    fn test_union_same_set_returns_false() {
        let mut uf = new_union_find_v3(3);
        uf3_union(&mut uf, 0, 1);
        assert!(!uf3_union(&mut uf, 0, 1) /* already connected */);
    }

    #[test]
    fn test_component_size() {
        let mut uf = new_union_find_v3(5);
        uf3_union(&mut uf, 0, 1);
        uf3_union(&mut uf, 1, 2);
        assert_eq!(uf3_component_size(&mut uf, 0), 3 /* three elements */);
    }

    #[test]
    fn test_find_consistency() {
        let mut uf = new_union_find_v3(3);
        uf3_union(&mut uf, 0, 1);
        assert_eq!(uf3_find(&mut uf, 0), uf3_find(&mut uf, 1) /* same root */);
    }

    #[test]
    fn test_len() {
        let uf = new_union_find_v3(10);
        assert_eq!(uf.len(), 10 /* ten elements */);
    }

    #[test]
    fn test_all_merged() {
        let mut uf = new_union_find_v3(4);
        uf3_union(&mut uf, 0, 1);
        uf3_union(&mut uf, 2, 3);
        uf3_union(&mut uf, 0, 2);
        assert_eq!(uf3_num_components(&uf), 1 /* all one component */);
    }

    #[test]
    fn test_empty() {
        let uf = new_union_find_v3(0);
        assert!(uf.is_empty() /* zero elements */);
    }
}
