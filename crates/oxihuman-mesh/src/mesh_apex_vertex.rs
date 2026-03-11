// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Apex vertex detection for mesh triangles.

/// Result of apex vertex analysis.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ApexResult {
    /// For each triangle, the index of the apex vertex (vertex opposite the longest edge).
    pub apex_indices: Vec<usize>,
    /// Average apex angle in radians.
    pub avg_apex_angle: f32,
}

/// Compute the squared distance between two 3D points.
#[allow(dead_code)]
pub fn dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

/// Find the apex vertex of a triangle (the vertex opposite the longest edge).
/// Returns local index 0, 1, or 2.
#[allow(dead_code)]
pub fn triangle_apex(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> usize {
    let d01 = dist_sq(v0, v1);
    let d12 = dist_sq(v1, v2);
    let d20 = dist_sq(v2, v0);
    if d01 >= d12 && d01 >= d20 {
        2 // opposite edge v0-v1
    } else if d12 >= d01 && d12 >= d20 {
        0 // opposite edge v1-v2
    } else {
        1 // opposite edge v2-v0
    }
}

/// Compute the angle at a given vertex of a triangle (in radians).
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

/// Find the apex vertex angle for a triangle.
#[allow(dead_code)]
pub fn apex_angle(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> f32 {
    let idx = triangle_apex(v0, v1, v2);
    match idx {
        0 => vertex_angle(v0, v1, v2),
        1 => vertex_angle(v1, v0, v2),
        _ => vertex_angle(v2, v0, v1),
    }
}

/// Analyse all triangles in a mesh and find apex vertices.
#[allow(dead_code)]
pub fn find_apex_vertices(positions: &[[f32; 3]], indices: &[u32]) -> ApexResult {
    let tri_count = indices.len() / 3;
    let mut apex_indices = Vec::with_capacity(tri_count);
    let mut angle_sum = 0.0f32;

    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        let local = triangle_apex(positions[i0], positions[i1], positions[i2]);
        let global = match local {
            0 => i0,
            1 => i1,
            _ => i2,
        };
        apex_indices.push(global);
        angle_sum += apex_angle(positions[i0], positions[i1], positions[i2]);
    }

    let avg = if tri_count > 0 {
        angle_sum / tri_count as f32
    } else {
        0.0
    };
    ApexResult {
        apex_indices,
        avg_apex_angle: avg,
    }
}

/// Return the number of apex results.
#[allow(dead_code)]
pub fn apex_count(result: &ApexResult) -> usize {
    result.apex_indices.len()
}

/// Check if a given vertex index appears as an apex vertex in any triangle.
#[allow(dead_code)]
pub fn is_apex_vertex(result: &ApexResult, vertex: usize) -> bool {
    result.apex_indices.contains(&vertex)
}

/// Count how many times a vertex appears as an apex.
#[allow(dead_code)]
pub fn apex_frequency(result: &ApexResult, vertex: usize) -> usize {
    result.apex_indices.iter().filter(|&&v| v == vertex).count()
}

/// Convert apex result to JSON string.
#[allow(dead_code)]
pub fn apex_result_to_json(result: &ApexResult) -> String {
    format!(
        "{{\"apex_count\":{},\"avg_apex_angle\":{:.6}}}",
        result.apex_indices.len(),
        result.avg_apex_angle
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn equilateral_triangle() -> ([f32; 3], [f32; 3], [f32; 3]) {
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.5, 0.866_025_4, 0.0];
        (v0, v1, v2)
    }

    #[test]
    fn test_dist_sq() {
        let d = dist_sq([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
        assert!((d - 25.0).abs() < 1e-6);
    }

    #[test]
    fn test_triangle_apex_right_angle() {
        // right triangle: hypotenuse is longest edge
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [3.0, 0.0, 0.0];
        let v2 = [0.0, 4.0, 0.0];
        // longest edge is v1-v2, apex is v0 (index 0)
        assert_eq!(triangle_apex(v0, v1, v2), 0);
    }

    #[test]
    fn test_vertex_angle_right() {
        let v = [0.0, 0.0, 0.0];
        let a = [1.0, 0.0, 0.0];
        let b = [0.0, 1.0, 0.0];
        let angle = vertex_angle(v, a, b);
        assert!((angle - PI / 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_apex_angle_equilateral() {
        let (v0, v1, v2) = equilateral_triangle();
        let angle = apex_angle(v0, v1, v2);
        // all angles ~60 degrees
        assert!((angle - PI / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_find_apex_vertices() {
        let positions = vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 4.0, 0.0]];
        let indices = vec![0, 1, 2];
        let result = find_apex_vertices(&positions, &indices);
        assert_eq!(result.apex_indices.len(), 1);
        assert_eq!(result.apex_indices[0], 0);
    }

    #[test]
    fn test_apex_count() {
        let result = ApexResult {
            apex_indices: vec![0, 1, 2],
            avg_apex_angle: 1.0,
        };
        assert_eq!(apex_count(&result), 3);
    }

    #[test]
    fn test_is_apex_vertex() {
        let result = ApexResult {
            apex_indices: vec![0, 2],
            avg_apex_angle: 1.0,
        };
        assert!(is_apex_vertex(&result, 0));
        assert!(!is_apex_vertex(&result, 1));
    }

    #[test]
    fn test_apex_frequency() {
        let result = ApexResult {
            apex_indices: vec![0, 0, 1],
            avg_apex_angle: 1.0,
        };
        assert_eq!(apex_frequency(&result, 0), 2);
        assert_eq!(apex_frequency(&result, 1), 1);
    }

    #[test]
    fn test_apex_result_to_json() {
        let result = ApexResult {
            apex_indices: vec![0],
            avg_apex_angle: 1.047,
        };
        let json = apex_result_to_json(&result);
        assert!(json.contains("\"apex_count\":1"));
    }

    #[test]
    fn test_empty_mesh() {
        let result = find_apex_vertices(&[], &[]);
        assert_eq!(apex_count(&result), 0);
        assert!((result.avg_apex_angle - 0.0).abs() < 1e-9);
    }
}
