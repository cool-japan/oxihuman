// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mesh topology analysis.
//!
//! Computes genus, Euler characteristic, and connectivity statistics for a
//! triangle mesh by counting vertices, edges, faces, boundary edges, and
//! non-manifold elements.

use std::collections::HashMap;

// ─── Structures ──────────────────────────────────────────────────────────────

/// Configuration for topology analysis (reserved for future extensions).
#[allow(dead_code)]
pub struct TopologyConfig {
    /// If true, treat boundary edges as manifold (open meshes are OK).
    pub allow_boundary: bool,
}

/// Topology statistics for a triangle mesh.
#[allow(dead_code)]
pub struct TopologyStats {
    /// Number of vertices (as declared by caller).
    pub n_verts: usize,
    /// Number of unique edges.
    pub n_edges: usize,
    /// Number of triangle faces.
    pub n_faces: usize,
    /// Number of edges referenced by exactly one face (boundary edges).
    pub n_boundary_edges: usize,
    /// Number of edges referenced by more than two faces (non-manifold).
    pub n_nonmanifold_edges: usize,
    /// True when every edge is shared by at most 2 faces.
    pub is_manifold: bool,
    /// Euler characteristic χ = V − E + F.
    pub euler: i32,
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn edge_key(a: u32, b: u32) -> (u32, u32) {
    if a < b { (a, b) } else { (b, a) }
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Default topology configuration.
#[allow(dead_code)]
pub fn default_topology_config() -> TopologyConfig {
    TopologyConfig {
        allow_boundary: true,
    }
}

/// Compute topology statistics for a triangle mesh.
///
/// `faces` is a slice of triangles (vertex-index triples).
/// `n_verts` is the total declared vertex count (may exceed referenced verts).
#[allow(dead_code)]
pub fn compute_topology(
    faces: &[[u32; 3]],
    n_verts: usize,
    _cfg: &TopologyConfig,
) -> TopologyStats {
    // Count how many faces reference each directed edge (a→b canonical).
    let mut edge_face_count: HashMap<(u32, u32), u32> = HashMap::new();

    for f in faces {
        let edges = [
            edge_key(f[0], f[1]),
            edge_key(f[1], f[2]),
            edge_key(f[2], f[0]),
        ];
        for e in edges {
            *edge_face_count.entry(e).or_insert(0) += 1;
        }
    }

    let n_edges = edge_face_count.len();
    let n_boundary_edges = edge_face_count.values().filter(|&&c| c == 1).count();
    let n_nonmanifold_edges = edge_face_count.values().filter(|&&c| c > 2).count();
    let is_manifold = n_nonmanifold_edges == 0;

    let v = n_verts as i32;
    let e = n_edges as i32;
    let f = faces.len() as i32;
    let euler = v - e + f;

    TopologyStats {
        n_verts,
        n_edges,
        n_faces: faces.len(),
        n_boundary_edges,
        n_nonmanifold_edges,
        is_manifold,
        euler,
    }
}

/// Euler characteristic χ = V − E + F.
#[allow(dead_code)]
pub fn topology_euler_characteristic(stats: &TopologyStats) -> i32 {
    stats.euler
}

/// Genus of a closed orientable surface: g = (2 − χ) / 2.
/// Returns 0 if the mesh is not closed (has boundary edges).
#[allow(dead_code)]
pub fn topology_genus(stats: &TopologyStats) -> i32 {
    if stats.n_boundary_edges > 0 {
        return 0;
    }
    (2 - stats.euler) / 2
}

/// True when every edge is shared by at most 2 faces.
#[allow(dead_code)]
pub fn topology_is_manifold(stats: &TopologyStats) -> bool {
    stats.is_manifold
}

/// Number of unique edges.
#[allow(dead_code)]
pub fn topology_edge_count(stats: &TopologyStats) -> usize {
    stats.n_edges
}

/// Number of boundary edges (referenced by exactly one face).
#[allow(dead_code)]
pub fn topology_boundary_edge_count(stats: &TopologyStats) -> usize {
    stats.n_boundary_edges
}

/// Number of vertices declared for the mesh.
#[allow(dead_code)]
pub fn topology_vertex_count(stats: &TopologyStats) -> usize {
    stats.n_verts
}

/// Number of triangle faces.
#[allow(dead_code)]
pub fn topology_face_count(stats: &TopologyStats) -> usize {
    stats.n_faces
}

/// Human-readable summary of topology statistics.
#[allow(dead_code)]
pub fn topology_to_string(stats: &TopologyStats) -> String {
    format!(
        "V={} E={} F={} χ={} genus={} manifold={} boundary_edges={} nonmanifold_edges={}",
        stats.n_verts,
        stats.n_edges,
        stats.n_faces,
        stats.euler,
        topology_genus(stats),
        stats.is_manifold,
        stats.n_boundary_edges,
        stats.n_nonmanifold_edges,
    )
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Single triangle (open mesh).
    fn single_tri() -> (Vec<[u32; 3]>, usize) {
        (vec![[0, 1, 2]], 3)
    }

    /// Tetrahedron (closed, genus 0, χ=2).
    fn tetrahedron() -> (Vec<[u32; 3]>, usize) {
        let faces = vec![[0u32, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]];
        (faces, 4)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_topology_config();
        assert!(cfg.allow_boundary);
    }

    #[test]
    fn test_single_tri_vertex_count() {
        let (faces, n) = single_tri();
        let cfg = default_topology_config();
        let stats = compute_topology(&faces, n, &cfg);
        assert_eq!(topology_vertex_count(&stats), 3);
    }

    #[test]
    fn test_single_tri_face_count() {
        let (faces, n) = single_tri();
        let cfg = default_topology_config();
        let stats = compute_topology(&faces, n, &cfg);
        assert_eq!(topology_face_count(&stats), 1);
    }

    #[test]
    fn test_single_tri_edge_count() {
        let (faces, n) = single_tri();
        let cfg = default_topology_config();
        let stats = compute_topology(&faces, n, &cfg);
        assert_eq!(topology_edge_count(&stats), 3);
    }

    #[test]
    fn test_single_tri_boundary_edges() {
        let (faces, n) = single_tri();
        let cfg = default_topology_config();
        let stats = compute_topology(&faces, n, &cfg);
        assert_eq!(topology_boundary_edge_count(&stats), 3);
    }

    #[test]
    fn test_tetrahedron_euler() {
        let (faces, n) = tetrahedron();
        let cfg = default_topology_config();
        let stats = compute_topology(&faces, n, &cfg);
        // Tetrahedron: V=4, E=6, F=4 → χ=2
        assert_eq!(topology_euler_characteristic(&stats), 2);
    }

    #[test]
    fn test_tetrahedron_genus_zero() {
        let (faces, n) = tetrahedron();
        let cfg = default_topology_config();
        let stats = compute_topology(&faces, n, &cfg);
        assert_eq!(topology_genus(&stats), 0);
    }

    #[test]
    fn test_tetrahedron_is_manifold() {
        let (faces, n) = tetrahedron();
        let cfg = default_topology_config();
        let stats = compute_topology(&faces, n, &cfg);
        assert!(topology_is_manifold(&stats));
    }

    #[test]
    fn test_tetrahedron_no_boundary() {
        let (faces, n) = tetrahedron();
        let cfg = default_topology_config();
        let stats = compute_topology(&faces, n, &cfg);
        assert_eq!(topology_boundary_edge_count(&stats), 0);
    }

    #[test]
    fn test_topology_to_string_contains_euler() {
        let (faces, n) = tetrahedron();
        let cfg = default_topology_config();
        let stats = compute_topology(&faces, n, &cfg);
        let s = topology_to_string(&stats);
        assert!(s.contains("χ=2"), "string={}", s);
    }

    #[test]
    fn test_empty_mesh() {
        let cfg = default_topology_config();
        let stats = compute_topology(&[], 0, &cfg);
        assert_eq!(topology_face_count(&stats), 0);
        assert_eq!(topology_edge_count(&stats), 0);
        assert_eq!(topology_euler_characteristic(&stats), 0);
    }
}
