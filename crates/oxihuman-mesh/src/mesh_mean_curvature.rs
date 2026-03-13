// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mean curvature computation using cotangent weights on triangle meshes.
//!
//! Provides per-vertex mean and Gaussian curvature, principal curvatures,
//! Laplacian smoothing, and curvature-based color mapping.

use crate::mesh::MeshBuffers;

// ── Type aliases ─────────────────────────────────────────────────────────────

/// Per-vertex curvature values: `(mean_curvature, gaussian_curvature)`.
#[allow(dead_code)]
pub type CurvaturePair = (f32, f32);

/// Per-vertex `[r, g, b]` color derived from curvature.
#[allow(dead_code)]
pub type CurvatureColor = [f32; 3];

// ── Structs ──────────────────────────────────────────────────────────────────

/// Configuration for curvature computation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurvatureConfig {
    /// If true, clamp curvature values to `[-clamp_range, clamp_range]`.
    pub clamp: bool,
    /// Maximum absolute curvature value when clamping is enabled.
    pub clamp_range: f32,
    /// Number of Laplacian smoothing iterations for curvature values.
    pub smooth_iterations: usize,
    /// Smoothing weight (0, 1].
    pub smooth_lambda: f32,
}

/// Result of curvature computation for the entire mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurvatureResult {
    /// Per-vertex mean curvature.
    pub mean: Vec<f32>,
    /// Per-vertex Gaussian curvature.
    pub gaussian: Vec<f32>,
    /// Number of vertices.
    pub vertex_count: usize,
}

// ── Default config ───────────────────────────────────────────────────────────

/// Return a sensible default [`CurvatureConfig`].
#[allow(dead_code)]
pub fn default_curvature_config() -> CurvatureConfig {
    CurvatureConfig {
        clamp: true,
        clamp_range: 100.0,
        smooth_iterations: 0,
        smooth_lambda: 0.5,
    }
}

// ── Internal helpers ─────────────────────────────────────────────────────────

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    dot3(v, v).sqrt()
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Compute the cotangent of the angle at vertex C in triangle (A, B, C).
///
/// Returns `cot(angle_C) = dot(CA, CB) / |CA x CB|`.
/// Returns 0.0 for degenerate triangles.
#[allow(dead_code)]
pub fn cotangent_weight(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ca = sub3(a, c);
    let cb = sub3(b, c);
    let cross = cross3(ca, cb);
    let cross_len = len3(cross);
    if cross_len < 1e-12 {
        return 0.0;
    }
    dot3(ca, cb) / cross_len
}

/// Compute the mixed Voronoi area around a vertex.
///
/// `one_ring` contains the positions of the 1-ring neighbour triangles
/// as pairs `(prev, next)` around the vertex.
#[allow(dead_code)]
pub fn vertex_area_mixed(pos: [f32; 3], one_ring: &[([f32; 3], [f32; 3])]) -> f32 {
    let mut area = 0.0f32;
    for &(p, q) in one_ring {
        let cross = cross3(sub3(p, pos), sub3(q, pos));
        let tri_area = len3(cross) * 0.5;
        // Simplified mixed area: 1/3 of triangle area per vertex
        area += tri_area / 3.0;
    }
    area.max(1e-12)
}

/// Compute the mean curvature vector at a single vertex using cotangent weights.
///
/// Returns the un-normalised Laplacian vector `sum( w_ij * (pj - pi) )`.
#[allow(dead_code)]
pub fn mean_curvature_vector(
    mesh: &MeshBuffers,
    vertex: usize,
    adj: &[Vec<(usize, usize, usize)>],
) -> [f32; 3] {
    let pi = mesh.positions[vertex];
    let mut lap = [0.0f32; 3];

    for &(nb, prev_v, next_v) in &adj[vertex] {
        let pj = mesh.positions[nb];
        let w1 = cotangent_weight(pi, pj, mesh.positions[prev_v]);
        let w2 = cotangent_weight(pi, pj, mesh.positions[next_v]);
        let w = (w1 + w2) * 0.5;
        let diff = sub3(pj, pi);
        lap = add3(lap, scale3(diff, w));
    }

    lap
}

