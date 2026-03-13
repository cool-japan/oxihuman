// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! UV rotate-to-align selected edges tool.

/// Aligns the UV island so that a selected edge becomes horizontal (U-axis aligned).
pub struct UvRotateAlignResult {
    pub rotation_deg: f32,
    pub vertex_count: usize,
}

/// Compute angle of an edge in UV space (in degrees).
pub fn edge_uv_angle_deg(uv0: [f32; 2], uv1: [f32; 2]) -> f32 {
    let du = uv1[0] - uv0[0];
    let dv = uv1[1] - uv0[1];
    dv.atan2(du).to_degrees()
}

/// Rotate all UVs so that the given edge becomes horizontal.
/// Returns the angle applied (in degrees) and the count of modified UVs.
pub fn rotate_align_edge(
    uvs: &mut [[f32; 2]],
    edge_v0: usize,
    edge_v1: usize,
) -> UvRotateAlignResult {
    if edge_v0 >= uvs.len() || edge_v1 >= uvs.len() {
        return UvRotateAlignResult {
            rotation_deg: 0.0,
            vertex_count: 0,
        };
    }
    let angle = edge_uv_angle_deg(uvs[edge_v0], uvs[edge_v1]);
    let rotation = -angle;
    let rad = rotation.to_radians();
    let cos_a = rad.cos();
    let sin_a = rad.sin();
    let pivot = uvs[edge_v0];
    for uv in uvs.iter_mut() {
        let du = uv[0] - pivot[0];
        let dv = uv[1] - pivot[1];
        uv[0] = pivot[0] + du * cos_a - dv * sin_a;
        uv[1] = pivot[1] + du * sin_a + dv * cos_a;
    }
    UvRotateAlignResult {
        rotation_deg: rotation,
        vertex_count: uvs.len(),
    }
}

/// Rotate UV island by an explicit angle around the centroid.
pub fn rotate_island(uvs: &mut [[f32; 2]], angle_deg: f32) {
    if uvs.is_empty() {
        return;
    }
    let mut cu = 0.0f32;
    let mut cv = 0.0f32;
    for uv in uvs.iter() {
        cu += uv[0];
        cv += uv[1];
    }
    let n = uvs.len() as f32;
    cu /= n;
    cv /= n;
    let rad = angle_deg.to_radians();
    let cos_a = rad.cos();
    let sin_a = rad.sin();
    for uv in uvs.iter_mut() {
        let du = uv[0] - cu;
        let dv = uv[1] - cv;
        uv[0] = cu + du * cos_a - dv * sin_a;
        uv[1] = cv + du * sin_a + dv * cos_a;
    }
}

/// Check if a UV edge is approximately horizontal after alignment.
pub fn edge_is_horizontal(uv0: [f32; 2], uv1: [f32; 2], tol: f32) -> bool {
    (uv1[1] - uv0[1]).abs() < tol
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_angle_horizontal_is_zero() {
        let angle = edge_uv_angle_deg([0.0, 0.0], [1.0, 0.0]);
        assert!(angle.abs() < 1e-5 /* horizontal */);
    }

    #[test]
    fn edge_angle_vertical_is_90() {
        let angle = edge_uv_angle_deg([0.0, 0.0], [0.0, 1.0]);
        assert!((angle - 90.0).abs() < 1e-4 /* 90 degrees */);
    }

    #[test]
    fn rotate_align_makes_edge_horizontal() {
        let mut uvs = vec![[0.0f32, 0.0], [0.0, 1.0], [1.0, 0.5]];
        rotate_align_edge(&mut uvs, 0, 1);
        assert!(edge_is_horizontal(uvs[0], uvs[1], 1e-4) /* horizontal after align */);
    }

    #[test]
    fn rotate_align_out_of_bounds_returns_zero() {
        let mut uvs = vec![[0.0f32, 0.0], [1.0, 0.0]];
        let res = rotate_align_edge(&mut uvs, 0, 99);
        assert_eq!(res.vertex_count, 0 /* out of bounds */);
    }

    #[test]
    fn rotate_island_360_is_identity() {
        let mut uvs = vec![[0.5f32, 0.2], [0.8, 0.6], [0.1, 0.9]];
        let original = uvs.clone();
        rotate_island(&mut uvs, 360.0);
        for (a, b) in uvs.iter().zip(original.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-5 /* U same */);
            assert!((a[1] - b[1]).abs() < 1e-5 /* V same */);
        }
    }

    #[test]
    fn rotate_island_empty_no_panic() {
        let mut uvs: Vec<[f32; 2]> = vec![];
        rotate_island(&mut uvs, 45.0);
        assert_eq!(uvs.len(), 0 /* still empty */);
    }

    #[test]
    fn edge_is_horizontal_true() {
        assert!(edge_is_horizontal([0.0, 0.5], [1.0, 0.5], 1e-4) /* same V */);
    }

    #[test]
    fn edge_is_horizontal_false() {
        assert!(!edge_is_horizontal([0.0, 0.0], [1.0, 1.0], 1e-4) /* diagonal */);
    }

    #[test]
    fn rotate_align_returns_count() {
        let mut uvs = vec![[0.0f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let res = rotate_align_edge(&mut uvs, 0, 1);
        assert_eq!(res.vertex_count, 3 /* three UVs */);
    }
}
