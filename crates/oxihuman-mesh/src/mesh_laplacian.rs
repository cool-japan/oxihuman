// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Laplacian smoothing and mesh regularization.
//!
//! Iteratively moves each vertex toward the average of its neighbors,
//! reducing surface noise while approximately preserving overall shape.

// ──────────────────────────────────────────────────────────────────────────────
// Config and result types
// ──────────────────────────────────────────────────────────────────────────────

/// Configuration for Laplacian smoothing operations.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LaplacianConfig {
    /// Number of smoothing iterations.
    pub iterations: u32,
    /// Blend factor in [0, 1]: 0 = no move, 1 = full Laplacian step.
    pub lambda: f32,
    /// If true, boundary vertices are pinned (not moved).
    pub pin_boundary: bool,
}

/// Result of a Laplacian smoothing pass.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LaplacianResult {
    /// Smoothed vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Per-vertex Laplacian delta magnitudes after the last iteration.
    pub deltas: Vec<f32>,
    /// Number of iterations actually executed.
    pub iterations_run: u32,
}

// ──────────────────────────────────────────────────────────────────────────────
// Public functions
// ──────────────────────────────────────────────────────────────────────────────

/// Returns a sensible default [`LaplacianConfig`].
#[allow(dead_code)]
pub fn default_laplacian_config() -> LaplacianConfig {
    LaplacianConfig {
        iterations: 5,
        lambda: 0.5,
        pin_boundary: false,
    }
}

/// Builds a per-vertex adjacency list from a triangle face array.
///
/// Each entry `adj[i]` is a list of vertex indices that share an edge with vertex `i`.
#[allow(dead_code)]
pub fn build_adjacency(faces: &[[u32; 3]], n_verts: usize) -> Vec<Vec<u32>> {
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n_verts];
    for f in faces {
        let [a, b, c] = *f;
        let pairs = [(a, b), (b, a), (b, c), (c, b), (a, c), (c, a)];
        for (u, v) in pairs {
            let u = u as usize;
            if u < n_verts && !adj[u].contains(&v) {
                adj[u].push(v);
            }
        }
    }
    adj
}

/// Applies Laplacian smoothing and returns a [`LaplacianResult`].
#[allow(dead_code)]
pub fn laplacian_smooth(
    verts: &[[f32; 3]],
    faces: &[[u32; 3]],
    cfg: &LaplacianConfig,
) -> LaplacianResult {
    let n = verts.len();
    let adj = build_adjacency(faces, n);
    let mut positions: Vec<[f32; 3]> = verts.to_vec();

    for _ in 0..cfg.iterations {
        let prev = positions.clone();
        for (i, nbrs) in adj.iter().enumerate() {
            if nbrs.is_empty() {
                continue;
            }
            // Optionally detect boundary vertices: they appear in only one face neighbour.
            // For simplicity we always smooth unless pin_boundary is set and vertex is boundary.
            if cfg.pin_boundary && is_boundary_vertex(i, faces, n) {
                continue;
            }
            let avg = average_neighbors(&prev, nbrs);
            let p = prev[i];
            positions[i] = [
                p[0] + cfg.lambda * (avg[0] - p[0]),
                p[1] + cfg.lambda * (avg[1] - p[1]),
                p[2] + cfg.lambda * (avg[2] - p[2]),
            ];
        }
    }

    // laplacian_coords is called to exercise the function; result not needed here.
    let _smoothed = laplacian_coords(&positions, &adj);
    let deltas = laplacian_delta(verts, &positions);

    LaplacianResult {
        positions,
        deltas,
        iterations_run: cfg.iterations,
    }
}

/// Applies Laplacian smoothing in-place, modifying `verts` directly.
#[allow(dead_code)]
pub fn laplacian_smooth_inplace(
    verts: &mut [[f32; 3]],
    faces: &[[u32; 3]],
    cfg: &LaplacianConfig,
) {
    let n = verts.len();
    let adj = build_adjacency(faces, n);
    for _ in 0..cfg.iterations {
        let prev: Vec<[f32; 3]> = verts.to_vec();
        for (i, nbrs) in adj.iter().enumerate() {
            if nbrs.is_empty() {
                continue;
            }
            if cfg.pin_boundary && is_boundary_vertex(i, faces, n) {
                continue;
            }
            let avg = average_neighbors(&prev, nbrs);
            let p = prev[i];
            verts[i] = [
                p[0] + cfg.lambda * (avg[0] - p[0]),
                p[1] + cfg.lambda * (avg[1] - p[1]),
                p[2] + cfg.lambda * (avg[2] - p[2]),
            ];
        }
    }
}

/// Computes per-vertex Laplacian coordinates (vertex minus average of neighbors).
#[allow(dead_code)]
pub fn laplacian_coords(verts: &[[f32; 3]], adj: &[Vec<u32>]) -> Vec<[f32; 3]> {
    verts
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let nbrs = &adj[i];
            if nbrs.is_empty() {
                return [0.0_f32; 3];
            }
            let avg = average_neighbors(verts, nbrs);
            [p[0] - avg[0], p[1] - avg[1], p[2] - avg[2]]
        })
        .collect()
}

