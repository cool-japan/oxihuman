// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Union-Find (disjoint set) with path compression and union by rank.

/// Union-Find structure.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UnionFindV2 {
    parent: Vec<usize>,
    rank: Vec<u32>,
    size: Vec<usize>,
    component_count: usize,
}

/// Create a new `UnionFindV2` with `n` elements (0..n).
#[allow(dead_code)]
pub fn new_union_find(n: usize) -> UnionFindV2 {
    UnionFindV2 {
        parent: (0..n).collect(),
        rank: vec![0; n],
        size: vec![1; n],
        component_count: n,
    }
}

/// Find root of element `x` with path compression.
#[allow(dead_code)]
pub fn uf_find(uf: &mut UnionFindV2, x: usize) -> usize {
    if uf.parent[x] != x {
        uf.parent[x] = uf_find(uf, uf.parent[x]);
    }
    uf.parent[x]
}

/// Union two elements. Returns true if they were in different components.
#[allow(dead_code)]
pub fn uf_union(uf: &mut UnionFindV2, x: usize, y: usize) -> bool {
    let rx = uf_find(uf, x);
    let ry = uf_find(uf, y);
    if rx == ry {
        return false;
    }
    match uf.rank[rx].cmp(&uf.rank[ry]) {
        std::cmp::Ordering::Less => {
            uf.parent[rx] = ry;
            uf.size[ry] += uf.size[rx];
        }
        std::cmp::Ordering::Greater => {
            uf.parent[ry] = rx;
            uf.size[rx] += uf.size[ry];
        }
        std::cmp::Ordering::Equal => {
            uf.parent[ry] = rx;
            uf.size[rx] += uf.size[ry];
            uf.rank[rx] += 1;
        }
    }
    uf.component_count -= 1;
    true
}

/// Whether two elements are in the same component.
#[allow(dead_code)]
pub fn uf_connected(uf: &mut UnionFindV2, x: usize, y: usize) -> bool {
    uf_find(uf, x) == uf_find(uf, y)
}

/// Size of the component containing element `x`.
#[allow(dead_code)]
pub fn uf_component_size(uf: &mut UnionFindV2, x: usize) -> usize {
    let root = uf_find(uf, x);
    uf.size[root]
}

/// Number of distinct components.
#[allow(dead_code)]
pub fn uf_component_count(uf: &UnionFindV2) -> usize {
    uf.component_count
}

/// Total number of elements.
#[allow(dead_code)]
pub fn uf_element_count(uf: &UnionFindV2) -> usize {
    uf.parent.len()
}

/// Reset to initial state (all singletons).
#[allow(dead_code)]
pub fn uf_reset(uf: &mut UnionFindV2) {
    let n = uf.parent.len();
    for i in 0..n {
        uf.parent[i] = i;
        uf.rank[i] = 0;
        uf.size[i] = 1;
    }
    uf.component_count = n;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_components() {
        let uf = new_union_find(5);
        assert_eq!(uf_component_count(&uf), 5);
    }

    #[test]
    fn test_union_reduces_components() {
        let mut uf = new_union_find(4);
        uf_union(&mut uf, 0, 1);
        assert_eq!(uf_component_count(&uf), 3);
    }

    #[test]
    fn test_connected_after_union() {
        let mut uf = new_union_find(4);
        uf_union(&mut uf, 1, 2);
        assert!(uf_connected(&mut uf, 1, 2));
    }

    #[test]
    fn test_not_connected_initially() {
        let mut uf = new_union_find(4);
        assert!(!uf_connected(&mut uf, 0, 3));
    }

    #[test]
    fn test_component_size() {
        let mut uf = new_union_find(4);
        uf_union(&mut uf, 0, 1);
        uf_union(&mut uf, 1, 2);
        assert_eq!(uf_component_size(&mut uf, 0), 3);
    }

    #[test]
    fn test_transitivity() {
        let mut uf = new_union_find(5);
        uf_union(&mut uf, 0, 1);
        uf_union(&mut uf, 1, 2);
        assert!(uf_connected(&mut uf, 0, 2));
    }

    #[test]
    fn test_union_same_returns_false() {
        let mut uf = new_union_find(3);
        uf_union(&mut uf, 0, 1);
        assert!(!uf_union(&mut uf, 0, 1));
    }

    #[test]
    fn test_reset() {
        let mut uf = new_union_find(3);
        uf_union(&mut uf, 0, 1);
        uf_reset(&mut uf);
        assert_eq!(uf_component_count(&uf), 3);
    }

    #[test]
    fn test_element_count() {
        let uf = new_union_find(7);
        assert_eq!(uf_element_count(&uf), 7);
    }
}
