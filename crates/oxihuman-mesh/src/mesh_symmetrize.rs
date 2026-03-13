// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mesh symmetrize: mirror and symmetrize geometry across a chosen axis plane.
//!
//! Supports X, Y, and Z mirror planes, finds paired vertices within a
//! tolerance, averages positions for symmetrization, and measures
//! asymmetry error.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// The axis whose zero-plane is used as the mirror plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymmetryAxis {
    /// Mirror across the X = 0 plane (negate x).
    X,
    /// Mirror across the Y = 0 plane (negate y).
    Y,
    /// Mirror across the Z = 0 plane (negate z).
    Z,
}

/// Configuration for symmetrize operations.
pub struct SymmetrizeConfig {
    /// Which axis plane to mirror across.
    pub axis: SymmetryAxis,
    /// Vertex-pair search tolerance: vertices whose mirrored position is
    /// within this distance are considered paired.
    pub tolerance: f32,
    /// If true, vertices on the mirror plane (within `tolerance`) are snapped
    /// exactly to zero on the axis component.
    pub snap_on_axis: bool,
    /// If true, symmetrize_mesh averages both sides; if false, uses only the
    /// positive-axis side and mirrors it.
    pub average_both_sides: bool,
}

/// Result returned by symmetrize operations.
pub struct SymmetrizeResult {
    /// The symmetrized positions (same vertex count as input).
    pub positions: Vec<[f32; 3]>,
    /// Symmetrized normals (same length as positions, or empty if not provided).
    pub normals: Vec<[f32; 3]>,
    /// Number of vertices that found a mirror pair.
    pub paired_count: usize,
    /// RMS asymmetry error before symmetrization.
    pub pre_error: f32,
    /// RMS asymmetry error after symmetrization (should be ~0).
    pub post_error: f32,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Return a `SymmetrizeConfig` with sensible defaults (X axis, tol = 0.001).
pub fn default_symmetrize_config() -> SymmetrizeConfig {
    SymmetrizeConfig {
        axis: SymmetryAxis::X,
        tolerance: 0.001,
        snap_on_axis: true,
        average_both_sides: true,
    }
}

/// Return the display name of a `SymmetryAxis`.
pub fn symmetry_axis_name(axis: SymmetryAxis) -> &'static str {
    match axis {
        SymmetryAxis::X => "X",
        SymmetryAxis::Y => "Y",
        SymmetryAxis::Z => "Z",
    }
}

/// Mirror a single vertex position across the given axis plane.
pub fn mirror_vertex_position(p: [f32; 3], axis: SymmetryAxis) -> [f32; 3] {
    match axis {
        SymmetryAxis::X => [-p[0], p[1], p[2]],
        SymmetryAxis::Y => [p[0], -p[1], p[2]],
        SymmetryAxis::Z => [p[0], p[1], -p[2]],
    }
}

/// Return true if vertex `p` lies on the mirror plane within `tolerance`.
pub fn vertex_is_on_axis(p: [f32; 3], axis: SymmetryAxis, tolerance: f32) -> bool {
    let component = match axis {
        SymmetryAxis::X => p[0],
        SymmetryAxis::Y => p[1],
        SymmetryAxis::Z => p[2],
    };
    component.abs() <= tolerance
}

/// Snap the axis component of `p` to zero if it is within `tolerance`.
pub fn snap_to_symmetry_plane(p: [f32; 3], axis: SymmetryAxis, tolerance: f32) -> [f32; 3] {
    let mut out = p;
    let component = match axis {
        SymmetryAxis::X => p[0],
        SymmetryAxis::Y => p[1],
        SymmetryAxis::Z => p[2],
    };
    if component.abs() <= tolerance {
        match axis {
            SymmetryAxis::X => out[0] = 0.0,
            SymmetryAxis::Y => out[1] = 0.0,
            SymmetryAxis::Z => out[2] = 0.0,
        }
    }
    out
}

/// Find the index of the nearest vertex to the mirrored position of `positions[idx]`.
///
/// Returns `None` if no vertex is within `tolerance` (excluding `idx` itself).
pub fn find_symmetric_vertex(
    positions: &[[f32; 3]],
    idx: usize,
    axis: SymmetryAxis,
    tolerance: f32,
) -> Option<usize> {
    if idx >= positions.len() {
        return None;
    }
    let mirrored = mirror_vertex_position(positions[idx], axis);
    let mut best_dist = tolerance;
    let mut best_idx = None;

    for (i, &p) in positions.iter().enumerate() {
        if i == idx {
            continue;
        }
        let d = len3(sub3(p, mirrored));
        if d < best_dist {
            best_dist = d;
            best_idx = Some(i);
        }
    }
    best_idx
}