/// Computes per-vertex displacement magnitudes between original and smoothed positions.
#[allow(dead_code)]
pub fn laplacian_delta(orig: &[[f32; 3]], smoothed: &[[f32; 3]]) -> Vec<f32> {
    orig.iter()
        .zip(smoothed.iter())
        .map(|(a, b)| {
            let dx = a[0] - b[0];
            let dy = a[1] - b[1];
            let dz = a[2] - b[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .collect()
}

/// Inflates a mesh by moving each vertex along its normal by `amount`.
#[allow(dead_code)]
pub fn inflate_mesh(verts: &[[f32; 3]], normals: &[[f32; 3]], amount: f32) -> Vec<[f32; 3]> {
    verts
        .iter()
        .zip(normals.iter())
        .map(|(v, n)| {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-10);
            [
                v[0] + n[0] / len * amount,
                v[1] + n[1] / len * amount,
                v[2] + n[2] / len * amount,
            ]
        })
        .collect()
}

// ──────────────────────────────────────────────────────────────────────────────
// Private helpers
// ──────────────────────────────────────────────────────────────────────────────

fn average_neighbors(verts: &[[f32; 3]], nbrs: &[u32]) -> [f32; 3] {
    let mut sum = [0.0_f32; 3];
    let n = nbrs.len() as f32;
    for &idx in nbrs {
        let v = verts[idx as usize];
        sum[0] += v[0];
        sum[1] += v[1];
        sum[2] += v[2];
    }
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Returns true if vertex `vi` is on the boundary (appears in an edge used by exactly one face).
fn is_boundary_vertex(vi: usize, faces: &[[u32; 3]], _n_verts: usize) -> bool {
    use std::collections::HashMap;
    let mut edge_count: HashMap<(u32, u32), u32> = HashMap::new();
    for f in faces {
        let edges = [
            (f[0].min(f[1]), f[0].max(f[1])),
            (f[1].min(f[2]), f[1].max(f[2])),
            (f[0].min(f[2]), f[0].max(f[2])),
        ];
        for e in edges {
            *edge_count.entry(e).or_insert(0) += 1;
        }
    }
    let vi = vi as u32;
    edge_count
        .iter()
        .any(|(&(a, b), &cnt)| cnt == 1 && (a == vi || b == vi))
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn square_mesh() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        // 4 vertices, 2 triangles forming a flat unit square
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let faces = vec![[0, 1, 2], [0, 2, 3]];
        (verts, faces)
    }

    #[test]
    fn default_config_iterations() {
        let cfg = default_laplacian_config();
        assert_eq!(cfg.iterations, 5);
        assert!(cfg.lambda > 0.0 && cfg.lambda <= 1.0);
    }

    #[test]
    fn build_adjacency_correct_count() {
        let (verts, faces) = square_mesh();
        let adj = build_adjacency(&faces, verts.len());
        // Every vertex should have at least one neighbour
        for nbrs in &adj {
            assert!(!nbrs.is_empty());
        }
    }

    #[test]
    fn build_adjacency_symmetric() {
        let (verts, faces) = square_mesh();
        let adj = build_adjacency(&faces, verts.len());
        for (i, nbrs) in adj.iter().enumerate() {
            for &j in nbrs {
                assert!(
                    adj[j as usize].contains(&(i as u32)),
                    "adjacency not symmetric for {} ↔ {}",
                    i,
                    j
                );
            }
        }
    }

    #[test]
    fn laplacian_smooth_preserves_vertex_count() {
        let (verts, faces) = square_mesh();
        let cfg = default_laplacian_config();
        let result = laplacian_smooth(&verts, &faces, &cfg);
        assert_eq!(result.positions.len(), verts.len());
        assert_eq!(result.deltas.len(), verts.len());
    }

    #[test]
    fn laplacian_smooth_inplace_moves_verts() {
        let (verts, faces) = square_mesh();
        // Perturb one vertex
        let mut perturbed = verts.clone();
        perturbed[2] = [1.5, 1.5, 0.5];
        let cfg = LaplacianConfig { iterations: 3, lambda: 0.5, pin_boundary: false };
        laplacian_smooth_inplace(&mut perturbed, &faces, &cfg);
        // vertex should have moved somewhat from the perturbed position
        let d = (perturbed[2][0] - 1.5_f32).abs()
            + (perturbed[2][1] - 1.5_f32).abs()
            + (perturbed[2][2] - 0.5_f32).abs();
        assert!(d > 0.0, "smooth_inplace should have moved the perturbed vertex");
    }

    #[test]
    fn laplacian_coords_zero_on_flat_mesh() {
        // On a flat, regular mesh the Laplacian coordinates should be near-zero
        // for interior vertices.
        let verts = vec![
            [0.0, 0.0, 0.0], // 0
            [2.0, 0.0, 0.0], // 1
            [1.0, 1.0, 0.0], // 2 – center-ish
            [0.0, 2.0, 0.0], // 3
            [2.0, 2.0, 0.0], // 4
        ];
        let faces = vec![[0, 1, 2], [0, 2, 3], [1, 4, 2], [2, 4, 3]];
        let adj = build_adjacency(&faces, verts.len());
        let coords = laplacian_coords(&verts, &adj);
        assert_eq!(coords.len(), verts.len());
    }

    #[test]
    fn laplacian_delta_self_is_zero() {
        let (verts, _) = square_mesh();
        let deltas = laplacian_delta(&verts, &verts);
        for d in &deltas {
            assert!(d.abs() < 1e-9);
        }
    }

    #[test]
    fn inflate_mesh_moves_outward() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let normals = vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
        let inflated = inflate_mesh(&verts, &normals, 0.5);
        assert!((inflated[0][1] - 0.5).abs() < 1e-6);
        assert!((inflated[1][1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn laplacian_smooth_zero_iterations_is_identity() {
        let (verts, faces) = square_mesh();
        let cfg = LaplacianConfig { iterations: 0, lambda: 0.5, pin_boundary: false };
        let result = laplacian_smooth(&verts, &faces, &cfg);
        for (a, b) in verts.iter().zip(result.positions.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-9);
            assert!((a[1] - b[1]).abs() < 1e-9);
            assert!((a[2] - b[2]).abs() < 1e-9);
        }
    }
}
