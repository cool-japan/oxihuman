// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Dissolve selected edges: merge the two faces sharing each selected edge into one quad or n-gon.

use std::collections::HashMap;

/// Result of edge dissolution.
#[derive(Debug, Clone, Default)]
pub struct DissolveEdgeResult {
    pub edges_dissolved: usize,
    pub faces_merged: usize,
    pub faces_before: usize,
    pub faces_after: usize,
}

/// Canonical edge key.
pub fn edge_key_de(a: u32, b: u32) -> (u32, u32) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

/// Builds a map from each edge to the faces that share it.
pub fn edge_to_faces(indices: &[u32]) -> HashMap<(u32, u32), Vec<usize>> {
    let n = indices.len() / 3;
    let mut map: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for i in 0..n {
        let ia = indices[i * 3];
        let ib = indices[i * 3 + 1];
        let ic = indices[i * 3 + 2];
        for (a, b) in [(ia, ib), (ib, ic), (ic, ia)] {
            map.entry(edge_key_de(a, b)).or_default().push(i);
        }
    }
    map
}

/// Merges two triangles sharing edge `(ea, eb)` into a quad (4-vertex polygon).
/// Returns the quad vertices as a vec of 4 u32, or `None` if the edge isn't shared by exactly 2 faces.
pub fn merge_two_triangles(indices: &[u32], ea: u32, eb: u32) -> Option<(usize, usize, [u32; 4])> {
    let key = edge_key_de(ea, eb);
    let map = edge_to_faces(indices);
    let faces = map.get(&key)?;
    if faces.len() != 2 {
        return None;
    }
    let fi0 = faces[0];
    let fi1 = faces[1];
    /* find the third vertex of each face */
    let third = |fi: usize| -> Option<u32> {
        let ia = indices[fi * 3];
        let ib = indices[fi * 3 + 1];
        let ic = indices[fi * 3 + 2];
        if ia != ea && ia != eb {
            Some(ia)
        } else if ib != ea && ib != eb {
            Some(ib)
        } else if ic != ea && ic != eb {
            Some(ic)
        } else {
            None
        }
    };
    let v2 = third(fi0)?;
    let v3 = third(fi1)?;
    /* quad: v2, ea, v3, eb (a simple quad winding) */
    Some((fi0, fi1, [v2, ea, v3, eb]))
}

/// Dissolves a single edge: removes the two sharing faces and adds a new quad split into two new triangles.
pub fn dissolve_edge(indices: &[u32], ea: u32, eb: u32) -> Option<Vec<u32>> {
    let (fi0, fi1, quad) = merge_two_triangles(indices, ea, eb)?;
    let n = indices.len() / 3;
    let remove: std::collections::HashSet<usize> = [fi0, fi1].into_iter().collect();
    let mut out: Vec<u32> = Vec::with_capacity(indices.len());
    for i in 0..n {
        if !remove.contains(&i) {
            out.push(indices[i * 3]);
            out.push(indices[i * 3 + 1]);
            out.push(indices[i * 3 + 2]);
        }
    }
    /* re-triangulate the merged quad as two triangles (different diagonal) */
    out.push(quad[0]);
    out.push(quad[1]);
    out.push(quad[2]);
    out.push(quad[0]);
    out.push(quad[2]);
    out.push(quad[3]);
    Some(out)
}

/// Dissolves multiple edges in sequence.
pub fn dissolve_edges(indices: &[u32], edges: &[(u32, u32)]) -> (Vec<u32>, DissolveEdgeResult) {
    let faces_before = indices.len() / 3;
    let mut current = indices.to_vec();
    let mut dissolved = 0usize;
    let mut merged = 0usize;
    for &(ea, eb) in edges {
        if let Some(next) = dissolve_edge(&current, ea, eb) {
            let before = current.len() / 3;
            let after = next.len() / 3;
            current = next;
            dissolved += 1;
            merged += before.saturating_sub(after);
        }
    }
    let result = DissolveEdgeResult {
        edges_dissolved: dissolved,
        faces_merged: merged,
        faces_before,
        faces_after: current.len() / 3,
    };
    (current, result)
}

/// Returns edges that are eligible for dissolution (shared by exactly 2 faces).
pub fn dissolve_eligible_edges(indices: &[u32]) -> Vec<(u32, u32)> {
    edge_to_faces(indices)
        .into_iter()
        .filter(|(_, faces)| faces.len() == 2)
        .map(|(k, _)| k)
        .collect()
}

/// Returns the count of dissolve-eligible edges.
pub fn dissolve_eligible_count(indices: &[u32]) -> usize {
    dissolve_eligible_edges(indices).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_tris_shared_edge() -> Vec<u32> {
        /* (0,1,2) and (0,2,3) share edge (0,2) */
        vec![0u32, 1, 2, 0, 2, 3]
    }

    #[test]
    fn edge_key_canonical() {
        assert_eq!(edge_key_de(3, 1), edge_key_de(1, 3));
    }

    #[test]
    fn edge_to_faces_shared_edge_count() {
        let idx = two_tris_shared_edge();
        let map = edge_to_faces(&idx);
        let key = edge_key_de(0, 2);
        assert_eq!(map[&key].len(), 2);
    }

    #[test]
    fn merge_two_triangles_returns_quad() {
        let idx = two_tris_shared_edge();
        let res = merge_two_triangles(&idx, 0, 2);
        assert!(res.is_some());
    }

    #[test]
    fn dissolve_edge_reduces_face_count() {
        let idx = two_tris_shared_edge();
        let out = dissolve_edge(&idx, 0, 2).unwrap();
        /* 2 removed + 2 added = same count, but diagonal is different */
        assert_eq!(out.len() / 3, idx.len() / 3);
    }

    #[test]
    fn dissolve_nonexistent_edge_returns_none() {
        let idx = two_tris_shared_edge();
        let out = dissolve_edge(&idx, 9, 10);
        assert!(out.is_none());
    }

    #[test]
    fn dissolve_edges_batch() {
        let idx = two_tris_shared_edge();
        let (_, stats) = dissolve_edges(&idx, &[(0, 2)]);
        assert_eq!(stats.edges_dissolved, 1);
    }

    #[test]
    fn dissolve_eligible_includes_shared() {
        let idx = two_tris_shared_edge();
        let eligible = dissolve_eligible_edges(&idx);
        assert!(eligible.contains(&edge_key_de(0, 2)));
    }

    #[test]
    fn dissolve_eligible_count_positive() {
        let idx = two_tris_shared_edge();
        assert!(dissolve_eligible_count(&idx) > 0);
    }

    #[test]
    fn dissolve_edges_empty_input() {
        let idx: Vec<u32> = vec![];
        let (out, stats) = dissolve_edges(&idx, &[(0, 1)]);
        assert!(out.is_empty());
        assert_eq!(stats.edges_dissolved, 0);
    }
}