/// Mirror all vertex positions across the chosen axis plane.
///
/// Indices are preserved; normals (if provided) have their axis component
/// negated to match the mirrored orientation.
pub fn mirror_mesh(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    indices: &[u32],
    axis: SymmetryAxis,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let mirrored_pos: Vec<[f32; 3]> = positions
        .iter()
        .map(|&p| mirror_vertex_position(p, axis))
        .collect();

    let mirrored_nrm: Vec<[f32; 3]> = normals
        .iter()
        .map(|&n| mirror_vertex_position(n, axis)) // negates same component
        .collect();

    // Flip winding order so faces still point outward after mirroring
    let flipped_idx: Vec<u32> = indices
        .chunks_exact(3)
        .flat_map(|tri| [tri[0], tri[2], tri[1]])
        .collect();

    (mirrored_pos, mirrored_nrm, flipped_idx)
}

/// Symmetrize positions: for each paired vertex, either average or copy the
/// positive-axis side to the negative-axis side.
pub fn symmetrize_mesh(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    cfg: &SymmetrizeConfig,
) -> SymmetrizeResult {
    let n = positions.len();
    let pre_error = symmetry_error(positions, cfg.axis, cfg.tolerance);

    let mut out_pos = positions.to_vec();
    let mut out_nrm = normals.to_vec();
    let mut paired_count = 0usize;

    // Build a visited bitmap so we don't process the same pair twice
    let mut visited = vec![false; n];

    for i in 0..n {
        if visited[i] {
            continue;
        }
        // Check if vertex is on the mirror plane
        if vertex_is_on_axis(positions[i], cfg.axis, cfg.tolerance) {
            if cfg.snap_on_axis {
                out_pos[i] = snap_to_symmetry_plane(positions[i], cfg.axis, cfg.tolerance);
            }
            visited[i] = true;
            continue;
        }

        // Try to find the mirror partner
        if let Some(j) = find_symmetric_vertex(positions, i, cfg.axis, cfg.tolerance) {
            if !visited[j] {
                if cfg.average_both_sides {
                    // Average: pi_sym = (pi + mirror(pj)) / 2
                    let mirror_j = mirror_vertex_position(positions[j], cfg.axis);
                    let avg = scale3(add3(positions[i], mirror_j), 0.5);
                    out_pos[i] = avg;
                    out_pos[j] = mirror_vertex_position(avg, cfg.axis);

                    if !out_nrm.is_empty() && i < out_nrm.len() && j < out_nrm.len() {
                        let mirror_nj = mirror_vertex_position(normals[j], cfg.axis);
                        let avg_n = normalize3(add3(normals[i], mirror_nj));
                        out_nrm[i] = avg_n;
                        out_nrm[j] = mirror_vertex_position(avg_n, cfg.axis);
                    }
                } else {
                    // Copy positive-axis side to negative-axis side
                    let axis_val = match cfg.axis {
                        SymmetryAxis::X => positions[i][0],
                        SymmetryAxis::Y => positions[i][1],
                        SymmetryAxis::Z => positions[i][2],
                    };
                    let (src, dst) = if axis_val >= 0.0 { (i, j) } else { (j, i) };
                    out_pos[dst] = mirror_vertex_position(out_pos[src], cfg.axis);
                    if !out_nrm.is_empty() && src < out_nrm.len() && dst < out_nrm.len() {
                        out_nrm[dst] = mirror_vertex_position(out_nrm[src], cfg.axis);
                    }
                }
                paired_count += 1;
                visited[i] = true;
                visited[j] = true;
            }
        }
    }

    let post_error = symmetry_error(&out_pos, cfg.axis, cfg.tolerance);

    SymmetrizeResult {
        positions: out_pos,
        normals: out_nrm,
        paired_count,
        pre_error,
        post_error,
    }
}