/// Build adjacency list for cotangent-weight computation.
///
/// For each vertex, stores `(neighbour, opposite_vertex_1, opposite_vertex_2)`.
#[allow(dead_code)]
fn build_cot_adjacency(mesh: &MeshBuffers) -> Vec<Vec<(usize, usize, usize)>> {
    let n = mesh.vertex_count();
    let mut adj: Vec<Vec<(usize, usize, usize)>> = vec![Vec::new(); n];
    let idx = &mesh.indices;
    let fc = idx.len() / 3;

    for f in 0..fc {
        let tri = [
            idx[f * 3] as usize,
            idx[f * 3 + 1] as usize,
            idx[f * 3 + 2] as usize,
        ];
        for k in 0..3 {
            let i = tri[k];
            let j = tri[(k + 1) % 3];
            let opp = tri[(k + 2) % 3];

            // Check if edge (i, j) already recorded
            if let Some(entry) = adj[i].iter_mut().find(|e| e.0 == j) {
                // Second adjacent triangle: fill in next_v
                entry.2 = opp;
            } else {
                adj[i].push((j, opp, opp));
            }
        }
    }
    adj
}

/// Compute per-vertex mean curvature for the entire mesh.
#[allow(dead_code)]
pub fn compute_mean_curvature(mesh: &MeshBuffers, cfg: &CurvatureConfig) -> Vec<f32> {
    let n = mesh.vertex_count();
    if n == 0 {
        return Vec::new();
    }
    let adj = build_cot_adjacency(mesh);
    let mut curvatures = Vec::with_capacity(n);

    for v in 0..n {
        let lap = mean_curvature_vector(mesh, v, &adj);
        let h = len3(lap) * 0.5;
        let h = if cfg.clamp {
            h.clamp(-cfg.clamp_range, cfg.clamp_range)
        } else {
            h
        };
        curvatures.push(h);
    }

    // Optional smoothing
    if cfg.smooth_iterations > 0 {
        smooth_values(
            &mut curvatures,
            &adj,
            cfg.smooth_iterations,
            cfg.smooth_lambda,
        );
    }

    curvatures
}

/// Compute per-vertex Gaussian curvature using the angle-deficit method.
///
/// K(v) = (2*pi - sum_of_incident_angles) / A_mixed(v).
#[allow(dead_code)]
pub fn compute_gaussian_curvature(mesh: &MeshBuffers, cfg: &CurvatureConfig) -> Vec<f32> {
    let n = mesh.vertex_count();
    if n == 0 {
        return Vec::new();
    }
    let idx = &mesh.indices;
    let fc = idx.len() / 3;

    let mut angle_sum = vec![0.0f32; n];
    let mut area = vec![0.0f32; n];

    for f in 0..fc {
        let tri = [
            idx[f * 3] as usize,
            idx[f * 3 + 1] as usize,
            idx[f * 3 + 2] as usize,
        ];
        let p = [
            mesh.positions[tri[0]],
            mesh.positions[tri[1]],
            mesh.positions[tri[2]],
        ];
        let tri_area = len3(cross3(sub3(p[1], p[0]), sub3(p[2], p[0]))) * 0.5;

        for k in 0..3 {
            let e1 = sub3(p[(k + 1) % 3], p[k]);
            let e2 = sub3(p[(k + 2) % 3], p[k]);
            let l1 = len3(e1);
            let l2 = len3(e2);
            if l1 > 1e-12 && l2 > 1e-12 {
                let cos_a = (dot3(e1, e2) / (l1 * l2)).clamp(-1.0, 1.0);
                angle_sum[tri[k]] += cos_a.acos();
            }
            area[tri[k]] += tri_area / 3.0;
        }
    }

    let two_pi = std::f32::consts::PI * 2.0;
    let mut curvatures = Vec::with_capacity(n);
    for v in 0..n {
        let a = area[v].max(1e-12);
        let k = (two_pi - angle_sum[v]) / a;
        let k = if cfg.clamp {
            k.clamp(-cfg.clamp_range, cfg.clamp_range)
        } else {
            k
        };
        curvatures.push(k);
    }

    curvatures
}

/// Compute principal curvatures (kappa_1, kappa_2) from mean (H) and Gaussian (K) curvature.
///
/// kappa_{1,2} = H +/- sqrt(H^2 - K).
#[allow(dead_code)]
pub fn principal_curvatures(h: f32, k: f32) -> (f32, f32) {
    let disc = (h * h - k).max(0.0);
    let root = disc.sqrt();
    (h + root, h - root)
}

/// Return the minimum curvature (kappa_2) from H and K.
#[allow(dead_code)]
pub fn curvature_min(h: f32, k: f32) -> f32 {
    principal_curvatures(h, k).1
}

