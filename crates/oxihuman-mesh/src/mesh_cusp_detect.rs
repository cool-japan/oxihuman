// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cusp detection: find vertices where surface normals change abruptly.

use std::f32::consts::PI;

/// A detected cusp vertex.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CuspVertex {
    pub index: usize,
    pub max_angle_deviation: f32,
}

/// Result of cusp detection.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CuspDetectResult {
    pub cusps: Vec<CuspVertex>,
    pub threshold_rad: f32,
}

/// Default cusp angle threshold (90 degrees).
#[allow(dead_code)]
pub fn default_cusp_threshold() -> f32 {
    PI / 2.0
}

/// Compute a face normal (unnormalized).
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

/// Normalize a 3D vector.
#[allow(dead_code)]
pub fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-12 {
        return [0.0; 3];
    }
    [v[0] / l, v[1] / l, v[2] / l]
}

/// Angle between two unit vectors in radians.
#[allow(dead_code)]
pub fn angle_between(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    dot.clamp(-1.0, 1.0).acos()
}

/// Build per-face normals.
#[allow(dead_code)]
pub fn compute_face_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let tri_count = indices.len() / 3;
    (0..tri_count)
        .map(|t| {
            let i0 = indices[t * 3] as usize;
            let i1 = indices[t * 3 + 1] as usize;
            let i2 = indices[t * 3 + 2] as usize;
            normalize3(face_normal_raw(positions[i0], positions[i1], positions[i2]))
        })
        .collect()
}

/// Detect cusps: vertices where adjacent face normals deviate more than threshold.
#[allow(dead_code)]
pub fn detect_cusps(positions: &[[f32; 3]], indices: &[u32], threshold: f32) -> CuspDetectResult {
    let normals = compute_face_normals(positions, indices);
    let tri_count = indices.len() / 3;
    let vert_count = positions.len();

    // For each vertex, collect adjacent face indices
    let mut vert_faces: Vec<Vec<usize>> = vec![Vec::new(); vert_count];
    for t in 0..tri_count {
        for k in 0..3 {
            let vi = indices[t * 3 + k] as usize;
            vert_faces[vi].push(t);
        }
    }

    let mut cusps = Vec::new();
    for (vi, faces) in vert_faces.iter().enumerate() {
        if faces.len() < 2 {
            continue;
        }
        let mut max_dev = 0.0f32;
        for i in 0..faces.len() {
            for j in (i + 1)..faces.len() {
                let angle = angle_between(normals[faces[i]], normals[faces[j]]);
                if angle > max_dev {
                    max_dev = angle;
                }
            }
        }
        if max_dev > threshold {
            cusps.push(CuspVertex {
                index: vi,
                max_angle_deviation: max_dev,
            });
        }
    }

    CuspDetectResult {
        cusps,
        threshold_rad: threshold,
    }
}

/// Number of detected cusps.
#[allow(dead_code)]
pub fn cusp_count(result: &CuspDetectResult) -> usize {
    result.cusps.len()
}

/// Check if a vertex is a cusp.
#[allow(dead_code)]
pub fn is_cusp(result: &CuspDetectResult, vertex: usize) -> bool {
    result.cusps.iter().any(|c| c.index == vertex)
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn cusp_result_to_json(result: &CuspDetectResult) -> String {
    format!(
        "{{\"cusp_count\":{},\"threshold_rad\":{:.6}}}",
        result.cusps.len(),
        result.threshold_rad
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize3() {
        let n = normalize3([3.0, 0.0, 0.0]);
        assert!((n[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize3_zero() {
        let n = normalize3([0.0, 0.0, 0.0]);
        assert!((n[0]).abs() < 1e-6);
    }

    #[test]
    fn test_angle_between_same() {
        let a = angle_between([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(a.abs() < 1e-6);
    }

    #[test]
    fn test_angle_between_perpendicular() {
        let a = angle_between([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((a - PI / 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_face_normal_raw() {
        let n = face_normal_raw([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(n[2] > 0.0);
    }

    #[test]
    fn test_detect_cusps_flat() {
        // Two coplanar triangles => no cusps
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [1.5, 0.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 1, 3, 2];
        let result = detect_cusps(&pos, &idx, default_cusp_threshold());
        assert_eq!(cusp_count(&result), 0);
    }

    #[test]
    fn test_detect_cusps_sharp() {
        // Two triangles at a sharp angle sharing edge (0,1)
        // Face 0 normal points up (+Y), face 1 normal points forward (+Z)
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.0, -1.0],
        ];
        let idx = vec![0, 1, 2, 1, 0, 3];
        let result = detect_cusps(&pos, &idx, 0.1);
        assert!(cusp_count(&result) > 0);
    }

    #[test]
    fn test_cusp_result_to_json() {
        let result = CuspDetectResult {
            cusps: vec![],
            threshold_rad: 1.5,
        };
        let json = cusp_result_to_json(&result);
        assert!(json.contains("\"cusp_count\":0"));
    }

    #[test]
    fn test_is_cusp() {
        let result = CuspDetectResult {
            cusps: vec![CuspVertex {
                index: 3,
                max_angle_deviation: 2.0,
            }],
            threshold_rad: 1.0,
        };
        assert!(is_cusp(&result, 3));
        assert!(!is_cusp(&result, 0));
    }

    #[test]
    fn test_empty_mesh() {
        let result = detect_cusps(&[], &[], default_cusp_threshold());
        assert_eq!(cusp_count(&result), 0);
    }
}
