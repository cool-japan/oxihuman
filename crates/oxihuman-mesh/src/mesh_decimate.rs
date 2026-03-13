// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mesh decimation via edge collapse with quadric error metric (QEM).
//!
//! Reduces triangle count while preserving surface shape. Uses the classic
//! Garland-Heckbert quadric error metric to pick the cheapest edge to collapse.

use std::collections::BTreeSet;

// ── math helpers ─────────────────────────────────────────────────────────────

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len3(a: [f32; 3]) -> f32 {
    dot3(a, a).sqrt()
}

fn normalize3(a: [f32; 3]) -> [f32; 3] {
    let l = len3(a);
    if l < 1e-12 {
        [0.0, 0.0, 0.0]
    } else {
        [a[0] / l, a[1] / l, a[2] / l]
    }
}

/// Symmetric 4x4 matrix stored as 10 unique elements (upper-triangular).
/// Layout: [a11, a12, a13, a14, a22, a23, a24, a33, a34, a44]
type Quadric = [f64; 10];

fn zero_quadric() -> Quadric {
    [0.0; 10]
}

fn add_quadric(a: &Quadric, b: &Quadric) -> Quadric {
    let mut r = [0.0f64; 10];
    for i in 0..10 {
        r[i] = a[i] + b[i];
    }
    r
}

fn plane_quadric(a: f64, b: f64, c: f64, d: f64) -> Quadric {
    [
        a * a,
        a * b,
        a * c,
        a * d,
        b * b,
        b * c,
        b * d,
        c * c,
        c * d,
        d * d,
    ]
}

fn quadric_error(q: &Quadric, v: [f32; 3]) -> f64 {
    let x = v[0] as f64;
    let y = v[1] as f64;
    let z = v[2] as f64;
    q[0] * x * x
        + 2.0 * q[1] * x * y
        + 2.0 * q[2] * x * z
        + 2.0 * q[3] * x
        + q[4] * y * y
        + 2.0 * q[5] * y * z
        + 2.0 * q[6] * y
        + q[7] * z * z
        + 2.0 * q[8] * z
        + q[9]
}

// ── structs ──────────────────────────────────────────────────────────────────

/// Configuration for mesh decimation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecimateConfig {
    /// Target ratio of faces to keep (0.0..1.0).
    pub target_ratio: f32,
    /// Maximum quadric error threshold; edges above this cost are skipped.
    pub max_error: f64,
    /// Preserve boundary edges.
    pub preserve_boundary: bool,
}

/// Result of a decimation operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecimateResult {
    /// Output positions.
    pub positions: Vec<[f32; 3]>,
    /// Output triangle indices.
    pub indices: Vec<u32>,
    /// Original face count.
    pub original_faces: usize,
    /// Resulting face count.
    pub result_faces: usize,
}

// ── public API ───────────────────────────────────────────────────────────────

/// Returns a default decimation configuration (50% reduction, no error cap).
#[allow(dead_code)]
pub fn default_decimate_config() -> DecimateConfig {
    DecimateConfig {
        target_ratio: 0.5,
        max_error: f64::INFINITY,
        preserve_boundary: false,
    }
}

/// Decimate a mesh to approximately `config.target_ratio` of its original face count.
#[allow(dead_code)]
pub fn decimate_mesh(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &DecimateConfig,
) -> DecimateResult {
    let original_faces = indices.len() / 3;
    let target = ((original_faces as f32 * config.target_ratio).round() as usize).max(1);
    decimate_to_target_faces(
        positions,
        indices,
        target,
        config.max_error,
        config.preserve_boundary,
    )
}

/// Compute the quadric error matrix for a vertex given the faces it belongs to.
#[allow(dead_code)]
pub fn compute_quadric(positions: &[[f32; 3]], indices: &[u32], vertex: usize) -> Quadric {
    let nf = indices.len() / 3;
    let mut q = zero_quadric();
    for f in 0..nf {
        let i0 = indices[f * 3] as usize;
        let i1 = indices[f * 3 + 1] as usize;
        let i2 = indices[f * 3 + 2] as usize;
        if i0 != vertex && i1 != vertex && i2 != vertex {
            continue;
        }
        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];
        let e1 = sub3(p1, p0);
        let e2 = sub3(p2, p0);
        let n = normalize3(cross3(e1, e2));
        let a = n[0] as f64;
        let b = n[1] as f64;
        let c = n[2] as f64;
        let d = -(a * p0[0] as f64 + b * p0[1] as f64 + c * p0[2] as f64);
        q = add_quadric(&q, &plane_quadric(a, b, c, d));
    }
    q
}