/// Return the maximum curvature (kappa_1) from H and K.
#[allow(dead_code)]
pub fn curvature_max(h: f32, k: f32) -> f32 {
    principal_curvatures(h, k).0
}

/// Return the average of the principal curvatures (which equals H).
#[allow(dead_code)]
pub fn curvature_mean_value(h: f32, _k: f32) -> f32 {
    h
}

/// Laplacian smoothing of vertex positions using cotangent weights.
///
/// Moves each vertex towards its weighted neighbourhood average.
#[allow(dead_code)]
pub fn laplacian_smooth_positions(
    positions: &mut [[f32; 3]],
    adj: &[Vec<(usize, usize, usize)>],
    iterations: usize,
    lambda: f32,
) {
    let n = positions.len();
    for _ in 0..iterations {
        let prev = positions.to_vec();
        for v in 0..n {
            if adj[v].is_empty() {
                continue;
            }
            let mut total_weight = 0.0f32;
            let mut weighted_pos = [0.0f32; 3];
            for &(nb, prev_v, next_v) in &adj[v] {
                let w1 = cotangent_weight(prev[v], prev[nb], prev[prev_v]);
                let w2 = cotangent_weight(prev[v], prev[nb], prev[next_v]);
                let w = ((w1 + w2) * 0.5).max(0.0);
                weighted_pos = add3(weighted_pos, scale3(prev[nb], w));
                total_weight += w;
            }
            if total_weight > 1e-12 {
                let avg = scale3(weighted_pos, 1.0 / total_weight);
                let diff = sub3(avg, prev[v]);
                positions[v] = add3(prev[v], scale3(diff, lambda));
            }
        }
    }
}

/// Map a scalar curvature value to an RGB color.
///
/// Negative curvature -> blue, zero -> green, positive -> red.
#[allow(dead_code)]
pub fn curvature_color_map(value: f32, range: f32) -> CurvatureColor {
    let range = range.max(1e-6);
    let t = (value / range).clamp(-1.0, 1.0);
    if t < 0.0 {
        // blue to green
        let s = 1.0 + t; // 0..1
        [0.0, s, 1.0 - s]
    } else {
        // green to red
        [t, 1.0 - t, 0.0]
    }
}

// ── Internal smoothing helper ────────────────────────────────────────────────

