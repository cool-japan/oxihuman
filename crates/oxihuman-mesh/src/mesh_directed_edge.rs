// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Directed edge data structure for mesh connectivity.

use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;

/// A directed (half) edge.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirectedEdge {
    pub from: u32,
    pub to: u32,
    pub face: u32,
}

/// Directed edge mesh structure.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DirectedEdgeMesh {
    pub edges: Vec<DirectedEdge>,
    pub twin_map: HashMap<(u32, u32), usize>,
}

/// Build directed edges from triangle indices.
#[allow(dead_code)]
pub fn build_directed_edges(indices: &[u32]) -> DirectedEdgeMesh {
    let tri_count = indices.len() / 3;
    let mut edges = Vec::with_capacity(tri_count * 3);
    let mut twin_map: HashMap<(u32, u32), usize> = HashMap::new();

    #[allow(clippy::needless_range_loop)]
    for t in 0..tri_count {
        let vs = [indices[t * 3], indices[t * 3 + 1], indices[t * 3 + 2]];
        for i in 0..3 {
            let from = vs[i];
            let to = vs[(i + 1) % 3];
            let idx = edges.len();
            edges.push(DirectedEdge {
                from,
                to,
                face: t as u32,
            });
            twin_map.insert((from, to), idx);
        }
    }

    DirectedEdgeMesh { edges, twin_map }
}

/// Edge count.
#[allow(dead_code)]
pub fn directed_edge_count(mesh: &DirectedEdgeMesh) -> usize {
    mesh.edges.len()
}

/// Find the twin of a directed edge.
#[allow(dead_code)]
pub fn find_twin(mesh: &DirectedEdgeMesh, from: u32, to: u32) -> Option<usize> {
    mesh.twin_map.get(&(to, from)).copied()
}

/// Check if an edge has a twin (is interior).
#[allow(dead_code)]
pub fn has_twin(mesh: &DirectedEdgeMesh, from: u32, to: u32) -> bool {
    mesh.twin_map.contains_key(&(to, from))
}

/// Count boundary edges (no twin).
#[allow(dead_code)]
pub fn boundary_edge_count(mesh: &DirectedEdgeMesh) -> usize {
    mesh.edges
        .iter()
        .filter(|e| !has_twin(mesh, e.from, e.to))
        .count()
}

/// Count interior edges (have twin).
#[allow(dead_code)]
pub fn interior_edge_count(mesh: &DirectedEdgeMesh) -> usize {
    mesh.edges
        .iter()
        .filter(|e| has_twin(mesh, e.from, e.to))
        .count()
}

/// Get edges for a face.
#[allow(dead_code)]
pub fn edges_for_face(mesh: &DirectedEdgeMesh, face: u32) -> Vec<&DirectedEdge> {
    mesh.edges.iter().filter(|e| e.face == face).collect()
}

/// FRAC_PI_2 reference.
#[allow(dead_code)]
pub fn half_pi_ref() -> f32 {
    FRAC_PI_2
}

/// Export to JSON.
#[allow(dead_code)]
pub fn directed_edge_to_json(mesh: &DirectedEdgeMesh) -> String {
    format!(
        "{{\"edges\":{},\"boundary\":{}}}",
        directed_edge_count(mesh),
        boundary_edge_count(mesh)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_single_tri() {
        let m = build_directed_edges(&[0, 1, 2]);
        assert_eq!(directed_edge_count(&m), 3);
    }

    #[test]
    fn test_boundary_single_tri() {
        let m = build_directed_edges(&[0, 1, 2]);
        assert_eq!(boundary_edge_count(&m), 3);
    }

    #[test]
    fn test_two_tris_shared() {
        let m = build_directed_edges(&[0, 1, 2, 2, 1, 3]);
        assert!(interior_edge_count(&m) >= 2);
    }

    #[test]
    fn test_find_twin() {
        let m = build_directed_edges(&[0, 1, 2, 2, 1, 3]);
        assert!(find_twin(&m, 1, 2).is_some());
    }

    #[test]
    fn test_has_twin() {
        let m = build_directed_edges(&[0, 1, 2, 2, 1, 3]);
        assert!(has_twin(&m, 1, 2));
    }

    #[test]
    fn test_no_twin() {
        let m = build_directed_edges(&[0, 1, 2]);
        assert!(!has_twin(&m, 0, 1));
    }

    #[test]
    fn test_edges_for_face() {
        let m = build_directed_edges(&[0, 1, 2, 3, 4, 5]);
        assert_eq!(edges_for_face(&m, 0).len(), 3);
    }

    #[test]
    fn test_empty() {
        let m = build_directed_edges(&[]);
        assert_eq!(directed_edge_count(&m), 0);
    }

    #[test]
    fn test_half_pi() {
        assert!((half_pi_ref() - FRAC_PI_2).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let m = build_directed_edges(&[0, 1, 2]);
        let j = directed_edge_to_json(&m);
        assert!(j.contains("\"edges\":3"));
    }
}
