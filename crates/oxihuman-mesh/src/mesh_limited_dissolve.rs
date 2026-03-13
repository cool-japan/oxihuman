// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Limited dissolve: merge coplanar/near-coplanar faces by angle threshold.

/// Configuration for limited dissolve.
#[derive(Debug, Clone)]
pub struct LimitedDissolveConfig {
    /// Maximum dihedral angle (radians) between faces to dissolve.
    pub angle_limit: f32,
    /// Whether to dissolve boundary edges.
    pub dissolve_boundary: bool,
}

impl Default for LimitedDissolveConfig {
    fn default() -> Self {
        Self {
            angle_limit: 0.0872665, /* ~5 degrees */
            dissolve_boundary: false,
        }
    }
}

/// Result of limited dissolve.
#[derive(Debug, Clone)]
pub struct DissolveResult {
    /// Remaining face groups after dissolve (each group is a list of original face indices).
    pub face_groups: Vec<Vec<usize>>,
    pub dissolved_count: usize,
    pub remaining_count: usize,
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

fn face_normal_tri(positions: &[[f32; 3]], tri: [usize; 3]) -> [f32; 3] {
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

fn angle_between(n_a: [f32; 3], n_b: [f32; 3]) -> f32 {
    (n_a[0] * n_b[0] + n_a[1] * n_b[1] + n_a[2] * n_b[2])
        .clamp(-1.0, 1.0)
        .acos()
}

/// Perform limited dissolve on a triangle mesh.
pub fn limited_dissolve(
    positions: &[[f32; 3]],
    triangles: &[[usize; 3]],
    config: &LimitedDissolveConfig,
) -> DissolveResult {
    let n_tris = triangles.len();
    let normals: Vec<[f32; 3]> = triangles
        .iter()
        .map(|&tri| face_normal_tri(positions, tri))
        .collect();

    /* Union-find grouping */
    let mut parent: Vec<usize> = (0..n_tris).collect();

    fn find(parent: &mut Vec<usize>, x: usize) -> usize {
        if parent[x] != x {
            parent[x] = find(parent, parent[x]);
        }
        parent[x]
    }

    for a in 0..n_tris {
        for b in (a + 1)..n_tris {
            let angle = angle_between(normals[a], normals[b]);
            if angle <= config.angle_limit {
                let ra = find(&mut parent, a);
                let rb = find(&mut parent, b);
                if ra != rb {
                    parent[rb] = ra;
                }
            }
        }
    }

    /* Collect groups */
    let mut groups: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for i in 0..n_tris {
        let root = find(&mut parent, i);
        groups.entry(root).or_default().push(i);
    }

    let face_groups: Vec<Vec<usize>> = groups.into_values().collect();
    let dissolved_count = n_tris.saturating_sub(face_groups.len());
    let remaining_count = face_groups.len();
    DissolveResult {
        face_groups,
        dissolved_count,
        remaining_count,
    }
}

/// Count the total faces that were effectively dissolved (merged into groups).
pub fn total_dissolved(result: &DissolveResult) -> usize {
    result.dissolved_count
}

/// Return groups with more than one face (actual dissolve candidates).
pub fn multi_face_groups(result: &DissolveResult) -> Vec<&Vec<usize>> {
    result.face_groups.iter().filter(|g| g.len() > 1).collect()
}

/// Build a default config with the given angle limit in degrees.
pub fn config_from_degrees(deg: f32) -> LimitedDissolveConfig {
    LimitedDissolveConfig {
        angle_limit: deg.to_radians(),
        dissolve_boundary: false,
    }
}

/// Check if two normals are within dissolve range.
pub fn within_dissolve_angle(n_a: [f32; 3], n_b: [f32; 3], limit: f32) -> bool {
    angle_between(n_a, n_b) <= limit
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_quad_tris() -> (Vec<[f32; 3]>, Vec<[usize; 3]>) {
        let verts = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        /* Two coplanar triangles from a flat quad */
        let tris = vec![[0usize, 1, 2], [0, 2, 3]];
        (verts, tris)
    }

    #[test]
    fn test_default_config_angle() {
        let cfg = LimitedDissolveConfig::default();
        assert!(cfg.angle_limit > 0.0 && cfg.angle_limit < 0.2);
    }

    #[test]
    fn test_config_from_degrees() {
        let cfg = config_from_degrees(5.0);
        assert!((cfg.angle_limit - 5f32.to_radians()).abs() < 1e-6);
    }

    #[test]
    fn test_within_dissolve_same() {
        let n = [0.0f32, 0.0, 1.0];
        assert!(within_dissolve_angle(n, n, 0.1));
    }

    #[test]
    fn test_coplanar_dissolve() {
        /* Coplanar faces should merge into a single group */
        let (verts, tris) = flat_quad_tris();
        let cfg = config_from_degrees(10.0);
        let result = limited_dissolve(&verts, &tris, &cfg);
        /* Both triangles are coplanar → one group */
        assert_eq!(result.remaining_count, 1);
    }

    #[test]
    fn test_perpendicular_no_dissolve() {
        let verts = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let tris = vec![[0usize, 1, 2], [0, 1, 3]];
        let cfg = config_from_degrees(2.0);
        let result = limited_dissolve(&verts, &tris, &cfg);
        assert_eq!(result.remaining_count, 2);
    }

    #[test]
    fn test_total_dissolved_coplanar() {
        let (verts, tris) = flat_quad_tris();
        let cfg = config_from_degrees(10.0);
        let result = limited_dissolve(&verts, &tris, &cfg);
        /* 2 input - 1 group = 1 dissolved */
        assert_eq!(total_dissolved(&result), 1);
    }

    #[test]
    fn test_multi_face_groups() {
        let (verts, tris) = flat_quad_tris();
        let cfg = config_from_degrees(10.0);
        let result = limited_dissolve(&verts, &tris, &cfg);
        let mfg = multi_face_groups(&result);
        /* Coplanar → one multi-face group */
        assert_eq!(mfg.len(), 1);
    }

    #[test]
    fn test_within_dissolve_perpendicular() {
        let a = [1.0f32, 0.0, 0.0];
        let b = [0.0f32, 1.0, 0.0];
        assert!(!within_dissolve_angle(a, b, 0.1));
    }

    #[test]
    fn test_empty_mesh() {
        let verts: Vec<[f32; 3]> = vec![];
        let tris: Vec<[usize; 3]> = vec![];
        let cfg = LimitedDissolveConfig::default();
        let result = limited_dissolve(&verts, &tris, &cfg);
        assert_eq!(result.remaining_count, 0);
    }
}
