// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Contact island: groups bodies connected by active contacts using union-find.

/// Union-find structure for island detection.
#[derive(Debug)]
#[allow(dead_code)]
pub struct IslandUnionFind {
    parent: Vec<usize>,
    rank: Vec<u32>,
    count: usize,
}

/// Create a union-find for `n` bodies.
#[allow(dead_code)]
pub fn new_island_uf(n: usize) -> IslandUnionFind {
    let parent = (0..n).collect();
    IslandUnionFind {
        parent,
        rank: vec![0; n],
        count: n,
    }
}

/// Find root with path compression.
#[allow(dead_code)]
pub fn uf_find(uf: &mut IslandUnionFind, i: usize) -> usize {
    if uf.parent[i] != i {
        uf.parent[i] = uf_find(uf, uf.parent[i]);
    }
    uf.parent[i]
}

/// Union two bodies into the same island; returns false if already same.
#[allow(dead_code)]
pub fn uf_union(uf: &mut IslandUnionFind, a: usize, b: usize) -> bool {
    let ra = uf_find(uf, a);
    let rb = uf_find(uf, b);
    if ra == rb {
        return false;
    }
    if uf.rank[ra] < uf.rank[rb] {
        uf.parent[ra] = rb;
    } else if uf.rank[ra] > uf.rank[rb] {
        uf.parent[rb] = ra;
    } else {
        uf.parent[rb] = ra;
        uf.rank[ra] += 1;
    }
    uf.count -= 1;
    true
}

/// Number of distinct islands.
#[allow(dead_code)]
pub fn island_count(uf: &mut IslandUnionFind) -> usize {
    uf.count
}

/// Assign island IDs (0-indexed) to each body.
#[allow(dead_code)]
pub fn island_labels(uf: &mut IslandUnionFind) -> Vec<usize> {
    let n = uf.parent.len();
    let roots: Vec<usize> = (0..n).map(|i| uf_find(uf, i)).collect();
    let mut id_map = std::collections::HashMap::new();
    let mut next = 0usize;
    roots
        .iter()
        .map(|&r| {
            *id_map.entry(r).or_insert_with(|| {
                let id = next;
                next += 1;
                id
            })
        })
        .collect()
}

/// Whether two bodies are in the same island.
#[allow(dead_code)]
pub fn same_island(uf: &mut IslandUnionFind, a: usize, b: usize) -> bool {
    uf_find(uf, a) == uf_find(uf, b)
}

/// Build islands from a contact list (pairs of body indices).
#[allow(dead_code)]
pub fn build_islands(n_bodies: usize, contacts: &[(usize, usize)]) -> IslandUnionFind {
    let mut uf = new_island_uf(n_bodies);
    for &(a, b) in contacts {
        uf_union(&mut uf, a, b);
    }
    uf
}

/// Size of the island containing body `i`.
#[allow(dead_code)]
pub fn island_size(uf: &mut IslandUnionFind, i: usize) -> usize {
    let root = uf_find(uf, i);
    let n = uf.parent.len();
    (0..n).filter(|&j| uf_find(uf, j) == root).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_body_own_island() {
        let mut uf = new_island_uf(3);
        assert!(!same_island(&mut uf, 0, 1));
        assert_eq!(island_count(&mut uf), 3);
    }

    #[test]
    fn test_union_reduces_count() {
        let mut uf = new_island_uf(4);
        uf_union(&mut uf, 0, 1);
        uf_union(&mut uf, 2, 3);
        assert_eq!(island_count(&mut uf), 2);
    }

    #[test]
    fn test_same_island_after_union() {
        let mut uf = new_island_uf(3);
        uf_union(&mut uf, 0, 2);
        assert!(same_island(&mut uf, 0, 2));
        assert!(!same_island(&mut uf, 0, 1));
    }

    #[test]
    fn test_transitive_island() {
        let mut uf = new_island_uf(4);
        uf_union(&mut uf, 0, 1);
        uf_union(&mut uf, 1, 2);
        assert!(same_island(&mut uf, 0, 2));
    }

    #[test]
    fn test_build_islands() {
        let contacts = vec![(0, 1), (1, 2)];
        let mut uf = build_islands(4, &contacts);
        assert!(same_island(&mut uf, 0, 2));
        assert!(!same_island(&mut uf, 0, 3));
    }

    #[test]
    fn test_labels_distinct() {
        let mut uf = new_island_uf(3);
        let labels = island_labels(&mut uf);
        assert_eq!(
            labels
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            3
        );
    }

    #[test]
    fn test_labels_same_group() {
        let mut uf = new_island_uf(3);
        uf_union(&mut uf, 0, 1);
        let labels = island_labels(&mut uf);
        assert_eq!(labels[0], labels[1]);
    }

    #[test]
    fn test_island_size() {
        let contacts = vec![(0, 1), (1, 2)];
        let mut uf = build_islands(4, &contacts);
        assert_eq!(island_size(&mut uf, 0), 3);
        assert_eq!(island_size(&mut uf, 3), 1);
    }

    #[test]
    fn test_double_union_no_change() {
        let mut uf = new_island_uf(2);
        assert!(uf_union(&mut uf, 0, 1));
        assert!(!uf_union(&mut uf, 0, 1));
        assert_eq!(island_count(&mut uf), 1);
    }
}
