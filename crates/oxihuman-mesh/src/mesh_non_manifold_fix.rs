// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Non-manifold edge/vertex repair.
//! An edge is non-manifold when it is shared by more than two faces.
//! This module detects and optionally removes the extra faces.

use std::collections::HashMap;

/// Result of the non-manifold repair pass.
#[derive(Debug, Clone, Default)]
pub struct NonManifoldFixResult {
    pub non_manifold_edges_found: usize,
    pub faces_removed: usize,
    pub edges_total: usize,
}

/// Key for an undirected edge.
pub fn edge_key(a: u32, b: u32) -> (u32, u32) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

/// Counts how many faces reference each undirected edge.
pub fn edge_face_counts(indices: &[u32]) -> HashMap<(u32, u32), Vec<usize>> {
    let n = indices.len() / 3;
    let mut map: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for i in 0..n {
        let ia = indices[i * 3];
        let ib = indices[i * 3 + 1];
        let ic = indices[i * 3 + 2];
        for (a, b) in [(ia, ib), (ib, ic), (ic, ia)] {
            map.entry(edge_key(a, b)).or_default().push(i);
        }
    }
    map
}

/// Returns a list of non-manifold edge keys (edges shared by > 2 faces).
pub fn find_non_manifold_edges_nm(indices: &[u32]) -> Vec<(u32, u32)> {
    edge_face_counts(indices)
        .into_iter()
        .filter(|(_, faces)| faces.len() > 2)
        .map(|(k, _)| k)
        .collect()
}

/// Returns face indices that belong to a non-manifold edge.
pub fn non_manifold_face_indices(indices: &[u32]) -> Vec<usize> {
    let map = edge_face_counts(indices);
    let mut bad: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for faces in map.values() {
        if faces.len() > 2 {
            for &fi in faces {
                bad.insert(fi);
            }
        }
    }
    let mut v: Vec<usize> = bad.into_iter().collect();
    v.sort_unstable();
    v
}

/// Removes faces that cause non-manifold edges.
pub fn fix_non_manifold(indices: &[u32]) -> (Vec<u32>, NonManifoldFixResult) {
    let n = indices.len() / 3;
    let bad: std::collections::HashSet<usize> =
        non_manifold_face_indices(indices).into_iter().collect();
    let mut out = Vec::with_capacity(indices.len());
    for i in 0..n {
        if !bad.contains(&i) {
            out.push(indices[i * 3]);
            out.push(indices[i * 3 + 1]);
            out.push(indices[i * 3 + 2]);
        }
    }
    let nm_edges = find_non_manifold_edges_nm(indices).len();
    let result = NonManifoldFixResult {
        non_manifold_edges_found: nm_edges,
        faces_removed: bad.len(),
        edges_total: n * 3,
    };
    (out, result)
}

/// Returns `true` if the mesh is manifold (every edge shared by ≤ 2 faces).
pub fn is_manifold_mesh_nm(indices: &[u32]) -> bool {
    find_non_manifold_edges_nm(indices).is_empty()
}

/// Counts manifold vs. non-manifold edges.
pub fn manifold_edge_stats(indices: &[u32]) -> (usize, usize) {
    let map = edge_face_counts(indices);
    let nm = map.values().filter(|v| v.len() > 2).count();
    (map.len().saturating_sub(nm), nm)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_manifold_tris() -> Vec<u32> {
        /* two triangles sharing edge (1,2) */
        vec![0u32, 1, 2, 3, 2, 1]
    }

    fn nm_mesh() -> Vec<u32> {
        /* three triangles sharing edge (1,2) — non-manifold */
        vec![0u32, 1, 2, 3, 2, 1, 4, 1, 2]
    }

    #[test]
    fn manifold_mesh_has_no_nm_edges() {
        let idx = two_manifold_tris();
        assert!(is_manifold_mesh_nm(&idx));
    }

    #[test]
    fn non_manifold_mesh_detected() {
        let idx = nm_mesh();
        assert!(!is_manifold_mesh_nm(&idx));
    }

    #[test]
    fn find_nm_edges_counts_correct() {
        let idx = nm_mesh();
        let nme = find_non_manifold_edges_nm(&idx);
        assert_eq!(nme.len(), 1);
    }

    #[test]
    fn fix_removes_correct_faces() {
        let idx = nm_mesh();
        let (out, stats) = fix_non_manifold(&idx);
        assert!(stats.faces_removed > 0);
        /* result should be manifold */
        assert!(is_manifold_mesh_nm(&out));
    }

    #[test]
    fn edge_key_canonical() {
        assert_eq!(edge_key(3, 1), edge_key(1, 3));
    }

    #[test]
    fn manifold_stats_nm_zero() {
        let idx = two_manifold_tris();
        let (_, nm) = manifold_edge_stats(&idx);
        assert_eq!(nm, 0);
    }

    #[test]
    fn manifold_stats_nm_one() {
        let idx = nm_mesh();
        let (_, nm) = manifold_edge_stats(&idx);
        assert_eq!(nm, 1);
    }

    #[test]
    fn empty_mesh_is_manifold() {
        let idx: Vec<u32> = vec![];
        assert!(is_manifold_mesh_nm(&idx));
    }

    #[test]
    fn face_indices_sorted() {
        let idx = nm_mesh();
        let faces = non_manifold_face_indices(&idx);
        for w in faces.windows(2) {
            assert!(w[0] < w[1]);
        }
    }
}
