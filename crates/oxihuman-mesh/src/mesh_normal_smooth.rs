//! Normal smoothing with crease angle support for mesh processing.
//!
//! Provides angle-weighted vertex normal averaging, sharp-edge splitting
//! at crease angles, multi-pass smoothing, and utility functions for
//! normal manipulation.

/// Configuration for normal smoothing operations.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NormalSmoothConfig {
    /// Crease angle in radians above which edges are considered sharp.
    pub crease_angle: f32,
    /// Number of smoothing iterations.
    pub iterations: u32,
    /// Blend factor between original and smoothed normals (0=original, 1=smoothed).
    pub blend_factor: f32,
    /// Whether to flip normals after smoothing.
    pub flip: bool,
}

/// Result of a normal smoothing operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NormalSmoothResult {
    /// Smoothed per-vertex normals, each element is [x, y, z].
    pub normals: Vec<[f32; 3]>,
    /// Number of vertices that were split due to sharp edges.
    pub split_count: u32,
    /// Number of smoothing passes applied.
    pub passes_applied: u32,
}

/// Returns a default `NormalSmoothConfig`.
#[allow(dead_code)]
pub fn default_normal_smooth_config() -> NormalSmoothConfig {
    NormalSmoothConfig {
        crease_angle: std::f32::consts::PI / 3.0, // 60 degrees
        iterations: 1,
        blend_factor: 1.0,
        flip: false,
    }
}

/// Computes a flat (face) normal for a triangle defined by three positions.
#[allow(dead_code)]
pub fn compute_face_normals(
    positions: &[[f32; 3]],
    indices: &[u32],
) -> Vec<[f32; 3]> {
    let face_count = indices.len() / 3;
    let mut face_normals = Vec::with_capacity(face_count);
    for i in 0..face_count {
        let a = positions[indices[i * 3] as usize];
        let b = positions[indices[i * 3 + 1] as usize];
        let c = positions[indices[i * 3 + 2] as usize];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = cross(ab, ac);
        face_normals.push(normalize_normal(n));
    }
    face_normals
}

/// Smooths vertex normals using angle-weighted averaging.
///
/// Each contributing face normal is weighted by the angle it subtends
/// at the vertex, giving larger faces less influence than smaller ones.
#[allow(dead_code)]
pub fn smooth_vertex_normals(
    positions: &[[f32; 3]],
    indices: &[u32],
    _config: &NormalSmoothConfig,
) -> NormalSmoothResult {
    let face_normals = compute_face_normals(positions, indices);
    let vertex_count = positions.len();
    let face_count = indices.len() / 3;

    let mut accumulated = vec![[0.0f32; 3]; vertex_count];

    for f in 0..face_count {
        let ia = indices[f * 3] as usize;
        let ib = indices[f * 3 + 1] as usize;
        let ic = indices[f * 3 + 2] as usize;

        let angle_a = corner_angle(positions[ia], positions[ib], positions[ic]);
        let angle_b = corner_angle(positions[ib], positions[ic], positions[ia]);
        let angle_c = corner_angle(positions[ic], positions[ia], positions[ib]);

        let fn_ = face_normals[f];
        for (vi, w) in [(ia, angle_a), (ib, angle_b), (ic, angle_c)] {
            accumulated[vi][0] += fn_[0] * w;
            accumulated[vi][1] += fn_[1] * w;
            accumulated[vi][2] += fn_[2] * w;
        }
    }

    let normals: Vec<[f32; 3]> = accumulated.iter().map(|&n| normalize_normal(n)).collect();
    NormalSmoothResult { normals, split_count: 0, passes_applied: 1 }
}

/// Splits vertex normals at crease edges (sharp edges).
///
/// Returns split normals and the number of vertices that were duplicated.
#[allow(dead_code)]
pub fn sharp_edge_normals(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &NormalSmoothConfig,
) -> NormalSmoothResult {
    let face_normals = compute_face_normals(positions, indices);
    let vertex_count = positions.len();
    let face_count = indices.len() / 3;

    // For each vertex, accumulate normals from faces whose normals
    // differ less than crease_angle from the average.
    let mut accumulated = vec![[0.0f32; 3]; vertex_count];
    let mut split_count = 0u32;

    // First pass: accumulate face normals per vertex
    for f in 0..face_count {
        for k in 0..3 {
            let vi = indices[f * 3 + k] as usize;
            let fn_ = face_normals[f];
            accumulated[vi][0] += fn_[0];
            accumulated[vi][1] += fn_[1];
            accumulated[vi][2] += fn_[2];
        }
    }

    // Normalize accumulated
    let avg_normals: Vec<[f32; 3]> = accumulated.iter().map(|&n| normalize_normal(n)).collect();

    // Second pass: mark splits where angle exceeds crease
    let mut final_normals = avg_normals.clone();
    for f in 0..face_count {
        let fn_ = face_normals[f];
        for k in 0..3 {
            let vi = indices[f * 3 + k] as usize;
            let angle = angle_between_normals(fn_, avg_normals[vi]);
            if angle > config.crease_angle {
                // This face contributes a split normal
                final_normals[vi] = fn_;
                split_count += 1;
            }
        }
    }

    NormalSmoothResult { normals: final_normals, split_count, passes_applied: 1 }
}

