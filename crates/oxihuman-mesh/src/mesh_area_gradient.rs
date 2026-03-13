// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Per-face area gradient computation for mesh analysis.

use std::f32::consts::FRAC_1_SQRT_2;

/// Result of area gradient computation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AreaGradientResult {
    pub face_areas: Vec<f32>,
    pub gradients: Vec<[f32; 3]>,
}

/// Compute triangle area from three vertices.
#[allow(dead_code)]
pub fn triangle_area(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> f32 {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let cx = e1[1] * e2[2] - e1[2] * e2[1];
    let cy = e1[2] * e2[0] - e1[0] * e2[2];
    let cz = e1[0] * e2[1] - e1[1] * e2[0];
    0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
}

/// Cross product of two 3D vectors.
#[allow(dead_code)]
pub fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Normalize a 3D vector; returns zero if length is near zero.
#[allow(dead_code)]
pub fn safe_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        [0.0; 3]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

/// Compute per-face area gradient (direction of steepest area increase per vertex perturbation).
#[allow(dead_code)]
pub fn compute_area_gradients(positions: &[[f32; 3]], indices: &[u32]) -> AreaGradientResult {
    let tri_count = indices.len() / 3;
    let mut face_areas = Vec::with_capacity(tri_count);
    let mut gradients = Vec::with_capacity(tri_count);

    #[allow(clippy::needless_range_loop)]
    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        let v0 = positions[i0];
        let v1 = positions[i1];
        let v2 = positions[i2];
        let area = triangle_area(v0, v1, v2);
        face_areas.push(area);
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let n = cross3(e1, e2);
        gradients.push(safe_normalize(n));
    }

    AreaGradientResult {
        face_areas,
        gradients,
    }
}

/// Face count in result.
#[allow(dead_code)]
pub fn gradient_face_count(r: &AreaGradientResult) -> usize {
    r.face_areas.len()
}

/// Total mesh area from gradient result.
#[allow(dead_code)]
pub fn total_area(r: &AreaGradientResult) -> f32 {
    r.face_areas.iter().sum()
}

/// Max face area.
#[allow(dead_code)]
pub fn max_face_area(r: &AreaGradientResult) -> f32 {
    r.face_areas.iter().cloned().fold(0.0_f32, f32::max)
}

/// Min face area.
#[allow(dead_code)]
pub fn min_face_area(r: &AreaGradientResult) -> f32 {
    r.face_areas.iter().cloned().fold(f32::MAX, f32::min)
}

/// Average face area.
#[allow(dead_code)]
pub fn avg_face_area(r: &AreaGradientResult) -> f32 {
    if r.face_areas.is_empty() {
        return 0.0;
    }
    total_area(r) / r.face_areas.len() as f32
}

/// Get gradient for a specific face.
#[allow(dead_code)]
pub fn get_gradient(r: &AreaGradientResult, face: usize) -> Option<[f32; 3]> {
    r.gradients.get(face).copied()
}

/// Export result to JSON string.
#[allow(dead_code)]
pub fn area_gradient_to_json(r: &AreaGradientResult) -> String {
    format!(
        "{{\"faces\":{},\"total_area\":{:.6},\"frac_1_sqrt_2\":{:.6}}}",
        gradient_face_count(r),
        total_area(r),
        FRAC_1_SQRT_2
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        (
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0, 1, 2],
        )
    }

    #[test]
    fn test_triangle_area() {
        let a = triangle_area([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((a - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_cross3() {
        let c = cross3([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((c[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_safe_normalize_zero() {
        let n = safe_normalize([0.0, 0.0, 0.0]);
        assert!((n[0]).abs() < 1e-6);
    }

    #[test]
    fn test_compute_area_gradients() {
        let (pos, idx) = unit_tri();
        let r = compute_area_gradients(&pos, &idx);
        assert_eq!(gradient_face_count(&r), 1);
    }

    #[test]
    fn test_total_area() {
        let (pos, idx) = unit_tri();
        let r = compute_area_gradients(&pos, &idx);
        assert!((total_area(&r) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_max_min_area() {
        let (pos, idx) = unit_tri();
        let r = compute_area_gradients(&pos, &idx);
        assert!((max_face_area(&r) - min_face_area(&r)).abs() < 1e-6);
    }

    #[test]
    fn test_avg_face_area() {
        let (pos, idx) = unit_tri();
        let r = compute_area_gradients(&pos, &idx);
        assert!((avg_face_area(&r) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_get_gradient() {
        let (pos, idx) = unit_tri();
        let r = compute_area_gradients(&pos, &idx);
        assert!(get_gradient(&r, 0).is_some());
        assert!(get_gradient(&r, 99).is_none());
    }

    #[test]
    fn test_to_json() {
        let (pos, idx) = unit_tri();
        let r = compute_area_gradients(&pos, &idx);
        let j = area_gradient_to_json(&r);
        assert!(j.contains("\"faces\":1"));
    }

    #[test]
    fn test_empty() {
        let r = compute_area_gradients(&[], &[]);
        assert_eq!(gradient_face_count(&r), 0);
        assert!((avg_face_area(&r)).abs() < 1e-6);
    }
}
