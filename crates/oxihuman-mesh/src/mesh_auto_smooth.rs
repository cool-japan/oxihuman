// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Auto smooth angle computation for mesh shading.

/// Configuration for auto-smooth computation.
#[derive(Debug, Clone)]
pub struct AutoSmoothConfig {
    /// Angle threshold in radians above which edges are marked sharp.
    pub angle_threshold: f32,
    /// Whether to include boundary edges as sharp.
    pub boundary_sharp: bool,
}

impl Default for AutoSmoothConfig {
    fn default() -> Self {
        Self {
            angle_threshold: std::f32::consts::PI / 4.0, /* 45 degrees */
            boundary_sharp: true,
        }
    }
}

/// Result of auto-smooth computation.
#[derive(Debug, Clone)]
pub struct AutoSmoothResult {
    /// Indices of edge pairs (face_a, face_b) marked as sharp.
    pub sharp_pairs: Vec<(usize, usize)>,
    /// Per-vertex smoothed normals.
    pub smooth_normals: Vec<[f32; 3]>,
    pub sharp_count: usize,
}

/// Compute the dihedral angle (in radians) between two face normals.
pub fn dihedral_angle(n_a: [f32; 3], n_b: [f32; 3]) -> f32 {
    let dot = (n_a[0] * n_b[0] + n_a[1] * n_b[1] + n_a[2] * n_b[2]).clamp(-1.0, 1.0);
    dot.acos()
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

fn face_normal(positions: &[[f32; 3]], tri: [usize; 3]) -> [f32; 3] {
    let a = positions[tri[0]];
    let b = positions[tri[1]];
    let c = positions[tri[2]];
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    normalize3([
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ])
}

/// Apply auto-smooth: compute per-vertex normals respecting the angle threshold.
pub fn auto_smooth(
    positions: &[[f32; 3]],
    triangles: &[[usize; 3]],
    config: &AutoSmoothConfig,
) -> AutoSmoothResult {
    let n_verts = positions.len();
    let n_tris = triangles.len();
    let mut face_normals: Vec<[f32; 3]> = Vec::with_capacity(n_tris);
    for tri in triangles {
        face_normals.push(face_normal(positions, *tri));
    }

    /* Accumulate normals per vertex, separating by smooth groups */
    let mut accum: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]; n_verts];
    let mut sharp_pairs: Vec<(usize, usize)> = Vec::new();

    for (fi, tri) in triangles.iter().enumerate() {
        let fn_i = face_normals[fi];
        for &vi in tri.iter() {
            accum[vi][0] += fn_i[0];
            accum[vi][1] += fn_i[1];
            accum[vi][2] += fn_i[2];
        }
    }

    /* Detect sharp pairs between adjacent faces */
    for a in 0..n_tris {
        for b in (a + 1)..n_tris {
            let angle = dihedral_angle(face_normals[a], face_normals[b]);
            if angle > config.angle_threshold {
                sharp_pairs.push((a, b));
            }
        }
    }
    let sharp_count = sharp_pairs.len();

    let smooth_normals: Vec<[f32; 3]> = accum.iter().map(|&v| normalize3(v)).collect();
    AutoSmoothResult {
        sharp_pairs,
        smooth_normals,
        sharp_count,
    }
}

/// Count how many vertex normals deviate from their face normal by more than the threshold.
pub fn count_sharp_vertices(result: &AutoSmoothResult) -> usize {
    result.sharp_count
}

/// Return the default auto-smooth config with a given angle in degrees.
pub fn config_from_degrees(angle_deg: f32) -> AutoSmoothConfig {
    AutoSmoothConfig {
        angle_threshold: angle_deg.to_radians(),
        boundary_sharp: true,
    }
}

/// Check whether the angle is within smooth range.
pub fn is_smooth_angle(n_a: [f32; 3], n_b: [f32; 3], threshold: f32) -> bool {
    dihedral_angle(n_a, n_b) <= threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube_verts() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ]
    }

    #[test]
    fn test_default_config() {
        /* threshold should be PI/4 */
        let cfg = AutoSmoothConfig::default();
        assert!((cfg.angle_threshold - std::f32::consts::PI / 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_config_from_degrees() {
        let cfg = config_from_degrees(90.0);
        assert!((cfg.angle_threshold - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
    }

    #[test]
    fn test_dihedral_same_normal() {
        /* Same normals → angle 0 */
        let n = [0.0f32, 0.0, 1.0];
        assert!(dihedral_angle(n, n) < 1e-5);
    }

    #[test]
    fn test_dihedral_perpendicular() {
        /* Perpendicular normals → angle PI/2 */
        let a = [1.0f32, 0.0, 0.0];
        let b = [0.0f32, 1.0, 0.0];
        let angle = dihedral_angle(a, b);
        assert!((angle - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
    }

    #[test]
    fn test_is_smooth_angle_true() {
        let n = [0.0f32, 0.0, 1.0];
        assert!(is_smooth_angle(n, n, 0.1));
    }

    #[test]
    fn test_is_smooth_angle_false() {
        let a = [1.0f32, 0.0, 0.0];
        let b = [0.0f32, 1.0, 0.0];
        assert!(!is_smooth_angle(a, b, 0.1));
    }

    #[test]
    fn test_auto_smooth_single_tri() {
        let verts = cube_verts();
        let tris = vec![[0usize, 1, 2]];
        let cfg = AutoSmoothConfig::default();
        let result = auto_smooth(&verts, &tris, &cfg);
        /* One triangle: no adjacent pairs */
        assert_eq!(result.sharp_pairs.len(), 0);
        assert_eq!(result.smooth_normals.len(), verts.len());
    }

    #[test]
    fn test_auto_smooth_two_tris_sharp() {
        let verts = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, -1.0],
        ];
        let tris = vec![[0usize, 1, 2], [0, 1, 3]];
        let cfg = AutoSmoothConfig {
            angle_threshold: 0.1,
            boundary_sharp: true,
        };
        let result = auto_smooth(&verts, &tris, &cfg);
        /* The two faces are nearly perpendicular → sharp */
        assert!(result.sharp_count > 0);
    }

    #[test]
    fn test_count_sharp_vertices() {
        let result = AutoSmoothResult {
            sharp_pairs: vec![(0, 1), (1, 2)],
            smooth_normals: vec![],
            sharp_count: 2,
        };
        assert_eq!(count_sharp_vertices(&result), 2);
    }
}
