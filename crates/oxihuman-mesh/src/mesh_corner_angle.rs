// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Corner angle computation for mesh triangles.

use std::f32::consts::PI;

/// Corner angle result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CornerAngleResult {
    pub angles: Vec<[f32; 3]>,
}

/// Compute angle at a corner given three positions (angle at v0).
#[allow(dead_code)]
pub fn corner_angle(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> f32 {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let dot = e1[0] * e2[0] + e1[1] * e2[1] + e1[2] * e2[2];
    let l1 = (e1[0] * e1[0] + e1[1] * e1[1] + e1[2] * e1[2]).sqrt();
    let l2 = (e2[0] * e2[0] + e2[1] * e2[1] + e2[2] * e2[2]).sqrt();
    let denom = l1 * l2;
    if denom < 1e-12 {
        return 0.0;
    }
    (dot / denom).clamp(-1.0, 1.0).acos()
}

/// Compute all three corner angles for a triangle.
#[allow(dead_code)]
pub fn triangle_angles(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    [
        corner_angle(v0, v1, v2),
        corner_angle(v1, v2, v0),
        corner_angle(v2, v0, v1),
    ]
}

/// Compute corner angles for all triangles.
#[allow(dead_code)]
pub fn compute_corner_angles(positions: &[[f32; 3]], indices: &[u32]) -> CornerAngleResult {
    let tri_count = indices.len() / 3;
    let mut angles = Vec::with_capacity(tri_count);
    #[allow(clippy::needless_range_loop)]
    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        angles.push(triangle_angles(positions[i0], positions[i1], positions[i2]));
    }
    CornerAngleResult { angles }
}

/// Triangle count.
#[allow(dead_code)]
pub fn angle_face_count(r: &CornerAngleResult) -> usize {
    r.angles.len()
}

/// Minimum corner angle across all faces.
#[allow(dead_code)]
pub fn min_corner_angle(r: &CornerAngleResult) -> f32 {
    r.angles
        .iter()
        .flat_map(|a| a.iter())
        .cloned()
        .fold(f32::MAX, f32::min)
}

/// Maximum corner angle across all faces.
#[allow(dead_code)]
pub fn max_corner_angle(r: &CornerAngleResult) -> f32 {
    r.angles
        .iter()
        .flat_map(|a| a.iter())
        .cloned()
        .fold(0.0_f32, f32::max)
}

/// Check if all angles sum to PI per triangle.
#[allow(dead_code)]
pub fn validate_angle_sums(r: &CornerAngleResult, tol: f32) -> bool {
    r.angles
        .iter()
        .all(|a| (a[0] + a[1] + a[2] - PI).abs() < tol)
}

/// Average minimum angle across faces.
#[allow(dead_code)]
pub fn avg_min_angle(r: &CornerAngleResult) -> f32 {
    if r.angles.is_empty() {
        return 0.0;
    }
    let sum: f32 = r.angles.iter().map(|a| a[0].min(a[1]).min(a[2])).sum();
    sum / r.angles.len() as f32
}

/// Export to JSON.
#[allow(dead_code)]
pub fn corner_angle_to_json(r: &CornerAngleResult) -> String {
    format!(
        "{{\"faces\":{},\"min_angle\":{:.6},\"max_angle\":{:.6}}}",
        angle_face_count(r),
        min_corner_angle(r),
        max_corner_angle(r)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn equilateral() -> (Vec<[f32; 3]>, Vec<u32>) {
        let h = (3.0_f32).sqrt() / 2.0;
        (
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, h, 0.0]],
            vec![0, 1, 2],
        )
    }

    #[test]
    fn test_corner_angle_right() {
        let a = corner_angle([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((a - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
    }

    #[test]
    fn test_triangle_angles_sum() {
        let a = triangle_angles([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((a[0] + a[1] + a[2] - PI).abs() < 1e-4);
    }

    #[test]
    fn test_equilateral_angles() {
        let (pos, idx) = equilateral();
        let r = compute_corner_angles(&pos, &idx);
        let expected = PI / 3.0;
        for a in &r.angles[0] {
            assert!((a - expected).abs() < 1e-4);
        }
    }

    #[test]
    fn test_face_count() {
        let (pos, idx) = equilateral();
        let r = compute_corner_angles(&pos, &idx);
        assert_eq!(angle_face_count(&r), 1);
    }

    #[test]
    fn test_min_max() {
        let (pos, idx) = equilateral();
        let r = compute_corner_angles(&pos, &idx);
        assert!((min_corner_angle(&r) - max_corner_angle(&r)).abs() < 1e-4);
    }

    #[test]
    fn test_validate_angle_sums() {
        let (pos, idx) = equilateral();
        let r = compute_corner_angles(&pos, &idx);
        assert!(validate_angle_sums(&r, 1e-3));
    }

    #[test]
    fn test_avg_min_angle() {
        let (pos, idx) = equilateral();
        let r = compute_corner_angles(&pos, &idx);
        assert!(avg_min_angle(&r) > 0.0);
    }

    #[test]
    fn test_empty() {
        let r = compute_corner_angles(&[], &[]);
        assert_eq!(angle_face_count(&r), 0);
        assert!((avg_min_angle(&r)).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let (pos, idx) = equilateral();
        let r = compute_corner_angles(&pos, &idx);
        let j = corner_angle_to_json(&r);
        assert!(j.contains("\"faces\":1"));
    }

    #[test]
    fn test_degenerate_angle() {
        let a = corner_angle([0.0; 3], [0.0; 3], [0.0; 3]);
        assert!((a).abs() < 1e-6);
    }
}
