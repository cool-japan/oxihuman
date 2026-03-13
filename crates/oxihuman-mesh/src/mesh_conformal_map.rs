// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Conformal mapping utilities for mesh UV parameterization.

use std::f32::consts::PI;

/// Result of conformal mapping.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConformalMapResult {
    pub uvs: Vec<[f32; 2]>,
    pub distortion: f32,
}

/// Compute cotangent weight for an edge opposite to a vertex in a triangle.
#[allow(dead_code)]
pub fn cotangent_weight(v: [f32; 3], a: [f32; 3], b: [f32; 3]) -> f32 {
    let va = [a[0] - v[0], a[1] - v[1], a[2] - v[2]];
    let vb = [b[0] - v[0], b[1] - v[1], b[2] - v[2]];
    let dot = va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2];
    let cross = [
        va[1] * vb[2] - va[2] * vb[1],
        va[2] * vb[0] - va[0] * vb[2],
        va[0] * vb[1] - va[1] * vb[0],
    ];
    let cross_len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    if cross_len < 1e-12 {
        return 0.0;
    }
    dot / cross_len
}

/// Map a disk of vertices to a circle boundary.
#[allow(dead_code)]
pub fn map_boundary_to_circle(boundary_count: usize) -> Vec<[f32; 2]> {
    if boundary_count == 0 {
        return vec![];
    }
    let mut uvs = Vec::with_capacity(boundary_count);
    for i in 0..boundary_count {
        let angle = 2.0 * PI * i as f32 / boundary_count as f32;
        uvs.push([0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()]);
    }
    uvs
}

/// Compute conformal energy for a triangle given UVs.
#[allow(dead_code)]
pub fn conformal_energy(uv0: [f32; 2], uv1: [f32; 2], uv2: [f32; 2]) -> f32 {
    let e1 = [uv1[0] - uv0[0], uv1[1] - uv0[1]];
    let e2 = [uv2[0] - uv0[0], uv2[1] - uv0[1]];
    let area = (e1[0] * e2[1] - e1[1] * e2[0]).abs();
    if area < 1e-12 {
        return f32::MAX;
    }
    let l1_sq = e1[0] * e1[0] + e1[1] * e1[1];
    let l2_sq = e2[0] * e2[0] + e2[1] * e2[1];
    (l1_sq + l2_sq) / area
}

/// Compute average conformal distortion for a UV mapping.
#[allow(dead_code)]
pub fn average_distortion(uvs: &[[f32; 2]], indices: &[u32]) -> f32 {
    let tri_count = indices.len() / 3;
    if tri_count == 0 {
        return 0.0;
    }
    let mut sum = 0.0f32;
    let mut valid = 0usize;
    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        if i0 < uvs.len() && i1 < uvs.len() && i2 < uvs.len() {
            let e = conformal_energy(uvs[i0], uvs[i1], uvs[i2]);
            if e < f32::MAX {
                sum += e;
                valid += 1;
            }
        }
    }
    if valid == 0 {
        0.0
    } else {
        sum / valid as f32
    }
}

/// Check if UV point is inside unit square.
#[allow(dead_code)]
pub fn is_in_unit_square(uv: [f32; 2]) -> bool {
    (0.0..=1.0).contains(&uv[0]) && (0.0..=1.0).contains(&uv[1])
}

/// Normalize UVs to `[0,1]` range.
#[allow(dead_code)]
pub fn normalize_uvs(uvs: &mut [[f32; 2]]) {
    if uvs.is_empty() {
        return;
    }
    let mut min = [f32::MAX; 2];
    let mut max = [f32::MIN; 2];
    for uv in uvs.iter() {
        min[0] = min[0].min(uv[0]);
        min[1] = min[1].min(uv[1]);
        max[0] = max[0].max(uv[0]);
        max[1] = max[1].max(uv[1]);
    }
    let range = [(max[0] - min[0]).max(1e-12), (max[1] - min[1]).max(1e-12)];
    for uv in uvs.iter_mut() {
        uv[0] = (uv[0] - min[0]) / range[0];
        uv[1] = (uv[1] - min[1]) / range[1];
    }
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn conformal_map_to_json(result: &ConformalMapResult) -> String {
    format!(
        "{{\"uv_count\":{},\"distortion\":{:.6}}}",
        result.uvs.len(),
        result.distortion
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cotangent_weight() {
        let w = cotangent_weight([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((w).abs() < 1e-5); // 90 degrees -> cot = 0
    }

    #[test]
    fn test_map_boundary_to_circle() {
        let uvs = map_boundary_to_circle(4);
        assert_eq!(uvs.len(), 4);
        // First point should be at (1, 0.5) mapped
        assert!((uvs[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_map_boundary_empty() {
        let uvs = map_boundary_to_circle(0);
        assert!(uvs.is_empty());
    }

    #[test]
    fn test_conformal_energy() {
        let e = conformal_energy([0.0, 0.0], [1.0, 0.0], [0.0, 1.0]);
        assert!(e > 0.0);
        assert!(e < 100.0);
    }

    #[test]
    fn test_conformal_energy_degenerate() {
        let e = conformal_energy([0.0, 0.0], [1.0, 0.0], [2.0, 0.0]);
        assert_eq!(e, f32::MAX);
    }

    #[test]
    fn test_average_distortion() {
        let uvs = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let idx = vec![0, 1, 2];
        let d = average_distortion(&uvs, &idx);
        assert!(d > 0.0);
    }

    #[test]
    fn test_average_distortion_empty() {
        assert!((average_distortion(&[], &[])).abs() < 1e-9);
    }

    #[test]
    fn test_is_in_unit_square() {
        assert!(is_in_unit_square([0.5, 0.5]));
        assert!(!is_in_unit_square([1.5, 0.5]));
    }

    #[test]
    fn test_normalize_uvs() {
        let mut uvs = vec![[2.0, 4.0], [4.0, 8.0]];
        normalize_uvs(&mut uvs);
        assert!((uvs[0][0]).abs() < 1e-6);
        assert!((uvs[1][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let r = ConformalMapResult {
            uvs: vec![[0.0, 0.0]],
            distortion: 1.5,
        };
        let j = conformal_map_to_json(&r);
        assert!(j.contains("\"uv_count\":1"));
    }
}