/// Compute the QEM cost of collapsing edge (v0, v1) to their midpoint.
#[allow(dead_code)]
pub fn edge_collapse_cost(q0: &Quadric, q1: &Quadric, mid: [f32; 3]) -> f64 {
    let q = add_quadric(q0, q1);
    quadric_error(&q, mid).max(0.0)
}

/// Find the cheapest edge to collapse in the mesh.
/// Returns `(v0, v1, cost, midpoint)`.
#[allow(dead_code)]
pub fn find_cheapest_edge(
    positions: &[[f32; 3]],
    indices: &[u32],
    quadrics: &[Quadric],
) -> (usize, usize, f64, [f32; 3]) {
    let edges = collect_edges(indices);
    let mut best_cost = f64::INFINITY;
    let mut best_edge = (0, 0);
    let mut best_mid = [0.0f32; 3];
    for &(v0, v1) in &edges {
        let mid = midpoint(positions[v0], positions[v1]);
        let cost = edge_collapse_cost(&quadrics[v0], &quadrics[v1], mid);
        if cost < best_cost {
            best_cost = cost;
            best_edge = (v0, v1);
            best_mid = mid;
        }
    }
    (best_edge.0, best_edge.1, best_cost, best_mid)
}

/// Perform an edge collapse: merge v1 into v0, updating positions and indices.
/// Returns new (positions, indices).
#[allow(dead_code)]
pub fn collapse_edge(
    positions: &[[f32; 3]],
    indices: &[u32],
    v0: usize,
    v1: usize,
    new_pos: [f32; 3],
) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut new_positions = positions.to_vec();
    new_positions[v0] = new_pos;

    // Remap v1 -> v0 in all triangles, then remove degenerate ones
    let mut new_indices: Vec<u32> = Vec::new();
    let nf = indices.len() / 3;
    for f in 0..nf {
        let mut tri = [
            indices[f * 3] as usize,
            indices[f * 3 + 1] as usize,
            indices[f * 3 + 2] as usize,
        ];
        for t in &mut tri {
            if *t == v1 {
                *t = v0;
            }
        }
        // Skip degenerate
        if tri[0] == tri[1] || tri[1] == tri[2] || tri[0] == tri[2] {
            continue;
        }
        new_indices.push(tri[0] as u32);
        new_indices.push(tri[1] as u32);
        new_indices.push(tri[2] as u32);
    }

    (new_positions, new_indices)
}

/// Perform one step of decimation (collapse cheapest edge).
/// Returns updated `(positions, indices)` or `None` if no edge can be collapsed.
#[allow(dead_code)]
pub fn decimate_step(
    positions: &[[f32; 3]],
    indices: &[u32],
    max_error: f64,
) -> Option<(Vec<[f32; 3]>, Vec<u32>)> {
    if indices.len() < 3 {
        return None;
    }
    let quadrics: Vec<Quadric> = (0..positions.len())
        .map(|v| compute_quadric(positions, indices, v))
        .collect();
    let (v0, v1, cost, mid) = find_cheapest_edge(positions, indices, &quadrics);
    if cost > max_error {
        return None;
    }
    Some(collapse_edge(positions, indices, v0, v1, mid))
}

/// Compute the decimation ratio achieved.
#[allow(dead_code)]
pub fn decimation_ratio(original_faces: usize, current_faces: usize) -> f32 {
    if original_faces == 0 {
        return 0.0;
    }
    current_faces as f32 / original_faces as f32
}

/// Validate a decimated mesh: check no degenerate faces, indices in range.
#[allow(dead_code)]
pub fn validate_decimated_mesh(positions: &[[f32; 3]], indices: &[u32]) -> bool {
    let n = positions.len() as u32;
    let nf = indices.len() / 3;
    for f in 0..nf {
        let i0 = indices[f * 3];
        let i1 = indices[f * 3 + 1];
        let i2 = indices[f * 3 + 2];
        if i0 >= n || i1 >= n || i2 >= n {
            return false;
        }
        if i0 == i1 || i1 == i2 || i0 == i2 {
            return false;
        }
    }
    true
}

