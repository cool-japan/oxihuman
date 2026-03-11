// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Edge hashing utilities for fast edge lookups.

use std::collections::{HashMap, HashSet};

/// Canonical edge key (smaller index first).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeKey {
    pub v0: u32,
    pub v1: u32,
}

/// Create a canonical edge key.
#[allow(dead_code)]
pub fn make_edge_key(a: u32, b: u32) -> EdgeKey {
    if a <= b {
        EdgeKey { v0: a, v1: b }
    } else {
        EdgeKey { v0: b, v1: a }
    }
}

/// A hash map of edges to their face indices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeHashMap {
    pub map: HashMap<EdgeKey, Vec<usize>>,
}

/// Build an edge hash map from triangle indices.
#[allow(dead_code)]
pub fn build_edge_hash_map(indices: &[u32]) -> EdgeHashMap {
    let mut map: HashMap<EdgeKey, Vec<usize>> = HashMap::new();
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let vs = [indices[t * 3], indices[t * 3 + 1], indices[t * 3 + 2]];
        for k in 0..3 {
            let key = make_edge_key(vs[k], vs[(k + 1) % 3]);
            map.entry(key).or_default().push(t);
        }
    }
    EdgeHashMap { map }
}

/// Number of unique edges.
#[allow(dead_code)]
pub fn edge_count(ehm: &EdgeHashMap) -> usize {
    ehm.map.len()
}

/// Get faces adjacent to an edge.
#[allow(dead_code)]
pub fn faces_for_edge(ehm: &EdgeHashMap, a: u32, b: u32) -> Option<&Vec<usize>> {
    ehm.map.get(&make_edge_key(a, b))
}

/// Check if an edge exists.
#[allow(dead_code)]
pub fn edge_exists(ehm: &EdgeHashMap, a: u32, b: u32) -> bool {
    ehm.map.contains_key(&make_edge_key(a, b))
}

/// Find boundary edges (only one adjacent face).
#[allow(dead_code)]
pub fn boundary_edges(ehm: &EdgeHashMap) -> Vec<EdgeKey> {
    ehm.map
        .iter()
        .filter(|(_, v)| v.len() == 1)
        .map(|(k, _)| *k)
        .collect()
}

/// Find non-manifold edges (more than two adjacent faces).
#[allow(dead_code)]
pub fn non_manifold_edges(ehm: &EdgeHashMap) -> Vec<EdgeKey> {
    ehm.map
        .iter()
        .filter(|(_, v)| v.len() > 2)
        .map(|(k, _)| *k)
        .collect()
}

/// Collect all unique vertex indices referenced by edges.
#[allow(dead_code)]
pub fn unique_vertices(ehm: &EdgeHashMap) -> HashSet<u32> {
    let mut set = HashSet::new();
    for key in ehm.map.keys() {
        set.insert(key.v0);
        set.insert(key.v1);
    }
    set
}

/// Convert to JSON summary.
#[allow(dead_code)]
pub fn edge_hash_to_json(ehm: &EdgeHashMap) -> String {
    let boundary = boundary_edges(ehm).len();
    format!(
        "{{\"edge_count\":{},\"boundary_edges\":{}}}",
        ehm.map.len(),
        boundary,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_tri() -> Vec<u32> {
        vec![0, 1, 2]
    }

    fn two_tris() -> Vec<u32> {
        vec![0, 1, 2, 1, 3, 2]
    }

    #[test]
    fn test_make_edge_key_canonical() {
        assert_eq!(make_edge_key(5, 3), make_edge_key(3, 5));
    }

    #[test]
    fn test_build_single_tri() {
        let ehm = build_edge_hash_map(&single_tri());
        assert_eq!(edge_count(&ehm), 3);
    }

    #[test]
    fn test_two_tris_edge_count() {
        let ehm = build_edge_hash_map(&two_tris());
        assert_eq!(edge_count(&ehm), 5);
    }

    #[test]
    fn test_faces_for_edge() {
        let ehm = build_edge_hash_map(&two_tris());
        let faces = faces_for_edge(&ehm, 1, 2).unwrap();
        assert_eq!(faces.len(), 2);
    }

    #[test]
    fn test_edge_exists() {
        let ehm = build_edge_hash_map(&single_tri());
        assert!(edge_exists(&ehm, 0, 1));
        assert!(!edge_exists(&ehm, 0, 5));
    }

    #[test]
    fn test_boundary_edges_single_tri() {
        let ehm = build_edge_hash_map(&single_tri());
        assert_eq!(boundary_edges(&ehm).len(), 3);
    }

    #[test]
    fn test_boundary_edges_two_tris() {
        let ehm = build_edge_hash_map(&two_tris());
        assert_eq!(boundary_edges(&ehm).len(), 4);
    }

    #[test]
    fn test_non_manifold_empty() {
        let ehm = build_edge_hash_map(&two_tris());
        assert!(non_manifold_edges(&ehm).is_empty());
    }

    #[test]
    fn test_unique_vertices() {
        let ehm = build_edge_hash_map(&two_tris());
        assert_eq!(unique_vertices(&ehm).len(), 4);
    }

    #[test]
    fn test_edge_hash_to_json() {
        let ehm = build_edge_hash_map(&single_tri());
        let json = edge_hash_to_json(&ehm);
        assert!(json.contains("\"edge_count\":3"));
    }
}
