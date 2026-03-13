// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;

// ─── helpers ─────────────────────────────────────────────────────────────────

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

// ─── adjacency & boundary ────────────────────────────────────────────────────

/// Build per-vertex adjacency list from triangle indices.
fn build_adjacency(n_verts: usize, indices: &[u32]) -> Vec<Vec<u32>> {
    let mut adj: Vec<Vec<u32>> = vec![vec![]; n_verts];
    for tri in indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= n_verts || i1 >= n_verts || i2 >= n_verts {
            continue;
        }
        adj[i0].push(tri[1]);
        adj[i0].push(tri[2]);
        adj[i1].push(tri[0]);
        adj[i1].push(tri[2]);
        adj[i2].push(tri[0]);
        adj[i2].push(tri[1]);
    }
    adj
}

/// Ordered edge key: smaller index first.
#[inline]
fn edge_key(a: u32, b: u32) -> (u32, u32) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

/// Build a bitmask of boundary vertices: those that lie on edges shared by
/// exactly one triangle.
fn build_boundary_flags(n_verts: usize, indices: &[u32]) -> Vec<bool> {
    let mut edge_count: HashMap<(u32, u32), u32> = HashMap::new();
    for tri in indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0], tri[1], tri[2]);
        *edge_count.entry(edge_key(i0, i1)).or_insert(0) += 1;
        *edge_count.entry(edge_key(i1, i2)).or_insert(0) += 1;
        *edge_count.entry(edge_key(i0, i2)).or_insert(0) += 1;
    }
    let mut boundary = vec![false; n_verts];
    for ((a, b), count) in &edge_count {
        if *count == 1 {
            if (*a as usize) < n_verts {
                boundary[*a as usize] = true;
            }
            if (*b as usize) < n_verts {
                boundary[*b as usize] = true;
            }
        }
    }
    boundary
}

// ─── public API ──────────────────────────────────────────────────────────────

/// Configuration for smoothing operations.
pub struct SmoothConfig {
    /// Number of smoothing iterations.
    pub iterations: usize,
    /// Blend factor [0..1]: 0 = no change, 1 = full Laplacian.
    pub factor: f32,
    /// If true, boundary vertices (on open edges) are not moved.
    pub preserve_boundary: bool,
}

impl SmoothConfig {
    pub fn new(iterations: usize, factor: f32) -> Self {
        Self {
            iterations,
            factor,
            preserve_boundary: true,
        }
    }

    /// Gentle 3-iteration smooth, factor=0.5
    pub fn gentle() -> Self {
        Self::new(3, 0.5)
    }

    /// Strong 10-iteration smooth, factor=0.8
    pub fn strong() -> Self {
        Self::new(10, 0.8)
    }
}

impl Default for SmoothConfig {
    fn default() -> Self {
        Self::gentle()
    }
}

/// Perform one Laplacian pass on `positions` in-place, blending each vertex
/// toward the average of its neighbours by `factor`.
fn laplacian_pass(
    positions: &mut [[f32; 3]],
    adj: &[Vec<u32>],
    boundary: &[bool],
    factor: f32,
    preserve_boundary: bool,
) {
    let old = positions.to_owned();
    for (v, pos) in positions.iter_mut().enumerate() {
        if preserve_boundary && boundary[v] {
            continue;
        }
        let neighbours = &adj[v];
        if neighbours.is_empty() {
            continue;
        }
        let mut sum = [0.0f32; 3];
        for &nb in neighbours {
            sum = add3(sum, old[nb as usize]);
        }
        let avg = scale3(sum, 1.0 / neighbours.len() as f32);
        *pos = lerp3(old[v], avg, factor);
    }
}

/// Perform Laplacian smoothing: each vertex moves toward its neighbours'
/// average. Returns a new [`MeshBuffers`] with smoothed positions; normals are
/// recomputed, UVs and indices are copied unchanged.
pub fn laplacian_smooth(mesh: &MeshBuffers, config: &SmoothConfig) -> MeshBuffers {
    let n_verts = mesh.positions.len();

    let adj = build_adjacency(n_verts, &mesh.indices);
    let boundary = if config.preserve_boundary {
        build_boundary_flags(n_verts, &mesh.indices)
    } else {
        vec![false; n_verts]
    };

    let mut positions = mesh.positions.clone();

    for _ in 0..config.iterations {
        laplacian_pass(
            &mut positions,
            &adj,
            &boundary,
            config.factor,
            config.preserve_boundary,
        );
    }

    let mut result = MeshBuffers {
        positions,
        normals: mesh.normals.clone(),
        tangents: mesh.tangents.clone(),
        uvs: mesh.uvs.clone(),
        colors: mesh.colors.clone(),
        indices: mesh.indices.clone(),
        has_suit: mesh.has_suit,
    };
    compute_normals(&mut result);
    result
}

