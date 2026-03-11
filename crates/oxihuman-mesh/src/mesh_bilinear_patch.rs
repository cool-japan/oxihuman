// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Bilinear patch evaluation and tessellation.

/// A bilinear patch defined by four corner points.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BilinearPatch {
    pub corners: [[f32; 3]; 4],
}

/// Create a new bilinear patch from four corners (p00, p10, p01, p11).
#[allow(dead_code)]
pub fn new_bilinear_patch(
    p00: [f32; 3],
    p10: [f32; 3],
    p01: [f32; 3],
    p11: [f32; 3],
) -> BilinearPatch {
    BilinearPatch {
        corners: [p00, p10, p01, p11],
    }
}

/// Evaluate the patch at parameters (u, v) in [0, 1].
#[allow(dead_code)]
pub fn evaluate_patch(patch: &BilinearPatch, u: f32, v: f32) -> [f32; 3] {
    let c = &patch.corners;
    let one_u = 1.0 - u;
    let one_v = 1.0 - v;
    [
        one_u * one_v * c[0][0] + u * one_v * c[1][0] + one_u * v * c[2][0] + u * v * c[3][0],
        one_u * one_v * c[0][1] + u * one_v * c[1][1] + one_u * v * c[2][1] + u * v * c[3][1],
        one_u * one_v * c[0][2] + u * one_v * c[1][2] + one_u * v * c[2][2] + u * v * c[3][2],
    ]
}

/// Compute the patch center (u=0.5, v=0.5).
#[allow(dead_code)]
pub fn patch_center(patch: &BilinearPatch) -> [f32; 3] {
    evaluate_patch(patch, 0.5, 0.5)
}

/// Compute partial derivative with respect to u.
#[allow(dead_code)]
pub fn patch_du(patch: &BilinearPatch, _u: f32, v: f32) -> [f32; 3] {
    let c = &patch.corners;
    let one_v = 1.0 - v;
    [
        one_v * (c[1][0] - c[0][0]) + v * (c[3][0] - c[2][0]),
        one_v * (c[1][1] - c[0][1]) + v * (c[3][1] - c[2][1]),
        one_v * (c[1][2] - c[0][2]) + v * (c[3][2] - c[2][2]),
    ]
}

/// Compute partial derivative with respect to v.
#[allow(dead_code)]
pub fn patch_dv(patch: &BilinearPatch, u: f32, _v: f32) -> [f32; 3] {
    let c = &patch.corners;
    let one_u = 1.0 - u;
    [
        one_u * (c[2][0] - c[0][0]) + u * (c[3][0] - c[1][0]),
        one_u * (c[2][1] - c[0][1]) + u * (c[3][1] - c[1][1]),
        one_u * (c[2][2] - c[0][2]) + u * (c[3][2] - c[1][2]),
    ]
}

/// Tessellate a bilinear patch into a triangle mesh.
#[allow(dead_code)]
pub fn tessellate_patch(patch: &BilinearPatch, subdivisions: usize) -> (Vec<[f32; 3]>, Vec<u32>) {
    let n = subdivisions.max(1);
    let step = 1.0 / n as f32;
    let mut positions = Vec::new();
    for j in 0..=n {
        for i in 0..=n {
            let u = i as f32 * step;
            let v = j as f32 * step;
            positions.push(evaluate_patch(patch, u, v));
        }
    }
    let mut indices = Vec::new();
    let stride = n + 1;
    for j in 0..n {
        for i in 0..n {
            let tl = (j * stride + i) as u32;
            let tr = tl + 1;
            let bl = ((j + 1) * stride + i) as u32;
            let br = bl + 1;
            indices.extend_from_slice(&[tl, bl, tr, tr, bl, br]);
        }
    }
    (positions, indices)
}

/// Approximate patch area using a grid sample.
#[allow(dead_code)]
pub fn patch_area_approx(patch: &BilinearPatch, samples: usize) -> f32 {
    let n = samples.max(1);
    let (positions, indices) = tessellate_patch(patch, n);
    let mut area = 0.0f32;
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        let v0 = positions[i0];
        let v1 = positions[i1];
        let v2 = positions[i2];
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let cx = e1[1] * e2[2] - e1[2] * e2[1];
        let cy = e1[2] * e2[0] - e1[0] * e2[2];
        let cz = e1[0] * e2[1] - e1[1] * e2[0];
        area += 0.5 * (cx * cx + cy * cy + cz * cz).sqrt();
    }
    area
}

/// Convert patch to JSON.
#[allow(dead_code)]
pub fn patch_to_json(patch: &BilinearPatch) -> String {
    let c = patch_center(patch);
    format!("{{\"center\":[{:.4},{:.4},{:.4}]}}", c[0], c[1], c[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_patch() -> BilinearPatch {
        new_bilinear_patch(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        )
    }

    #[test]
    fn test_evaluate_corner() {
        let p = unit_patch();
        let v = evaluate_patch(&p, 0.0, 0.0);
        assert!((v[0]).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_opposite_corner() {
        let p = unit_patch();
        let v = evaluate_patch(&p, 1.0, 1.0);
        assert!((v[0] - 1.0).abs() < 1e-6);
        assert!((v[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_center() {
        let p = unit_patch();
        let c = patch_center(&p);
        assert!((c[0] - 0.5).abs() < 1e-6);
        assert!((c[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_du() {
        let p = unit_patch();
        let du = patch_du(&p, 0.5, 0.5);
        assert!((du[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_dv() {
        let p = unit_patch();
        let dv = patch_dv(&p, 0.5, 0.5);
        assert!((dv[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_tessellate() {
        let p = unit_patch();
        let (verts, idx) = tessellate_patch(&p, 2);
        assert_eq!(verts.len(), 9); // 3x3
        assert_eq!(idx.len(), 24); // 4 quads * 6 indices
    }

    #[test]
    fn test_area_approx() {
        let p = unit_patch();
        let area = patch_area_approx(&p, 4);
        assert!((area - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_patch_to_json() {
        let p = unit_patch();
        let j = patch_to_json(&p);
        assert!(j.contains("\"center\""));
    }

    #[test]
    fn test_tessellate_min() {
        let p = unit_patch();
        let (verts, idx) = tessellate_patch(&p, 0);
        assert_eq!(verts.len(), 4);
        assert_eq!(idx.len(), 6);
    }

    #[test]
    fn test_nonplanar_patch() {
        let p = new_bilinear_patch(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
        );
        let c = patch_center(&p);
        assert!((c[2] - 0.25).abs() < 1e-6);
    }
}