/// Symmetrize a per-vertex weight slice in the same way as positions.
///
/// For each paired (i, j), the weight is averaged if `average_both_sides`,
/// otherwise the positive-axis vertex's weight is copied to its mirror.
pub fn symmetrize_weights(
    positions: &[[f32; 3]],
    weights: &[f32],
    cfg: &SymmetrizeConfig,
) -> Vec<f32> {
    let n = positions.len();
    let mut out = weights.to_vec();
    out.resize(n, 0.0);
    let mut visited = vec![false; n];

    for i in 0..n {
        if visited[i] {
            continue;
        }
        if vertex_is_on_axis(positions[i], cfg.axis, cfg.tolerance) {
            visited[i] = true;
            continue;
        }
        if let Some(j) = find_symmetric_vertex(positions, i, cfg.axis, cfg.tolerance) {
            if !visited[j] {
                if cfg.average_both_sides {
                    let avg = (out[i] + out[j]) * 0.5;
                    out[i] = avg;
                    out[j] = avg;
                } else {
                    let axis_val = match cfg.axis {
                        SymmetryAxis::X => positions[i][0],
                        SymmetryAxis::Y => positions[i][1],
                        SymmetryAxis::Z => positions[i][2],
                    };
                    let (src, dst) = if axis_val >= 0.0 { (i, j) } else { (j, i) };
                    out[dst] = out[src];
                }
                visited[i] = true;
                visited[j] = true;
            }
        }
    }
    out
}

/// Symmetrize normal vectors: paired normals have their axis component negated
/// and optionally averaged.
pub fn symmetrize_normals(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    cfg: &SymmetrizeConfig,
) -> Vec<[f32; 3]> {
    let n = positions.len();
    let mut out = normals.to_vec();
    out.resize(n, [0.0, 1.0, 0.0]);
    let mut visited = vec![false; n];

    for i in 0..n {
        if visited[i] {
            continue;
        }
        if vertex_is_on_axis(positions[i], cfg.axis, cfg.tolerance) {
            visited[i] = true;
            continue;
        }
        if let Some(j) = find_symmetric_vertex(positions, i, cfg.axis, cfg.tolerance) {
            if !visited[j] {
                let mirror_nj = mirror_vertex_position(normals[j], cfg.axis);
                if cfg.average_both_sides {
                    let avg = normalize3(add3(normals[i], mirror_nj));
                    out[i] = avg;
                    out[j] = mirror_vertex_position(avg, cfg.axis);
                } else {
                    let axis_val = match cfg.axis {
                        SymmetryAxis::X => positions[i][0],
                        SymmetryAxis::Y => positions[i][1],
                        SymmetryAxis::Z => positions[i][2],
                    };
                    let (src, dst) = if axis_val >= 0.0 { (i, j) } else { (j, i) };
                    out[dst] = mirror_vertex_position(out[src], cfg.axis);
                }
                visited[i] = true;
                visited[j] = true;
            }
        }
    }
    out
}