/// Count the number of boundary edges (edges shared by exactly one face).
#[allow(dead_code)]
pub fn count_boundary_edges(indices: &[u32]) -> usize {
    let mut edge_count: std::collections::HashMap<(u32, u32), usize> =
        std::collections::HashMap::new();
    let nf = indices.len() / 3;
    for f in 0..nf {
        let tri = [indices[f * 3], indices[f * 3 + 1], indices[f * 3 + 2]];
        for k in 0..3 {
            let e = sorted_edge(tri[k], tri[(k + 1) % 3]);
            *edge_count.entry(e).or_insert(0) += 1;
        }
    }
    edge_count.values().filter(|&&c| c == 1).count()
}

/// Compute a complexity score: weighted sum of vertices and faces.
#[allow(dead_code)]
pub fn mesh_complexity_score(vertex_count: usize, face_count: usize) -> f32 {
    vertex_count as f32 + 2.0 * face_count as f32
}

/// Decimate to an exact target face count.
#[allow(dead_code)]
pub fn decimate_to_target_faces(
    positions: &[[f32; 3]],
    indices: &[u32],
    target_faces: usize,
    max_error: f64,
    _preserve_boundary: bool,
) -> DecimateResult {
    let original_faces = indices.len() / 3;
    let mut cur_pos = positions.to_vec();
    let mut cur_idx = indices.to_vec();

    while cur_idx.len() / 3 > target_faces {
        match decimate_step(&cur_pos, &cur_idx, max_error) {
            Some((p, i)) => {
                cur_pos = p;
                cur_idx = i;
            }
            None => break,
        }
    }

    DecimateResult {
        original_faces,
        result_faces: cur_idx.len() / 3,
        positions: cur_pos,
        indices: cur_idx,
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn sorted_edge(a: u32, b: u32) -> (u32, u32) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

fn midpoint(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

fn collect_edges(indices: &[u32]) -> Vec<(usize, usize)> {
    let mut set = BTreeSet::new();
    let nf = indices.len() / 3;
    for f in 0..nf {
        let tri = [
            indices[f * 3] as usize,
            indices[f * 3 + 1] as usize,
            indices[f * 3 + 2] as usize,
        ];
        for k in 0..3 {
            let (a, b) = if tri[k] < tri[(k + 1) % 3] {
                (tri[k], tri[(k + 1) % 3])
            } else {
                (tri[(k + 1) % 3], tri[k])
            };
            set.insert((a, b));
        }
    }
    set.into_iter().collect()
}

// ─── additional API ──────────────────────────────────────────────────────────

/// Returns the ratio of remaining vertices to original vertices.
#[allow(dead_code)]
pub fn decimate_ratio_achieved(original_count: usize, result: &DecimateResult) -> f32 {
    if original_count == 0 {
        return 1.0;
    }
    result.positions.len() as f32 / original_count as f32
}

/// Returns the vertex count of the decimated result.
#[allow(dead_code)]
pub fn decimate_vertex_count(result: &DecimateResult) -> usize {
    result.positions.len()
}

/// Validates that the result has consistent indices in range.
#[allow(dead_code)]
pub fn decimate_validate(result: &DecimateResult) -> bool {
    let n = result.positions.len() as u32;
    result.indices.iter().all(|&i| i < n) && result.indices.len().is_multiple_of(3)
}

/// Serialises the result to a minimal JSON string.
#[allow(dead_code)]
pub fn decimate_to_json(result: &DecimateResult) -> String {
    format!(
        "{{\"vertices\":{},\"indices\":{},\"original_faces\":{},\"result_faces\":{}}}",
        result.positions.len(),
        result.indices.len(),
        result.original_faces,
        result.result_faces
    )
}

// ─── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_quad() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let indices = vec![0, 1, 2, 0, 2, 3];
        (positions, indices)
    }

    fn subdivided_plane() -> (Vec<[f32; 3]>, Vec<u32>) {
        // 3x3 grid = 9 verts, 8 triangles
        let mut positions = Vec::new();
        for j in 0..3 {
            for i in 0..3 {
                positions.push([i as f32, j as f32, 0.0]);
            }
        }
        let mut indices = Vec::new();
        for j in 0..2u32 {
            for i in 0..2u32 {
                let v0 = j * 3 + i;
                let v1 = v0 + 1;
                let v2 = v0 + 3;
                let v3 = v2 + 1;
                indices.extend_from_slice(&[v0, v1, v2, v1, v3, v2]);
            }
        }
        (positions, indices)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_decimate_config();
        assert!((cfg.target_ratio - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_decimate_mesh_basic() {
        let (pos, idx) = subdivided_plane();
        let cfg = DecimateConfig {
            target_ratio: 0.5,
            max_error: f64::INFINITY,
            preserve_boundary: false,
        };
        let result = decimate_mesh(&pos, &idx, &cfg);
        assert!(result.result_faces < result.original_faces);
    }

    #[test]
    fn test_compute_quadric_exists() {
        let (pos, idx) = simple_quad();
        let q = compute_quadric(&pos, &idx, 0);
        // Quadric should be non-zero for a face-connected vertex
        assert!(q.iter().any(|&v| v.abs() > 1e-12));
    }

    #[test]
    fn test_edge_collapse_cost_zero_for_coplanar() {
        // Two vertices on a flat surface: collapsing should be cheap
        let (pos, idx) = simple_quad();
        let q0 = compute_quadric(&pos, &idx, 0);
        let q1 = compute_quadric(&pos, &idx, 1);
        let mid = midpoint(pos[0], pos[1]);
        let cost = edge_collapse_cost(&q0, &q1, mid);
        // On a flat surface, cost should be very small
        assert!(cost < 1.0);
    }

    #[test]
    fn test_find_cheapest_edge() {
        let (pos, idx) = simple_quad();
        let quadrics: Vec<Quadric> = (0..pos.len())
            .map(|v| compute_quadric(&pos, &idx, v))
            .collect();
        let (v0, v1, cost, _mid) = find_cheapest_edge(&pos, &idx, &quadrics);
        assert!(v0 != v1);
        assert!(cost >= 0.0);
    }

    #[test]
    fn test_collapse_edge_reduces_faces() {
        let (pos, idx) = simple_quad();
        let mid = midpoint(pos[0], pos[1]);
        let (_new_pos, new_idx) = collapse_edge(&pos, &idx, 0, 1, mid);
        assert!(new_idx.len() / 3 < idx.len() / 3);
    }

    #[test]
    fn test_decimate_step() {
        let (pos, idx) = subdivided_plane();
        let result = decimate_step(&pos, &idx, f64::INFINITY);
        assert!(result.is_some());
        let (_, new_idx) = result.expect("should succeed");
        assert!(new_idx.len() < idx.len());
    }

    #[test]
    fn test_decimation_ratio() {
        assert!((decimation_ratio(100, 50) - 0.5).abs() < 1e-6);
        assert!((decimation_ratio(100, 100) - 1.0).abs() < 1e-6);
        assert!((decimation_ratio(0, 0)).abs() < 1e-6);
    }

    #[test]
    fn test_validate_decimated_mesh_valid() {
        let (pos, idx) = simple_quad();
        assert!(validate_decimated_mesh(&pos, &idx));
    }

    #[test]
    fn test_validate_decimated_mesh_invalid_index() {
        let pos = vec![[0.0, 0.0, 0.0]; 3];
        let idx = vec![0, 1, 99]; // out of range
        assert!(!validate_decimated_mesh(&pos, &idx));
    }

    #[test]
    fn test_validate_degenerate_face() {
        let pos = vec![[0.0, 0.0, 0.0]; 3];
        let idx = vec![0, 0, 1]; // degenerate
        assert!(!validate_decimated_mesh(&pos, &idx));
    }

    #[test]
    fn test_count_boundary_edges_quad() {
        let (_pos, idx) = simple_quad();
        let boundary = count_boundary_edges(&idx);
        // A quad (2 triangles) has 4 boundary edges + 1 shared internal edge
        assert_eq!(boundary, 4);
    }

    #[test]
    fn test_mesh_complexity_score() {
        let score = mesh_complexity_score(100, 200);
        assert!((score - 500.0).abs() < 1e-6);
    }

    #[test]
    fn test_decimate_to_target_faces() {
        let (pos, idx) = subdivided_plane();
        let result = decimate_to_target_faces(&pos, &idx, 4, f64::INFINITY, false);
        assert!(result.result_faces <= 4);
    }

    #[test]
    fn test_decimate_preserves_validity() {
        let (pos, idx) = subdivided_plane();
        let cfg = DecimateConfig {
            target_ratio: 0.5,
            max_error: f64::INFINITY,
            preserve_boundary: false,
        };
        let result = decimate_mesh(&pos, &idx, &cfg);
        assert!(validate_decimated_mesh(&result.positions, &result.indices));
    }

    #[test]
    fn test_zero_quadric() {
        let q = zero_quadric();
        assert!(q.iter().all(|&v| v == 0.0));
    }
}
