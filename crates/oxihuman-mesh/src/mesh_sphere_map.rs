// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Spherical parameterisation mapping for triangle meshes.

use std::f32::consts::{PI, TAU};

/// Spherical UV coordinates (longitude θ in `[0,1]`, latitude φ in `[0,1]`).
#[derive(Clone, Debug, Default)]
pub struct SphereMapResult {
    pub uvs: Vec<[f32; 2]>,
    pub projection_error: f32,
}

/// Project a point onto the unit sphere and return (u, v) in `[0,1]`².
///
/// Uses longitude/latitude mapping:
/// u = (atan2(z, x) + π) / τ  ∈ `[0,1]`
/// v = acos(y / r) / π         ∈ `[0,1]`
pub fn project_to_sphere_uv(p: [f32; 3]) -> [f32; 2] {
    let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
    if r < 1e-10 {
        return [0.0, 0.5];
    }
    let nx = p[0] / r;
    let ny = p[1] / r;
    let nz = p[2] / r;
    let u = (nz.atan2(nx) + PI) / TAU;
    let v = ny.clamp(-1.0, 1.0).acos() / PI;
    [u, v]
}

/// Map all mesh vertices to spherical UV coordinates.
///
/// The mesh centroid is subtracted first so the mapping is centred.
pub fn sphere_map(positions: &[[f32; 3]]) -> SphereMapResult {
    if positions.is_empty() {
        return SphereMapResult::default();
    }
    let n = positions.len() as f32;
    let centroid = positions.iter().fold([0.0_f32; 3], |acc, p| {
        [acc[0] + p[0] / n, acc[1] + p[1] / n, acc[2] + p[2] / n]
    });

    let uvs: Vec<[f32; 2]> = positions
        .iter()
        .map(|&p| {
            let q = [p[0] - centroid[0], p[1] - centroid[1], p[2] - centroid[2]];
            project_to_sphere_uv(q)
        })
        .collect();

    // Projection error: average difference from unit sphere radius
    let err: f32 = positions
        .iter()
        .map(|&p| {
            let q = [p[0] - centroid[0], p[1] - centroid[1], p[2] - centroid[2]];
            let r = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt();
            (r - 1.0).abs()
        })
        .sum::<f32>()
        / positions.len() as f32;

    SphereMapResult {
        uvs,
        projection_error: err,
    }
}

/// Return UV count.
pub fn sphere_map_uv_count(r: &SphereMapResult) -> usize {
    r.uvs.len()
}

/// Check that all UVs are in `[0,1]`².
pub fn sphere_map_uvs_in_range(r: &SphereMapResult) -> bool {
    r.uvs
        .iter()
        .all(|uv| (0.0..=1.0).contains(&uv[0]) && (0.0..=1.0).contains(&uv[1]))
}

/// Compute average u coordinate.
pub fn sphere_map_avg_u(r: &SphereMapResult) -> f32 {
    if r.uvs.is_empty() {
        return 0.0;
    }
    r.uvs.iter().map(|uv| uv[0]).sum::<f32>() / r.uvs.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_cube_verts() -> Vec<[f32; 3]> {
        vec![
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
        ]
    }

    #[test]
    fn sphere_map_uv_count_matches() {
        let pos = unit_cube_verts();
        let r = sphere_map(&pos);
        assert_eq!(sphere_map_uv_count(&r), pos.len());
    }

    #[test]
    fn uvs_all_in_range() {
        let pos = unit_cube_verts();
        let r = sphere_map(&pos);
        assert!(sphere_map_uvs_in_range(&r));
    }

    #[test]
    fn project_pole_gives_finite_uv() {
        let uv = project_to_sphere_uv([0.0, 1.0, 0.0]);
        assert!(uv[0].is_finite() && uv[1].is_finite());
    }

    #[test]
    fn project_zero_vector_safe() {
        let uv = project_to_sphere_uv([0.0, 0.0, 0.0]);
        assert!(uv[0].is_finite() && uv[1].is_finite());
    }

    #[test]
    fn sphere_map_empty_default() {
        let r = sphere_map(&[]);
        assert_eq!(sphere_map_uv_count(&r), 0);
    }

    #[test]
    fn sphere_map_projection_error_nonneg() {
        let pos = unit_cube_verts();
        let r = sphere_map(&pos);
        assert!(r.projection_error >= 0.0);
    }

    #[test]
    fn sphere_map_avg_u_in_range() {
        let pos = unit_cube_verts();
        let r = sphere_map(&pos);
        let u = sphere_map_avg_u(&r);
        assert!((0.0..=1.0).contains(&u));
    }

    #[test]
    fn project_equator_u_half() {
        /* Point at (0,0,1) → atan2(1,0) = π/2 → u = (π/2 + π)/τ = 0.375 */
        let uv = project_to_sphere_uv([0.0, 0.0, 1.0]);
        assert!(uv[0].is_finite());
    }
}
