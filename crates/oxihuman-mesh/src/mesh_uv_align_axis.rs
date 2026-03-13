// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! UV axis alignment tool — aligns UV islands to U or V axis.

/// Axis to align against.
#[derive(Clone, Copy, PartialEq)]
pub enum AlignAxis {
    U,
    V,
}

/// Result of UV axis alignment.
pub struct UvAlignResult {
    pub aligned_count: usize,
    pub rotation_applied_deg: f32,
}

/// Compute the dominant angle of a UV island (in degrees).
pub fn dominant_angle_deg(uvs: &[[f32; 2]]) -> f32 {
    if uvs.len() < 2 {
        return 0.0;
    }
    let u0 = uvs[0];
    let u1 = uvs[1];
    let du = u1[0] - u0[0];
    let dv = u1[1] - u0[1];
    dv.atan2(du).to_degrees()
}

/// Rotate a UV coordinate by angle_deg around the island centroid.
pub fn rotate_uv(uv: [f32; 2], centre: [f32; 2], angle_deg: f32) -> [f32; 2] {
    let rad = angle_deg.to_radians();
    let cos_a = rad.cos();
    let sin_a = rad.sin();
    let du = uv[0] - centre[0];
    let dv = uv[1] - centre[1];
    [
        centre[0] + du * cos_a - dv * sin_a,
        centre[1] + du * sin_a + dv * cos_a,
    ]
}

/// Compute centroid of UV coordinates.
pub fn uv_centroid(uvs: &[[f32; 2]]) -> [f32; 2] {
    if uvs.is_empty() {
        return [0.0, 0.0];
    }
    let mut su = 0.0f32;
    let mut sv = 0.0f32;
    for uv in uvs {
        su += uv[0];
        sv += uv[1];
    }
    let n = uvs.len() as f32;
    [su / n, sv / n]
}

/// Align UV island to the given axis; modifies uvs in place.
pub fn align_to_axis(uvs: &mut [[f32; 2]], axis: AlignAxis) -> UvAlignResult {
    let angle = dominant_angle_deg(uvs);
    let target = match axis {
        AlignAxis::U => 0.0,
        AlignAxis::V => 90.0,
    };
    let rotation = target - angle;
    let centre = uv_centroid(uvs);
    for uv in uvs.iter_mut() {
        *uv = rotate_uv(*uv, centre, rotation);
    }
    UvAlignResult {
        aligned_count: uvs.len(),
        rotation_applied_deg: rotation,
    }
}

/// Bounding box of UV coordinates [min_u, min_v, max_u, max_v].
pub fn uv_bounding_box(uvs: &[[f32; 2]]) -> [f32; 4] {
    if uvs.is_empty() {
        return [0.0, 0.0, 0.0, 0.0];
    }
    let mut min_u = uvs[0][0];
    let mut min_v = uvs[0][1];
    let mut max_u = uvs[0][0];
    let mut max_v = uvs[0][1];
    for uv in uvs.iter().skip(1) {
        min_u = min_u.min(uv[0]);
        min_v = min_v.min(uv[1]);
        max_u = max_u.max(uv[0]);
        max_v = max_v.max(uv[1]);
    }
    [min_u, min_v, max_u, max_v]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centroid_of_unit_square() {
        let uvs = vec![[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let c = uv_centroid(&uvs);
        assert!((c[0] - 0.5).abs() < 1e-6 /* U centroid */);
        assert!((c[1] - 0.5).abs() < 1e-6 /* V centroid */);
    }

    #[test]
    fn centroid_empty() {
        let c = uv_centroid(&[]);
        assert_eq!(c, [0.0, 0.0] /* zero */);
    }

    #[test]
    fn dominant_angle_horizontal() {
        let uvs = vec![[0.0f32, 0.0], [1.0, 0.0]];
        let angle = dominant_angle_deg(&uvs);
        assert!(angle.abs() < 1e-4 /* horizontal = 0 deg */);
    }

    #[test]
    fn rotate_uv_90_degrees() {
        let uv = [1.0f32, 0.0];
        let centre = [0.0f32, 0.0];
        let rotated = rotate_uv(uv, centre, 90.0);
        assert!((rotated[0] - 0.0).abs() < 1e-5 /* X near 0 */);
        assert!((rotated[1] - 1.0).abs() < 1e-5 /* Y near 1 */);
    }

    #[test]
    fn align_to_u_axis_empty() {
        let mut uvs: Vec<[f32; 2]> = vec![];
        let res = align_to_axis(&mut uvs, AlignAxis::U);
        assert_eq!(res.aligned_count, 0 /* empty */);
    }

    #[test]
    fn align_to_axis_returns_count() {
        let mut uvs = vec![[0.0f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let res = align_to_axis(&mut uvs, AlignAxis::U);
        assert_eq!(res.aligned_count, 3 /* three UVs */);
    }

    #[test]
    fn bounding_box_correct() {
        let uvs = vec![[0.1f32, 0.2], [0.8, 0.9]];
        let bb = uv_bounding_box(&uvs);
        assert!((bb[0] - 0.1).abs() < 1e-6 /* min U */);
        assert!((bb[3] - 0.9).abs() < 1e-6 /* max V */);
    }

    #[test]
    fn bounding_box_empty() {
        let bb = uv_bounding_box(&[]);
        assert_eq!(bb, [0.0, 0.0, 0.0, 0.0] /* empty */);
    }

    #[test]
    fn align_axis_variants_differ() {
        let mut uvs_u = vec![[0.0f32, 0.0], [1.0, 0.5]];
        let mut uvs_v = uvs_u.clone();
        let r_u = align_to_axis(&mut uvs_u, AlignAxis::U);
        let r_v = align_to_axis(&mut uvs_v, AlignAxis::V);
        assert!((r_u.rotation_applied_deg - r_v.rotation_applied_deg).abs() > 1e-6 /* different */);
    }
}
