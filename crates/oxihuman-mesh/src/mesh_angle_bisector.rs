// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Angle bisector computation for mesh triangles and edges.

/// Result of angle bisector analysis for a mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AngleBisectorResult {
    /// Per-triangle bisector directions from each vertex.
    pub bisectors: Vec<[f32; 3]>,
    /// Average bisector angle in radians.
    pub avg_angle: f32,
}

/// Normalize a 3D vector, returning zero vector if length is near zero.
#[allow(dead_code)]
pub fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        return [0.0, 0.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Compute the angle at vertex `v` between edges to `a` and `b`.
#[allow(dead_code)]
pub fn vertex_angle(v: [f32; 3], a: [f32; 3], b: [f32; 3]) -> f32 {
    let va = [a[0] - v[0], a[1] - v[1], a[2] - v[2]];
    let vb = [b[0] - v[0], b[1] - v[1], b[2] - v[2]];
    let dot = va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2];
    let la = (va[0] * va[0] + va[1] * va[1] + va[2] * va[2]).sqrt();
    let lb = (vb[0] * vb[0] + vb[1] * vb[1] + vb[2] * vb[2]).sqrt();
    let denom = la * lb;
    if denom < 1e-12 {
        return 0.0;
    }
    (dot / denom).clamp(-1.0, 1.0).acos()
}

/// Compute the bisector direction at vertex `v` between edges to `a` and `b`.
#[allow(dead_code)]
pub fn angle_bisector(v: [f32; 3], a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    let na = normalize3([a[0] - v[0], a[1] - v[1], a[2] - v[2]]);
    let nb = normalize3([b[0] - v[0], b[1] - v[1], b[2] - v[2]]);
    normalize3([na[0] + nb[0], na[1] + nb[1], na[2] + nb[2]])
}

/// Compute bisectors for all triangles at vertex 0.
#[allow(dead_code)]
pub fn compute_bisectors(positions: &[[f32; 3]], indices: &[u32]) -> AngleBisectorResult {
    let tri_count = indices.len() / 3;
    let mut bisectors = Vec::with_capacity(tri_count);
    let mut angle_sum = 0.0f32;

    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        let b = angle_bisector(positions[i0], positions[i1], positions[i2]);
        bisectors.push(b);
        angle_sum += vertex_angle(positions[i0], positions[i1], positions[i2]);
    }

    let avg = if tri_count > 0 {
        angle_sum / tri_count as f32
    } else {
        0.0
    };
    AngleBisectorResult {
        bisectors,
        avg_angle: avg,
    }
}

/// Dot product of two 3D vectors.
#[allow(dead_code)]
pub fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Length of a 3D vector.
#[allow(dead_code)]
pub fn vec_length(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Check if bisector is valid (non-zero length).
#[allow(dead_code)]
pub fn is_valid_bisector(b: [f32; 3]) -> bool {
    vec_length(b) > 1e-9
}

/// Number of bisectors in a result.
#[allow(dead_code)]
pub fn bisector_count(result: &AngleBisectorResult) -> usize {
    result.bisectors.len()
}

/// Convert result to JSON.
#[allow(dead_code)]
pub fn bisector_result_to_json(result: &AngleBisectorResult) -> String {
    format!(
        "{{\"bisector_count\":{},\"avg_angle\":{:.6}}}",
        result.bisectors.len(),
        result.avg_angle
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_normalize3() {
        let n = normalize3([3.0, 0.0, 0.0]);
        assert!((n[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize3_zero() {
        let n = normalize3([0.0, 0.0, 0.0]);
        assert!((n[0]).abs() < 1e-9);
    }

    #[test]
    fn test_vertex_angle_right() {
        let angle = vertex_angle([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((angle - PI / 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_angle_bisector_right_angle() {
        let b = angle_bisector([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        // bisector at 45 degrees
        assert!((b[0] - b[1]).abs() < 1e-5);
        assert!(b[0] > 0.0);
    }

    #[test]
    fn test_compute_bisectors_single() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = vec![0, 1, 2];
        let result = compute_bisectors(&positions, &indices);
        assert_eq!(bisector_count(&result), 1);
    }

    #[test]
    fn test_compute_bisectors_empty() {
        let result = compute_bisectors(&[], &[]);
        assert_eq!(bisector_count(&result), 0);
        assert!((result.avg_angle).abs() < 1e-9);
    }

    #[test]
    fn test_dot3() {
        let d = dot3([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((d).abs() < 1e-9);
    }

    #[test]
    fn test_is_valid_bisector() {
        assert!(is_valid_bisector([1.0, 0.0, 0.0]));
        assert!(!is_valid_bisector([0.0, 0.0, 0.0]));
    }

    #[test]
    fn test_bisector_result_to_json() {
        let result = AngleBisectorResult {
            bisectors: vec![[1.0, 0.0, 0.0]],
            avg_angle: 1.0,
        };
        let json = bisector_result_to_json(&result);
        assert!(json.contains("\"bisector_count\":1"));
    }

    #[test]
    fn test_vec_length() {
        let l = vec_length([3.0, 4.0, 0.0]);
        assert!((l - 5.0).abs() < 1e-6);
    }
}