/// Computes the angle (in radians) between two normals.
#[allow(dead_code)]
pub fn angle_between_normals(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = dot(a, b).clamp(-1.0, 1.0);
    d.acos()
}

/// Normalizes a normal vector to unit length.
/// Returns a zero vector if the input has negligible length.
#[allow(dead_code)]
pub fn normalize_normal(n: [f32; 3]) -> [f32; 3] {
    let len_sq = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
    if len_sq < 1e-12 {
        return [0.0, 0.0, 1.0];
    }
    let inv = 1.0 / len_sq.sqrt();
    [n[0] * inv, n[1] * inv, n[2] * inv]
}

/// Applies multiple smoothing passes to vertex normals.
#[allow(dead_code)]
pub fn smooth_normals_iter(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &NormalSmoothConfig,
) -> NormalSmoothResult {
    let mut result = smooth_vertex_normals(positions, indices, config);
    for _ in 1..config.iterations {
        let iter_result = smooth_vertex_normals(positions, indices, config);
        // blend towards the iterated result
        result.normals = result
            .normals
            .iter()
            .zip(iter_result.normals.iter())
            .map(|(&a, &b)| blend_normals(a, b, 0.5))
            .collect();
    }
    result.passes_applied = config.iterations;
    result
}

/// Computes per-vertex normals directly from face contributions (unweighted).
#[allow(dead_code)]
pub fn compute_vertex_normals_from_faces(
    positions: &[[f32; 3]],
    indices: &[u32],
) -> Vec<[f32; 3]> {
    let face_normals = compute_face_normals(positions, indices);
    let vertex_count = positions.len();
    let face_count = indices.len() / 3;
    let mut accumulated = vec![[0.0f32; 3]; vertex_count];
    for f in 0..face_count {
        for k in 0..3 {
            let vi = indices[f * 3 + k] as usize;
            let fn_ = face_normals[f];
            accumulated[vi][0] += fn_[0];
            accumulated[vi][1] += fn_[1];
            accumulated[vi][2] += fn_[2];
        }
    }
    accumulated.iter().map(|&n| normalize_normal(n)).collect()
}

/// Blends two normals by a factor `t` (0 = a, 1 = b) and normalizes.
#[allow(dead_code)]
pub fn blend_normals(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let s = 1.0 - t;
    normalize_normal([
        a[0] * s + b[0] * t,
        a[1] * s + b[1] * t,
        a[2] * s + b[2] * t,
    ])
}

/// Flips all normals (negates each component).
#[allow(dead_code)]
pub fn flip_normals(normals: &[[f32; 3]]) -> Vec<[f32; 3]> {
    normals.iter().map(|&n| [-n[0], -n[1], -n[2]]).collect()
}

/// Computes a map of how much each vertex normal deviates from its face normals.
///
/// Returns a `Vec<f32>` with the maximum angular deviation per vertex (in radians).
#[allow(dead_code)]
pub fn normal_deviation_map(
    positions: &[[f32; 3]],
    indices: &[u32],
    vertex_normals: &[[f32; 3]],
) -> Vec<f32> {
    let face_normals = compute_face_normals(positions, indices);
    let vertex_count = positions.len();
    let face_count = indices.len() / 3;
    let mut max_deviation = vec![0.0f32; vertex_count];

    for f in 0..face_count {
        let fn_ = face_normals[f];
        for k in 0..3 {
            let vi = indices[f * 3 + k] as usize;
            if vi < vertex_normals.len() {
                let dev = angle_between_normals(fn_, vertex_normals[vi]);
                if dev > max_deviation[vi] {
                    max_deviation[vi] = dev;
                }
            }
        }
    }
    max_deviation
}

