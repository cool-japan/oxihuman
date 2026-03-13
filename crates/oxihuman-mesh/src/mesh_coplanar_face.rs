// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Detect coplanar faces in a triangle mesh.

use std::f32::consts::FRAC_PI_4;

/// A group of coplanar faces.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CoplanarGroup {
    pub face_indices: Vec<usize>,
    pub normal: [f32; 3],
}

/// Result of coplanar detection.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CoplanarResult {
    pub groups: Vec<CoplanarGroup>,
    pub threshold: f32,
}

/// Compute face normal (unnormalized).
#[allow(dead_code)]
pub fn face_normal_raw(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ]
}

/// Normalize vector.
#[allow(dead_code)]
pub fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        return [0.0; 3];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Dot product.
#[allow(dead_code)]
pub fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Check if two normals are coplanar within a threshold angle (cosine).
#[allow(dead_code)]
pub fn are_coplanar(n1: [f32; 3], n2: [f32; 3], cos_thresh: f32) -> bool {
    dot3(n1, n2).abs() >= cos_thresh
}

/// Detect coplanar face groups.
#[allow(dead_code)]
pub fn detect_coplanar_faces(
    positions: &[[f32; 3]],
    indices: &[u32],
    angle_threshold: f32,
) -> CoplanarResult {
    let cos_thresh = angle_threshold.cos();
    let tri_count = indices.len() / 3;
    let mut normals = Vec::with_capacity(tri_count);

    #[allow(clippy::needless_range_loop)]
    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        normals.push(normalize3(face_normal_raw(
            positions[i0],
            positions[i1],
            positions[i2],
        )));
    }

    let mut assigned = vec![false; tri_count];
    let mut groups = Vec::new();

    #[allow(clippy::needless_range_loop)]
    for t in 0..tri_count {
        if assigned[t] {
            continue;
        }
        assigned[t] = true;
        let mut group_faces = vec![t];
        for t2 in (t + 1)..tri_count {
            if assigned[t2] {
                continue;
            }
            if are_coplanar(normals[t], normals[t2], cos_thresh) {
                assigned[t2] = true;
                group_faces.push(t2);
            }
        }
        groups.push(CoplanarGroup {
            face_indices: group_faces,
            normal: normals[t],
        });
    }

    CoplanarResult {
        groups,
        threshold: angle_threshold,
    }
}

/// Group count.
#[allow(dead_code)]
pub fn group_count(r: &CoplanarResult) -> usize {
    r.groups.len()
}

/// Largest group size.
#[allow(dead_code)]
pub fn largest_group(r: &CoplanarResult) -> usize {
    r.groups
        .iter()
        .map(|g| g.face_indices.len())
        .max()
        .unwrap_or(0)
}

/// Total faces across all groups.
#[allow(dead_code)]
pub fn total_grouped_faces(r: &CoplanarResult) -> usize {
    r.groups.iter().map(|g| g.face_indices.len()).sum()
}

/// Default threshold (quarter pi).
#[allow(dead_code)]
pub fn default_threshold() -> f32 {
    FRAC_PI_4
}

/// Export to JSON.
#[allow(dead_code)]
pub fn coplanar_to_json(r: &CoplanarResult) -> String {
    format!(
        "{{\"groups\":{},\"largest\":{}}}",
        group_count(r),
        largest_group(r)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_quad() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    #[test]
    fn test_dot3() {
        assert!((dot3([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize3() {
        let n = normalize3([3.0, 0.0, 0.0]);
        assert!((n[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_are_coplanar() {
        assert!(are_coplanar([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], 0.99));
    }

    #[test]
    fn test_detect_flat_quad() {
        let (pos, idx) = flat_quad();
        let r = detect_coplanar_faces(&pos, &idx, 0.1);
        assert_eq!(group_count(&r), 1);
    }

    #[test]
    fn test_total_grouped_faces() {
        let (pos, idx) = flat_quad();
        let r = detect_coplanar_faces(&pos, &idx, 0.1);
        assert_eq!(total_grouped_faces(&r), 2);
    }

    #[test]
    fn test_largest_group() {
        let (pos, idx) = flat_quad();
        let r = detect_coplanar_faces(&pos, &idx, 0.1);
        assert_eq!(largest_group(&r), 2);
    }

    #[test]
    fn test_empty() {
        let r = detect_coplanar_faces(&[], &[], 0.1);
        assert_eq!(group_count(&r), 0);
    }

    #[test]
    fn test_default_threshold() {
        let t = default_threshold();
        assert!((t - FRAC_PI_4).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let r = CoplanarResult {
            groups: vec![],
            threshold: 0.1,
        };
        assert!(coplanar_to_json(&r).contains("\"groups\":0"));
    }

    #[test]
    fn test_face_normal_raw() {
        let n = face_normal_raw([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(n[2] > 0.0);
    }
}