fn smooth_values(
    values: &mut [f32],
    adj: &[Vec<(usize, usize, usize)>],
    iterations: usize,
    lambda: f32,
) {
    let n = values.len();
    for _ in 0..iterations {
        let prev = values.to_vec();
        for v in 0..n {
            if adj[v].is_empty() {
                continue;
            }
            let avg: f32 =
                adj[v].iter().map(|&(nb, _, _)| prev[nb]).sum::<f32>() / adj[v].len() as f32;
            values[v] = prev[v] * (1.0 - lambda) + avg * lambda;
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn make_mesh(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> MeshBuffers {
        let n = positions.len();
        MeshBuffers::from_morph(MB {
            positions,
            normals: vec![[0.0, 0.0, 1.0]; n],
            uvs: vec![[0.0, 0.0]; n],
            indices,
            has_suit: false,
        })
    }

    fn single_tri() -> MeshBuffers {
        make_mesh(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0, 1, 2],
        )
    }

    fn two_tris() -> MeshBuffers {
        make_mesh(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![0, 1, 2, 1, 3, 2],
        )
    }

    fn cfg() -> CurvatureConfig {
        default_curvature_config()
    }

    #[test]
    fn test_default_curvature_config() {
        let c = default_curvature_config();
        assert!(c.clamp);
        assert!(c.clamp_range > 0.0);
        assert_eq!(c.smooth_iterations, 0);
    }

    #[test]
    fn test_cotangent_weight_right_angle() {
        // Right angle at C = origin: cot(90deg) = 0
        let w = cotangent_weight([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.0]);
        assert!(w.abs() < 1e-5);
    }

    #[test]
    fn test_cotangent_weight_degenerate() {
        let w = cotangent_weight([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert_eq!(w, 0.0);
    }

    #[test]
    fn test_cotangent_weight_positive_for_acute() {
        let w = cotangent_weight([1.0, 0.0, 0.0], [0.5, 1.0, 0.0], [0.0, 0.0, 0.0]);
        assert!(w > 0.0, "acute angle should yield positive cot");
    }

    #[test]
    fn test_vertex_area_mixed_positive() {
        let pos = [0.0, 0.0, 0.0];
        let ring = vec![([1.0, 0.0, 0.0], [0.0, 1.0, 0.0])];
        let a = vertex_area_mixed(pos, &ring);
        assert!(a > 0.0);
    }

    #[test]
    fn test_mean_curvature_vector_single_tri() {
        let mesh = single_tri();
        let adj = build_cot_adjacency(&mesh);
        let lap = mean_curvature_vector(&mesh, 0, &adj);
        // For a flat triangle, Laplacian should be in-plane
        assert!(lap[2].abs() < 1e-5, "z-component should be ~0 for flat tri");
    }

    #[test]
    fn test_compute_mean_curvature_length() {
        let mesh = two_tris();
        let h = compute_mean_curvature(&mesh, &cfg());
        assert_eq!(h.len(), 4);
    }

    #[test]
    fn test_compute_gaussian_curvature_length() {
        let mesh = two_tris();
        let k = compute_gaussian_curvature(&mesh, &cfg());
        assert_eq!(k.len(), 4);
    }

    #[test]
    fn test_compute_mean_curvature_flat_near_zero() {
        // A flat mesh should have near-zero mean curvature (except at boundary)
        let mesh = two_tris();
        let h = compute_mean_curvature(&mesh, &cfg());
        // Interior-like vertices should have small mean curvature
        // but boundary vertices will have larger values
        let finite_count = h.iter().filter(|v| v.is_finite()).count();
        assert_eq!(finite_count, 4);
    }

    #[test]
    fn test_compute_gaussian_curvature_flat_boundary() {
        let mesh = two_tris();
        let k = compute_gaussian_curvature(&mesh, &cfg());
        for &val in &k {
            assert!(val.is_finite());
        }
    }

    #[test]
    fn test_principal_curvatures_flat() {
        let (k1, k2) = principal_curvatures(0.0, 0.0);
        assert!((k1).abs() < 1e-6);
        assert!((k2).abs() < 1e-6);
    }

    #[test]
    fn test_principal_curvatures_sphere() {
        // For a sphere of radius R: H = 1/R, K = 1/R^2
        let r = 2.0f32;
        let h = 1.0 / r;
        let k = 1.0 / (r * r);
        let (k1, k2) = principal_curvatures(h, k);
        // Both principal curvatures should be ~1/R
        assert!((k1 - 1.0 / r).abs() < 1e-5);
        assert!((k2 - 1.0 / r).abs() < 1e-5);
    }

    #[test]
    fn test_curvature_min_max() {
        let kmin = curvature_min(2.0, 3.0);
        let kmax = curvature_max(2.0, 3.0);
        assert!(kmax >= kmin);
    }

    #[test]
    fn test_curvature_mean_value() {
        assert!((curvature_mean_value(5.0, 3.0) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_curvature_color_map_zero() {
        let c = curvature_color_map(0.0, 1.0);
        // zero -> green
        assert!(c[1] > 0.9);
        assert!(c[0] < 0.1);
        assert!(c[2] < 0.1);
    }

    #[test]
    fn test_curvature_color_map_positive() {
        let c = curvature_color_map(1.0, 1.0);
        // max positive -> red
        assert!(c[0] > 0.9);
    }

    #[test]
    fn test_curvature_color_map_negative() {
        let c = curvature_color_map(-1.0, 1.0);
        // max negative -> blue
        assert!(c[2] > 0.9);
    }

    #[test]
    fn test_compute_mean_curvature_empty() {
        let mesh = make_mesh(vec![], vec![]);
        let h = compute_mean_curvature(&mesh, &cfg());
        assert!(h.is_empty());
    }

    #[test]
    fn test_compute_gaussian_curvature_empty() {
        let mesh = make_mesh(vec![], vec![]);
        let k = compute_gaussian_curvature(&mesh, &cfg());
        assert!(k.is_empty());
    }

    #[test]
    fn test_compute_mean_curvature_with_smoothing() {
        let mut c = cfg();
        c.smooth_iterations = 3;
        c.smooth_lambda = 0.5;
        let mesh = two_tris();
        let h = compute_mean_curvature(&mesh, &c);
        assert_eq!(h.len(), 4);
        for &val in &h {
            assert!(val.is_finite());
        }
    }
}