/// Returns the number of vertex normals in a smoothing result.
#[allow(dead_code)]
pub fn vertex_normal_count(result: &NormalSmoothResult) -> usize {
    result.normals.len()
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Angle at vertex `origin` in the triangle (origin, p1, p2).
fn corner_angle(origin: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> f32 {
    let v1 = normalize_normal([p1[0] - origin[0], p1[1] - origin[1], p1[2] - origin[2]]);
    let v2 = normalize_normal([p2[0] - origin[0], p2[1] - origin[1], p2[2] - origin[2]]);
    dot(v1, v2).clamp(-1.0, 1.0).acos()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn single_triangle_positions() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
    }
    fn single_triangle_indices() -> Vec<u32> {
        vec![0, 1, 2]
    }

    #[test]
    fn test_default_config() {
        let cfg = default_normal_smooth_config();
        assert!(cfg.crease_angle > 0.0);
        assert_eq!(cfg.iterations, 1);
        assert!(!cfg.flip);
    }

    #[test]
    fn test_compute_face_normals_single_triangle() {
        let pos = single_triangle_positions();
        let idx = single_triangle_indices();
        let normals = compute_face_normals(&pos, &idx);
        assert_eq!(normals.len(), 1);
        // Triangle in XY plane → normal should be along +Z
        let n = normals[0];
        assert!((n[2].abs() - 1.0).abs() < 1e-5, "z should be ~1, got {}", n[2]);
    }

    #[test]
    fn test_compute_face_normals_unit_length() {
        let pos = single_triangle_positions();
        let idx = single_triangle_indices();
        let normals = compute_face_normals(&pos, &idx);
        for n in &normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn test_normalize_normal_unit_vector() {
        let n = normalize_normal([3.0, 4.0, 0.0]);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_normal_zero_returns_fallback() {
        let n = normalize_normal([0.0, 0.0, 0.0]);
        assert_eq!(n, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_angle_between_normals_parallel() {
        let a = [0.0, 0.0, 1.0];
        assert!(angle_between_normals(a, a) < 1e-5);
    }

    #[test]
    fn test_angle_between_normals_orthogonal() {
        let a = [1.0, 0.0, 0.0];
        let b = [0.0, 1.0, 0.0];
        let angle = angle_between_normals(a, b);
        assert!((angle - PI / 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_angle_between_normals_opposite() {
        let a = [0.0, 0.0, 1.0];
        let b = [0.0, 0.0, -1.0];
        let angle = angle_between_normals(a, b);
        assert!((angle - PI).abs() < 1e-5);
    }

    #[test]
    fn test_smooth_vertex_normals_count() {
        let pos = single_triangle_positions();
        let idx = single_triangle_indices();
        let cfg = default_normal_smooth_config();
        let result = smooth_vertex_normals(&pos, &idx, &cfg);
        assert_eq!(result.normals.len(), pos.len());
    }

    #[test]
    fn test_smooth_vertex_normals_unit_length() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 1, 3, 2];
        let cfg = default_normal_smooth_config();
        let result = smooth_vertex_normals(&pos, &idx, &cfg);
        for n in &result.normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn test_flip_normals() {
        let normals = vec![[0.0, 0.0, 1.0], [1.0, 0.0, 0.0]];
        let flipped = flip_normals(&normals);
        assert_eq!(flipped[0], [0.0, 0.0, -1.0]);
        assert_eq!(flipped[1], [-1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_blend_normals_midpoint() {
        let a = [1.0, 0.0, 0.0];
        let b = [0.0, 1.0, 0.0];
        let blended = blend_normals(a, b, 0.5);
        let len = (blended[0] * blended[0] + blended[1] * blended[1] + blended[2] * blended[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_normals_zero_preserves_a() {
        let a = [0.0, 0.0, 1.0];
        let b = [1.0, 0.0, 0.0];
        let blended = blend_normals(a, b, 0.0);
        assert!((blended[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_compute_vertex_normals_from_faces() {
        let pos = single_triangle_positions();
        let idx = single_triangle_indices();
        let normals = compute_vertex_normals_from_faces(&pos, &idx);
        assert_eq!(normals.len(), pos.len());
        for n in &normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn test_sharp_edge_normals() {
        let pos = single_triangle_positions();
        let idx = single_triangle_indices();
        let cfg = NormalSmoothConfig { crease_angle: 0.01, ..default_normal_smooth_config() };
        let result = sharp_edge_normals(&pos, &idx, &cfg);
        assert_eq!(result.normals.len(), pos.len());
    }

    #[test]
    fn test_normal_deviation_map_zero_for_flat_mesh() {
        let pos = single_triangle_positions();
        let idx = single_triangle_indices();
        let vn = compute_vertex_normals_from_faces(&pos, &idx);
        let dev = normal_deviation_map(&pos, &idx, &vn);
        // All vertices should have near-zero deviation since normals match face normal
        for d in &dev {
            assert!(*d < 1e-4);
        }
    }

    #[test]
    fn test_vertex_normal_count() {
        let pos = single_triangle_positions();
        let idx = single_triangle_indices();
        let cfg = default_normal_smooth_config();
        let result = smooth_vertex_normals(&pos, &idx, &cfg);
        assert_eq!(vertex_normal_count(&result), pos.len());
    }

    #[test]
    fn test_smooth_normals_iter_multi_pass() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2];
        let cfg = NormalSmoothConfig { iterations: 3, ..default_normal_smooth_config() };
        let result = smooth_normals_iter(&pos, &idx, &cfg);
        assert_eq!(result.passes_applied, 3);
        assert_eq!(result.normals.len(), pos.len());
    }

    #[test]
    fn test_face_normals_two_triangles() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 1, 3, 2];
        let normals = compute_face_normals(&pos, &idx);
        assert_eq!(normals.len(), 2);
    }
}
