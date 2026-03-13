// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Sequential face peeling — remove outermost boundary layer of faces iteratively.

#![allow(dead_code)]

use std::collections::HashSet;

/// A face peel result containing the remaining faces after one peel step.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FacePeelResult {
    /// Face indices that were retained.
    pub retained: Vec<usize>,
    /// Face indices that were peeled (removed).
    pub peeled: Vec<usize>,
}

/// Build an adjacency list: for each face, which other faces share an edge.
#[allow(dead_code)]
pub fn build_face_adjacency(indices: &[u32], face_count: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); face_count];
    // Map edge -> list of faces containing it
    use std::collections::HashMap;
    let mut edge_map: HashMap<(u32, u32), Vec<usize>> = HashMap::new();

    for f in 0..face_count {
        let a = indices[f * 3];
        let b = indices[f * 3 + 1];
        let c = indices[f * 3 + 2];
        for &(p, q) in &[(a, b), (b, c), (c, a)] {
            let key = if p < q { (p, q) } else { (q, p) };
            edge_map.entry(key).or_default().push(f);
        }
    }

    for faces in edge_map.values() {
        if faces.len() == 2 {
            let (f0, f1) = (faces[0], faces[1]);
            adj[f0].push(f1);
            adj[f1].push(f0);
        }
    }
    adj
}

/// Identify boundary faces: faces that have at least one edge shared with fewer than 2 faces.
#[allow(dead_code)]
pub fn boundary_faces(indices: &[u32], face_count: usize) -> Vec<usize> {
    use std::collections::HashMap;
    let mut edge_count: HashMap<(u32, u32), usize> = HashMap::new();
    for f in 0..face_count {
        let a = indices[f * 3];
        let b = indices[f * 3 + 1];
        let c = indices[f * 3 + 2];
        for &(p, q) in &[(a, b), (b, c), (c, a)] {
            let key = if p < q { (p, q) } else { (q, p) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }

    let mut boundary = Vec::new();
    for f in 0..face_count {
        let a = indices[f * 3];
        let b = indices[f * 3 + 1];
        let c = indices[f * 3 + 2];
        for &(p, q) in &[(a, b), (b, c), (c, a)] {
            let key = if p < q { (p, q) } else { (q, p) };
            if *edge_count.get(&key).unwrap_or(&0) < 2 {
                boundary.push(f);
                break;
            }
        }
    }
    boundary
}

/// Peel one layer of boundary faces from `active_faces`.
/// Returns a new [`FacePeelResult`].
#[allow(dead_code)]
pub fn peel_one_layer(indices: &[u32], active_faces: &[usize]) -> FacePeelResult {
    if active_faces.is_empty() {
        return FacePeelResult {
            retained: vec![],
            peeled: vec![],
        };
    }

    // Rebuild a dense index list from active faces
    let n = active_faces.len();
    let mut sub_indices: Vec<u32> = Vec::with_capacity(n * 3);
    for &f in active_faces {
        let base = f * 3;
        if base + 2 < indices.len() {
            sub_indices.push(indices[base]);
            sub_indices.push(indices[base + 1]);
            sub_indices.push(indices[base + 2]);
        }
    }

    let bnd = boundary_faces(&sub_indices, n);
    let bnd_set: HashSet<usize> = bnd.into_iter().collect();

    let mut peeled = Vec::new();
    let mut retained = Vec::new();
    for (local, &global) in active_faces.iter().enumerate() {
        if bnd_set.contains(&local) {
            peeled.push(global);
        } else {
            retained.push(global);
        }
    }

    FacePeelResult { retained, peeled }
}

/// Peel `layers` layers off a mesh. Returns all intermediate results.
#[allow(dead_code)]
pub fn peel_layers(indices: &[u32], face_count: usize, layers: usize) -> Vec<FacePeelResult> {
    let mut results = Vec::with_capacity(layers);
    let mut active: Vec<usize> = (0..face_count).collect();
    for _ in 0..layers {
        if active.is_empty() {
            break;
        }
        let r = peel_one_layer(indices, &active);
        active.clone_from(&r.retained);
        results.push(r);
    }
    results
}

/// Count the number of faces remaining after `layers` peels.
#[allow(dead_code)]
pub fn remaining_after_peels(indices: &[u32], face_count: usize, layers: usize) -> usize {
    let results = peel_layers(indices, face_count, layers);
    results.last().map_or(face_count, |r| r.retained.len())
}

/// Serialise a single peel result as JSON.
#[allow(dead_code)]
pub fn peel_result_to_json(result: &FacePeelResult) -> String {
    format!(
        "{{\"retained\":{},\"peeled\":{}}}",
        result.retained.len(),
        result.peeled.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // A simple 2x2 quad mesh triangulated into 4 triangles.
    // Vertex grid:
    //  0---1---2
    //  |\ |\ |
    //  | \| \|
    //  3---4---5
    //  |\ |\ |
    //  | \| \|
    //  6---7---8
    fn grid_indices() -> Vec<u32> {
        vec![
            0, 3, 1, 1, 3, 4, // top-left quad
            1, 4, 2, 2, 4, 5, // top-right quad
            3, 6, 4, 4, 6, 7, // bottom-left quad
            4, 7, 5, 5, 7, 8, // bottom-right quad
        ]
    }

    #[test]
    fn test_build_adjacency_non_empty() {
        let idx = grid_indices();
        let adj = build_face_adjacency(&idx, 8);
        assert_eq!(adj.len(), 8);
    }

    #[test]
    fn test_boundary_faces_non_empty() {
        let idx = grid_indices();
        let bnd = boundary_faces(&idx, 8);
        assert!(!bnd.is_empty());
    }

    #[test]
    fn test_peel_one_layer_returns_result() {
        let idx = grid_indices();
        let all: Vec<usize> = (0..8).collect();
        let result = peel_one_layer(&idx, &all);
        assert!(!result.peeled.is_empty());
    }

    #[test]
    fn test_peel_reduces_faces() {
        let idx = grid_indices();
        let all: Vec<usize> = (0..8).collect();
        let result = peel_one_layer(&idx, &all);
        assert!(result.retained.len() < 8);
    }

    #[test]
    fn test_peel_layers_multiple() {
        let idx = grid_indices();
        let results = peel_layers(&idx, 8, 2);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_empty_active_faces() {
        let idx = grid_indices();
        let result = peel_one_layer(&idx, &[]);
        assert!(result.retained.is_empty());
        assert!(result.peeled.is_empty());
    }

    #[test]
    fn test_remaining_after_peels() {
        let idx = grid_indices();
        let remaining = remaining_after_peels(&idx, 8, 1);
        assert!(remaining <= 8);
    }

    #[test]
    fn test_peel_result_to_json() {
        let r = FacePeelResult {
            retained: vec![0, 1],
            peeled: vec![2, 3],
        };
        let json = peel_result_to_json(&r);
        assert!(json.contains("retained"));
    }

    #[test]
    fn test_peel_peeled_plus_retained() {
        let idx = grid_indices();
        let all: Vec<usize> = (0..8).collect();
        let result = peel_one_layer(&idx, &all);
        assert_eq!(result.retained.len() + result.peeled.len(), 8);
    }

    #[test]
    fn test_boundary_single_triangle() {
        // A single triangle — all 3 edges are boundary
        let idx = vec![0u32, 1, 2];
        let bnd = boundary_faces(&idx, 1);
        assert_eq!(bnd.len(), 1);
    }
}
