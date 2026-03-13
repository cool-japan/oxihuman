// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Texture projection utilities: planar, cylindrical, spherical, and box.

use std::f32::consts::{PI, TAU};

/// Projection mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TextureProjMode {
    Planar,
    Cylindrical,
    Spherical,
    Box,
}

/// Project a position using planar projection along the Z axis.
#[allow(dead_code)]
pub fn planar_project(pos: [f32; 3], scale: [f32; 2], offset: [f32; 2]) -> [f32; 2] {
    [pos[0] * scale[0] + offset[0], pos[1] * scale[1] + offset[1]]
}

/// Cylindrical projection (assumes axis = Y).
#[allow(dead_code)]
pub fn cylindrical_project(pos: [f32; 3], radius: f32, height: f32) -> [f32; 2] {
    let u = (pos[2].atan2(pos[0]) + PI) / TAU;
    let v = (pos[1] / height.max(1e-9)).clamp(0.0, 1.0);
    let _ = radius;
    [u, v]
}

/// Spherical projection.
#[allow(dead_code)]
pub fn spherical_project(pos: [f32; 3]) -> [f32; 2] {
    let len = (pos[0].powi(2) + pos[1].powi(2) + pos[2].powi(2))
        .sqrt()
        .max(1e-9);
    let u = (pos[2].atan2(pos[0]) + PI) / TAU;
    let v = (pos[1] / len).clamp(-1.0, 1.0).acos() / PI;
    [u, v]
}

/// Box projection: select the dominant axis and do planar projection on that face.
#[allow(dead_code)]
pub fn box_project(pos: [f32; 3]) -> [f32; 2] {
    let ax = pos[0].abs();
    let ay = pos[1].abs();
    let az = pos[2].abs();
    if ax >= ay && ax >= az {
        [pos[1] * 0.5 + 0.5, pos[2] * 0.5 + 0.5]
    } else if ay >= ax && ay >= az {
        [pos[0] * 0.5 + 0.5, pos[2] * 0.5 + 0.5]
    } else {
        [pos[0] * 0.5 + 0.5, pos[1] * 0.5 + 0.5]
    }
}

/// Apply a projection mode to all positions.
#[allow(dead_code)]
pub fn project_all(positions: &[[f32; 3]], mode: TextureProjMode) -> Vec<[f32; 2]> {
    positions
        .iter()
        .map(|&p| match mode {
            TextureProjMode::Planar => planar_project(p, [1.0, 1.0], [0.0, 0.0]),
            TextureProjMode::Cylindrical => cylindrical_project(p, 1.0, 2.0),
            TextureProjMode::Spherical => spherical_project(p),
            TextureProjMode::Box => box_project(p),
        })
        .collect()
}

/// Check that all UVs are within [0, 1].
#[allow(dead_code)]
pub fn uvs_in_unit_range(uvs: &[[f32; 2]]) -> bool {
    uvs.iter()
        .all(|uv| (0.0..=1.0).contains(&uv[0]) && (0.0..=1.0).contains(&uv[1]))
}

/// Tile UVs by a factor.
#[allow(dead_code)]
pub fn tile_uvs_proj(uvs: &[[f32; 2]], tile_u: f32, tile_v: f32) -> Vec<[f32; 2]> {
    uvs.iter()
        .map(|uv| [uv[0] * tile_u, uv[1] * tile_v])
        .collect()
}

/// Normalize UVs to [0, 1] range based on min/max.
#[allow(dead_code)]
pub fn normalize_uvs_proj(uvs: &[[f32; 2]]) -> Vec<[f32; 2]> {
    if uvs.is_empty() {
        return vec![];
    }
    let (mut mnu, mut mxu) = (f32::MAX, f32::MIN);
    let (mut mnv, mut mxv) = (f32::MAX, f32::MIN);
    for uv in uvs {
        mnu = mnu.min(uv[0]);
        mxu = mxu.max(uv[0]);
        mnv = mnv.min(uv[1]);
        mxv = mxv.max(uv[1]);
    }
    let ru = (mxu - mnu).max(1e-9);
    let rv = (mxv - mnv).max(1e-9);
    uvs.iter()
        .map(|uv| [(uv[0] - mnu) / ru, (uv[1] - mnv) / rv])
        .collect()
}

/// Compute UV bounds.
#[allow(dead_code)]
pub fn uv_bounds_proj(uvs: &[[f32; 2]]) -> ([f32; 2], [f32; 2]) {
    let mut mn = [f32::MAX; 2];
    let mut mx = [f32::MIN; 2];
    for uv in uvs {
        mn[0] = mn[0].min(uv[0]);
        mn[1] = mn[1].min(uv[1]);
        mx[0] = mx[0].max(uv[0]);
        mx[1] = mx[1].max(uv[1]);
    }
    (mn, mx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planar_project_basic() {
        let uv = planar_project([0.5, 0.5, 0.0], [1.0, 1.0], [0.0, 0.0]);
        assert!((uv[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn cylindrical_u_range() {
        let pos = vec![[1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [-1.0, 0.0, 0.0]];
        let uvs = project_all(&pos, TextureProjMode::Cylindrical);
        assert!(uvs.iter().all(|uv| (0.0..=1.0).contains(&uv[0])));
    }

    #[test]
    fn spherical_on_unit_sphere() {
        let pos = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let uvs = project_all(&pos, TextureProjMode::Spherical);
        // v should be in [0,1]
        assert!(uvs.iter().all(|uv| (0.0..=1.0).contains(&uv[1])));
    }

    #[test]
    fn box_project_in_range() {
        let pos = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let uvs: Vec<_> = pos.iter().map(|&p| box_project(p)).collect();
        assert!(uvs_in_unit_range(&uvs));
    }

    #[test]
    fn tile_uvs_scales() {
        let uvs = vec![[0.5, 0.5]];
        let t = tile_uvs_proj(&uvs, 2.0, 3.0);
        assert!((t[0][0] - 1.0).abs() < 1e-6);
        assert!((t[0][1] - 1.5).abs() < 1e-6);
    }

    #[test]
    fn normalize_uvs_range() {
        let uvs = vec![[0.0, 2.0], [0.5, 4.0], [1.0, 6.0]];
        let n = normalize_uvs_proj(&uvs);
        assert!((n[0][1] - 0.0).abs() < 1e-6);
        assert!((n[2][1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn uv_bounds_test() {
        let uvs = vec![[0.1, 0.2], [0.9, 0.8]];
        let (mn, mx) = uv_bounds_proj(&uvs);
        assert!((mn[0] - 0.1).abs() < 1e-6);
        assert!((mx[1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn empty_uvs_normalize() {
        let n = normalize_uvs_proj(&[]);
        assert!(n.is_empty());
    }

    #[test]
    fn pi_tau_used() {
        assert!((TAU - 2.0 * PI).abs() < 1e-5);
    }

    #[test]
    fn projection_mode_planar_all() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
        let uvs = project_all(&pos, TextureProjMode::Planar);
        assert_eq!(uvs.len(), 2);
    }
}