/// Measure asymmetry: for each vertex, find its mirror partner and accumulate
/// the squared distance difference; return the RMS value.
pub fn symmetry_error(positions: &[[f32; 3]], axis: SymmetryAxis, tolerance: f32) -> f32 {
    let n = positions.len();
    if n == 0 {
        return 0.0;
    }
    let mut sum_sq = 0.0f32;
    let mut count = 0usize;

    for i in 0..n {
        if let Some(j) = find_symmetric_vertex(positions, i, axis, tolerance * 10.0) {
            let mirrored_i = mirror_vertex_position(positions[i], axis);
            let d = len3(sub3(positions[j], mirrored_i));
            sum_sq += d * d;
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        (sum_sq / count as f32).sqrt()
    }
}

/// Count the number of vertices that have a symmetric partner within `tolerance`.
pub fn count_paired_vertices(
    positions: &[[f32; 3]],
    axis: SymmetryAxis,
    tolerance: f32,
) -> usize {
    let n = positions.len();
    let mut visited = vec![false; n];
    let mut count = 0usize;

    for i in 0..n {
        if visited[i] {
            continue;
        }
        if let Some(j) = find_symmetric_vertex(positions, i, axis, tolerance) {
            if !visited[j] {
                count += 1;
                visited[i] = true;
                visited[j] = true;
            }
        }
    }
    count
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sym_positions() -> Vec<[f32; 3]> {
        // Four vertices: two pairs symmetric about X=0
        vec![
            [1.0, 0.5, 0.0],
            [-1.0, 0.5, 0.0],
            [2.0, -0.5, 1.0],
            [-2.0, -0.5, 1.0],
        ]
    }

    fn asymmetric_positions() -> Vec<[f32; 3]> {
        // Slightly asymmetric version
        vec![
            [1.0, 0.5, 0.0],
            [-1.05, 0.5, 0.0],
            [2.0, -0.5, 1.0],
            [-2.1, -0.5, 1.0],
        ]
    }

    // -----------------------------------------------------------------------
    // default_symmetrize_config
    // -----------------------------------------------------------------------

    #[test]
    fn default_config_axis_is_x() {
        let cfg = default_symmetrize_config();
        assert_eq!(cfg.axis, SymmetryAxis::X);
    }

    #[test]
    fn default_config_tolerance_positive() {
        let cfg = default_symmetrize_config();
        assert!(cfg.tolerance > 0.0);
    }

    // -----------------------------------------------------------------------
    // symmetry_axis_name
    // -----------------------------------------------------------------------

    #[test]
    fn axis_name_x() {
        assert_eq!(symmetry_axis_name(SymmetryAxis::X), "X");
    }

    #[test]
    fn axis_name_y() {
        assert_eq!(symmetry_axis_name(SymmetryAxis::Y), "Y");
    }

    #[test]
    fn axis_name_z() {
        assert_eq!(symmetry_axis_name(SymmetryAxis::Z), "Z");
    }

    // -----------------------------------------------------------------------
    // mirror_vertex_position
    // -----------------------------------------------------------------------

    #[test]
    fn mirror_x_negates_x() {
        let p = [3.0f32, 1.0, 2.0];
        let m = mirror_vertex_position(p, SymmetryAxis::X);
        assert!((m[0] + 3.0).abs() < 1e-6);
        assert!((m[1] - 1.0).abs() < 1e-6);
        assert!((m[2] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn mirror_y_negates_y() {
        let p = [1.0f32, 4.0, 2.0];
        let m = mirror_vertex_position(p, SymmetryAxis::Y);
        assert!((m[1] + 4.0).abs() < 1e-6);
    }

    #[test]
    fn mirror_z_negates_z() {
        let p = [1.0f32, 2.0, 5.0];
        let m = mirror_vertex_position(p, SymmetryAxis::Z);
        assert!((m[2] + 5.0).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // vertex_is_on_axis
    // -----------------------------------------------------------------------

    #[test]
    fn vertex_on_axis_within_tolerance() {
        let p = [0.0005f32, 1.0, 0.0];
        assert!(vertex_is_on_axis(p, SymmetryAxis::X, 0.001));
    }

    #[test]
    fn vertex_not_on_axis_outside_tolerance() {
        let p = [1.0f32, 0.0, 0.0];
        assert!(!vertex_is_on_axis(p, SymmetryAxis::X, 0.001));
    }

    // -----------------------------------------------------------------------
    // snap_to_symmetry_plane
    // -----------------------------------------------------------------------

    #[test]
    fn snap_zeros_small_x_component() {
        let p = [0.0005f32, 1.0, 2.0];
        let snapped = snap_to_symmetry_plane(p, SymmetryAxis::X, 0.001);
        assert!(snapped[0].abs() < 1e-10);
        assert!((snapped[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn snap_leaves_large_x_component() {
        let p = [1.0f32, 0.0, 0.0];
        let snapped = snap_to_symmetry_plane(p, SymmetryAxis::X, 0.001);
        assert!((snapped[0] - 1.0).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // find_symmetric_vertex
    // -----------------------------------------------------------------------

    #[test]
    fn find_symmetric_vertex_finds_mirror_pair() {
        let pos = sym_positions();
        // Vertex 0 at [1,0.5,0] mirrors to [-1,0.5,0] which is vertex 1
        let partner = find_symmetric_vertex(&pos, 0, SymmetryAxis::X, 0.01);
        assert_eq!(partner, Some(1));
    }

    #[test]
    fn find_symmetric_vertex_returns_none_if_no_match() {
        let pos = vec![[1.0f32, 0.0, 0.0], [5.0, 0.0, 0.0]];
        let partner = find_symmetric_vertex(&pos, 0, SymmetryAxis::X, 0.01);
        assert!(partner.is_none());
    }

    // -----------------------------------------------------------------------
    // mirror_mesh
    // -----------------------------------------------------------------------

    #[test]
    fn mirror_mesh_negates_positions() {
        let pos = vec![[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let nrm = vec![[0.0f32, 0.0, 1.0]; 2];
        let idx = vec![0u32, 1, 0];
        let (mp, _, _) = mirror_mesh(&pos, &nrm, &idx, SymmetryAxis::X);
        assert!((mp[0][0] + 1.0).abs() < 1e-6);
        assert!((mp[1][0] + 4.0).abs() < 1e-6);
    }

    #[test]
    fn mirror_mesh_flips_winding() {
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let nrm = vec![[0.0f32, 0.0, 1.0]; 3];
        let idx = vec![0u32, 1, 2];
        let (_, _, flipped) = mirror_mesh(&pos, &nrm, &idx, SymmetryAxis::X);
        // Winding flipped: should be 0, 2, 1
        assert_eq!(flipped, vec![0, 2, 1]);
    }

    // -----------------------------------------------------------------------
    // symmetrize_mesh
    // -----------------------------------------------------------------------

    #[test]
    fn symmetrize_mesh_reduces_error() {
        let pos = asymmetric_positions();
        let cfg = default_symmetrize_config();
        let result = symmetrize_mesh(&pos, &[], &cfg);
        assert!(result.post_error <= result.pre_error + 1e-5);
    }

    #[test]
    fn symmetrize_mesh_reports_paired_count() {
        let pos = sym_positions();
        let cfg = SymmetrizeConfig {
            axis: SymmetryAxis::X,
            tolerance: 0.05,
            snap_on_axis: true,
            average_both_sides: true,
        };
        let result = symmetrize_mesh(&pos, &[], &cfg);
        assert!(result.paired_count >= 2);
    }

    // -----------------------------------------------------------------------
    // count_paired_vertices
    // -----------------------------------------------------------------------

    #[test]
    fn count_paired_vertices_symmetric_mesh() {
        let pos = sym_positions();
        let count = count_paired_vertices(&pos, SymmetryAxis::X, 0.05);
        assert_eq!(count, 2);
    }

    #[test]
    fn count_paired_vertices_no_pairs() {
        let pos = vec![[1.0f32, 0.0, 0.0], [3.0, 0.0, 0.0]];
        let count = count_paired_vertices(&pos, SymmetryAxis::X, 0.01);
        assert_eq!(count, 0);
    }

    // -----------------------------------------------------------------------
    // symmetrize_weights
    // -----------------------------------------------------------------------

    #[test]
    fn symmetrize_weights_averages_pairs() {
        let pos = sym_positions();
        let weights = vec![0.8f32, 0.4, 0.6, 0.2];
        let cfg = SymmetrizeConfig {
            axis: SymmetryAxis::X,
            tolerance: 0.05,
            snap_on_axis: false,
            average_both_sides: true,
        };
        let out = symmetrize_weights(&pos, &weights, &cfg);
        // Vertices 0 and 1 are paired: expected avg = (0.8+0.4)/2 = 0.6
        assert!((out[0] - 0.6).abs() < 1e-5, "out[0]={}", out[0]);
        assert!((out[1] - 0.6).abs() < 1e-5, "out[1]={}", out[1]);
    }

    // -----------------------------------------------------------------------
    // symmetrize_normals
    // -----------------------------------------------------------------------

    #[test]
    fn symmetrize_normals_mirrors_partner() {
        let pos = sym_positions();
        let nrm = vec![
            [0.0f32, 0.0, 1.0],
            [0.1, 0.0, 1.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let cfg = SymmetrizeConfig {
            axis: SymmetryAxis::X,
            tolerance: 0.05,
            snap_on_axis: false,
            average_both_sides: true,
        };
        let out = symmetrize_normals(&pos, &nrm, &cfg);
        // After averaging, normals[0] and normals[1] should be mirrors of each other
        let mirror_1 = mirror_vertex_position(out[1], SymmetryAxis::X);
        let diff = len3(sub3(out[0], mirror_1));
        assert!(diff < 1e-4, "normals not mirrored: diff={}", diff);
    }

    // -----------------------------------------------------------------------
    // symmetry_error
    // -----------------------------------------------------------------------

    #[test]
    fn symmetry_error_zero_for_perfect_symmetry() {
        let pos = sym_positions();
        let err = symmetry_error(&pos, SymmetryAxis::X, 0.05);
        assert!(err < 1e-5, "expected ~0 error, got {}", err);
    }

    #[test]
    fn symmetry_error_positive_for_asymmetric() {
        let pos = asymmetric_positions();
        let err = symmetry_error(&pos, SymmetryAxis::X, 0.5);
        assert!(err > 0.0, "expected non-zero error for asymmetric mesh");
    }
}
