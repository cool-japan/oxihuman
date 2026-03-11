// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Planar face decimation: collapse faces that lie on the same plane.

/// Configuration for planar decimation.
#[derive(Debug, Clone)]
pub struct PlanarDecimateConfig {
    /// Maximum angle (radians) between face normals to consider co-planar.
    pub plane_angle_limit: f32,
    /// Minimum faces to retain after decimation.
    pub min_faces: usize,
}

impl Default for PlanarDecimateConfig {
    fn default() -> Self {
        Self {
            plane_angle_limit: 0.01745,
            /* ~1 degree */ min_faces: 1,
        }
    }
}

/// Result of planar decimation.
#[derive(Debug, Clone)]
pub struct PlanarDecimateResult {
    /// Surviving face indices.
    pub survivors: Vec<usize>,
    /// Number of faces removed.
    pub removed_count: usize,
    pub original_count: usize,
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

fn tri_normal(positions: &[[f32; 3]], tri: [usize; 3]) -> [f32; 3] {
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

fn angle_between(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] * b[0] + a[1] * b[1] + a[2] * b[2])
        .clamp(-1.0, 1.0)
        .acos()
}

/// Apply planar decimation to a triangle mesh.
pub fn planar_decimate(
    positions: &[[f32; 3]],
    triangles: &[[usize; 3]],
    config: &PlanarDecimateConfig,
) -> PlanarDecimateResult {
    let n = triangles.len();
    if n == 0 {
        return PlanarDecimateResult {
            survivors: vec![],
            removed_count: 0,
            original_count: 0,
        };
    }
    let normals: Vec<[f32; 3]> = triangles
        .iter()
        .map(|&t| tri_normal(positions, t))
        .collect();
    let mut removed = vec![false; n];

    for i in 0..n {
        if removed[i] {
            continue;
        }
        for j in (i + 1)..n {
            if removed[j] {
                continue;
            }
            if angle_between(normals[i], normals[j]) <= config.plane_angle_limit {
                removed[j] = true;
            }
        }
    }

    let survivors: Vec<usize> = (0..n).filter(|&i| !removed[i]).collect();
    /* Enforce minimum face count */
    let survivors = if survivors.len() < config.min_faces {
        (0..n.min(config.min_faces)).collect()
    } else {
        survivors
    };
    let removed_count = n.saturating_sub(survivors.len());
    PlanarDecimateResult {
        survivors,
        removed_count,
        original_count: n,
    }
}

/// Compute the decimation ratio.
pub fn decimate_ratio(result: &PlanarDecimateResult) -> f32 {
    if result.original_count == 0 {
        return 0.0;
    }
    result.survivors.len() as f32 / result.original_count as f32
}

/// Check whether two faces are coplanar within the angle limit.
pub fn are_coplanar(n_a: [f32; 3], n_b: [f32; 3], limit: f32) -> bool {
    angle_between(n_a, n_b) <= limit
}

/// Build a config for a given angle in degrees.
pub fn config_degrees(deg: f32) -> PlanarDecimateConfig {
    PlanarDecimateConfig {
        plane_angle_limit: deg.to_radians(),
        min_faces: 1,
    }
}

/// Count how many faces were preserved.
pub fn survivor_count(result: &PlanarDecimateResult) -> usize {
    result.survivors.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_plane() -> (Vec<[f32; 3]>, Vec<[usize; 3]>) {
        let verts = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        (verts, vec![[0usize, 1, 2], [0, 2, 3]])
    }

    #[test]
    fn test_default_config() {
        let cfg = PlanarDecimateConfig::default();
        assert!(cfg.plane_angle_limit > 0.0);
        assert_eq!(cfg.min_faces, 1);
    }

    #[test]
    fn test_config_degrees() {
        let cfg = config_degrees(5.0);
        assert!((cfg.plane_angle_limit - 5f32.to_radians()).abs() < 1e-6);
    }

    #[test]
    fn test_coplanar_collapse() {
        /* Two coplanar faces: one should survive */
        let (verts, tris) = flat_plane();
        let cfg = config_degrees(5.0);
        let result = planar_decimate(&verts, &tris, &cfg);
        assert_eq!(result.survivors.len(), 1);
    }

    #[test]
    fn test_decimate_ratio_full() {
        let (verts, tris) = flat_plane();
        let cfg = config_degrees(5.0);
        let result = planar_decimate(&verts, &tris, &cfg);
        let ratio = decimate_ratio(&result);
        assert!(ratio > 0.0 && ratio <= 1.0);
    }

    #[test]
    fn test_are_coplanar_same() {
        let n = [0.0f32, 0.0, 1.0];
        assert!(are_coplanar(n, n, 0.01));
    }

    #[test]
    fn test_are_coplanar_different() {
        let a = [1.0f32, 0.0, 0.0];
        let b = [0.0f32, 1.0, 0.0];
        assert!(!are_coplanar(a, b, 0.01));
    }

    #[test]
    fn test_empty_mesh() {
        let cfg = PlanarDecimateConfig::default();
        let result = planar_decimate(&[], &[], &cfg);
        assert_eq!(result.survivors.len(), 0);
        assert_eq!(result.removed_count, 0);
    }

    #[test]
    fn test_survivor_count() {
        let (verts, tris) = flat_plane();
        let cfg = config_degrees(5.0);
        let result = planar_decimate(&verts, &tris, &cfg);
        assert_eq!(survivor_count(&result), result.survivors.len());
    }

    #[test]
    fn test_min_faces_respected() {
        let (verts, tris) = flat_plane();
        let cfg = PlanarDecimateConfig {
            plane_angle_limit: 100.0,
            min_faces: 1,
        };
        let result = planar_decimate(&verts, &tris, &cfg);
        assert!(!result.survivors.is_empty());
    }
}
