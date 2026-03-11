#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Box/triplanar UV projection.

#[allow(dead_code)]
pub fn dominant_axis(n: [f32; 3]) -> u8 {
    let ax = n[0].abs();
    let ay = n[1].abs();
    let az = n[2].abs();
    if ax >= ay && ax >= az {
        0
    } else if ay >= ax && ay >= az {
        1
    } else {
        2
    }
}

#[allow(dead_code)]
pub fn planar_uv_x(p: [f32; 3]) -> [f32; 2] {
    [p[2], p[1]]
}

#[allow(dead_code)]
pub fn planar_uv_y(p: [f32; 3]) -> [f32; 2] {
    [p[0], p[2]]
}

#[allow(dead_code)]
pub fn planar_uv_z(p: [f32; 3]) -> [f32; 2] {
    [p[0], p[1]]
}

#[allow(dead_code)]
pub fn blend_triplanar(p: [f32; 3], n: [f32; 3]) -> [f32; 2] {
    let wx = n[0].abs().max(0.0);
    let wy = n[1].abs().max(0.0);
    let wz = n[2].abs().max(0.0);
    let sum = wx + wy + wz + 1e-10;
    let wx = wx / sum;
    let wy = wy / sum;
    let wz = wz / sum;
    let uvx = planar_uv_x(p);
    let uvy = planar_uv_y(p);
    let uvz = planar_uv_z(p);
    [
        uvx[0] * wx + uvy[0] * wy + uvz[0] * wz,
        uvx[1] * wx + uvy[1] * wy + uvz[1] * wz,
    ]
}

#[allow(dead_code)]
pub fn uvw_box_project(verts: &[[f32; 3]], normals: &[[f32; 3]]) -> Vec<[f32; 2]> {
    verts
        .iter()
        .zip(normals.iter())
        .map(|(v, n)| match dominant_axis(*n) {
            0 => planar_uv_x(*v),
            1 => planar_uv_y(*v),
            _ => planar_uv_z(*v),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dominant_axis_x() {
        assert_eq!(dominant_axis([1.0, 0.0, 0.0]), 0);
    }

    #[test]
    fn dominant_axis_y() {
        assert_eq!(dominant_axis([0.0, 1.0, 0.0]), 1);
    }

    #[test]
    fn dominant_axis_z() {
        assert_eq!(dominant_axis([0.0, 0.0, 1.0]), 2);
    }

    #[test]
    fn planar_uv_x_returns_zy() {
        let uv = planar_uv_x([1.0, 2.0, 3.0]);
        assert!((uv[0] - 3.0).abs() < 1e-6);
        assert!((uv[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn planar_uv_y_returns_xz() {
        let uv = planar_uv_y([1.0, 2.0, 3.0]);
        assert!((uv[0] - 1.0).abs() < 1e-6);
        assert!((uv[1] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn planar_uv_z_returns_xy() {
        let uv = planar_uv_z([1.0, 2.0, 3.0]);
        assert!((uv[0] - 1.0).abs() < 1e-6);
        assert!((uv[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn uvw_box_project_count() {
        let verts = vec![[1.0f32, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let normals = vec![[1.0f32, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let uvs = uvw_box_project(&verts, &normals);
        assert_eq!(uvs.len(), 2);
    }

    #[test]
    fn blend_triplanar_sums_to_finite() {
        let uv = blend_triplanar([1.0, 2.0, 3.0], [0.5, 0.5, 0.0]);
        assert!(uv[0].is_finite());
        assert!(uv[1].is_finite());
    }

    #[test]
    fn uvw_box_project_empty() {
        let uvs = uvw_box_project(&[], &[]);
        assert!(uvs.is_empty());
    }

    #[test]
    fn blend_triplanar_y_dominant() {
        let n = [0.0f32, 1.0, 0.0];
        let p = [2.0f32, 0.0, 5.0];
        let uv = blend_triplanar(p, n);
        let expected = planar_uv_y(p);
        assert!((uv[0] - expected[0]).abs() < 0.01);
        assert!((uv[1] - expected[1]).abs() < 0.01);
    }
}