/// Taubin smoothing: alternates positive (λ) and negative (μ) Laplacian
/// passes to avoid mesh shrinkage.  Typical values: lambda=0.5, mu=-0.53.
pub fn taubin_smooth(mesh: &MeshBuffers, iterations: usize, lambda: f32, mu: f32) -> MeshBuffers {
    let n_verts = mesh.positions.len();

    let adj = build_adjacency(n_verts, &mesh.indices);
    let boundary = build_boundary_flags(n_verts, &mesh.indices);

    let mut positions = mesh.positions.clone();

    for _ in 0..iterations {
        // Positive pass (shrink)
        laplacian_pass(&mut positions, &adj, &boundary, lambda, true);
        // Negative pass (expand)
        laplacian_pass(&mut positions, &adj, &boundary, mu, true);
    }

    let mut result = MeshBuffers {
        positions,
        normals: mesh.normals.clone(),
        tangents: mesh.tangents.clone(),
        uvs: mesh.uvs.clone(),
        colors: mesh.colors.clone(),
        indices: mesh.indices.clone(),
        has_suit: mesh.has_suit,
    };
    compute_normals(&mut result);
    result
}

/// Smooth only the vertex normals (positions unchanged): average normals of
/// neighbouring vertices and renormalise after each iteration.  Useful for
/// visual normal smoothing without changing geometry.
pub fn smooth_normals(mesh: &mut MeshBuffers, iterations: usize) {
    let n_verts = mesh.positions.len();
    let adj = build_adjacency(n_verts, &mesh.indices);

    for _ in 0..iterations {
        let old = mesh.normals.clone();
        for (v, norm) in mesh.normals.iter_mut().enumerate() {
            let neighbours = &adj[v];
            if neighbours.is_empty() {
                continue;
            }
            let mut sum = old[v];
            for &nb in neighbours {
                sum = add3(sum, old[nb as usize]);
            }
            let count = 1 + neighbours.len();
            *norm = normalize3(scale3(sum, 1.0 / count as f32));
        }
    }
}

// ─── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn grid_mesh(n: usize) -> MeshBuffers {
        // n×n vertices in XZ plane (y=0)
        let mut positions = Vec::new();
        for i in 0..n {
            for j in 0..n {
                positions.push([i as f32, 0.0, j as f32]);
            }
        }
        let mut indices = Vec::new();
        for i in 0..n - 1 {
            for j in 0..n - 1 {
                let base = (i * n + j) as u32;
                indices.push(base);
                indices.push(base + 1);
                indices.push(base + n as u32);
                indices.push(base + 1);
                indices.push(base + n as u32 + 1);
                indices.push(base + n as u32);
            }
        }
        MeshBuffers {
            positions,
            normals: vec![[0.0, 1.0, 0.0]; n * n],
            uvs: vec![[0.0, 0.0]; n * n],
            tangents: vec![],
            colors: None,
            indices,
            has_suit: true,
        }
    }

    #[test]
    fn smooth_preserves_vertex_count() {
        let smoothed = laplacian_smooth(&grid_mesh(5), &SmoothConfig::gentle());
        assert_eq!(smoothed.positions.len(), 25);
    }

    #[test]
    fn smooth_preserves_index_count() {
        let mesh = grid_mesh(5);
        let original_index_count = mesh.indices.len();
        let smoothed = laplacian_smooth(&mesh, &SmoothConfig::gentle());
        assert_eq!(smoothed.indices.len(), original_index_count);
    }

    #[test]
    fn smooth_interior_moves_toward_neighbors() {
        let mut mesh = grid_mesh(5);
        // Set the centre vertex (index 12 = row 2, col 2) to a spike.
        mesh.positions[12] = [2.0, 10.0, 2.0];
        let config = SmoothConfig::new(5, 0.8);
        let smoothed = laplacian_smooth(&mesh, &config);
        assert!(
            smoothed.positions[12][1] < 10.0,
            "spike y should decrease after smoothing, got {}",
            smoothed.positions[12][1]
        );
    }

    #[test]
    fn taubin_smooth_less_shrinkage() {
        // Measure sum of |y| after spiking a central vertex.
        // Taubin should preserve volume better than plain Laplacian.
        let mut mesh_lap = grid_mesh(5);
        mesh_lap.positions[12] = [2.0, 5.0, 2.0];
        let mesh_taub = mesh_lap.clone();

        let lap = laplacian_smooth(&mesh_lap, &SmoothConfig::new(10, 0.5));
        let taub = taubin_smooth(&mesh_taub, 10, 0.5, -0.53);

        // Both should reduce the spike, but Taubin's spike should be larger.
        let lap_spike_y = lap.positions[12][1].abs();
        let taub_spike_y = taub.positions[12][1].abs();
        assert!(
            taub_spike_y >= lap_spike_y,
            "Taubin should preserve the spike better than Laplacian: taub={} lap={}",
            taub_spike_y,
            lap_spike_y
        );
    }

    #[test]
    fn smooth_normals_length_unchanged() {
        let mut mesh = grid_mesh(5);
        smooth_normals(&mut mesh, 3);
        for (i, n) in mesh.normals.iter().enumerate() {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!(
                (len - 1.0).abs() < 1e-5,
                "normal {} has length {}, expected ~1.0",
                i,
                len
            );
        }
    }

    #[test]
    fn zero_iterations_no_change() {
        let mesh = grid_mesh(5);
        let original_positions = mesh.positions.clone();
        let smoothed = laplacian_smooth(&mesh, &SmoothConfig::new(0, 0.5));
        for (orig, smth) in original_positions.iter().zip(smoothed.positions.iter()) {
            assert_eq!(
                orig, smth,
                "positions should be identical with 0 iterations"
            );
        }
    }
}
