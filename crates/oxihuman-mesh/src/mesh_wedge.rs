// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Wedge (triangular prism) mesh primitive and per-wedge attribute storage.

/// A wedge (triangular prism) mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WedgeMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
}

/// Generate a wedge (triangular prism) with base triangle and given height.
#[allow(dead_code)]
pub fn generate_wedge(
    base_v0: [f32; 3],
    base_v1: [f32; 3],
    base_v2: [f32; 3],
    height: f32,
) -> WedgeMesh {
    let top_v0 = [base_v0[0], base_v0[1] + height, base_v0[2]];
    let top_v1 = [base_v1[0], base_v1[1] + height, base_v1[2]];
    let top_v2 = [base_v2[0], base_v2[1] + height, base_v2[2]];

    let positions = vec![base_v0, base_v1, base_v2, top_v0, top_v1, top_v2];
    // bottom, top, and 3 side quads (each = 2 triangles)
    let indices = vec![
        0, 2, 1, // bottom (reversed winding)
        3, 4, 5, // top
        0, 1, 4, 0, 4, 3, // side 0-1
        1, 2, 5, 1, 5, 4, // side 1-2
        2, 0, 3, 2, 3, 5, // side 2-0
    ];
    let normals = vec![[0.0; 3]; 6];
    WedgeMesh {
        positions,
        indices,
        normals,
    }
}

/// Count vertices in a wedge mesh.
#[allow(dead_code)]
pub fn wedge_vertex_count(w: &WedgeMesh) -> usize {
    w.positions.len()
}

/// Count triangles in a wedge mesh.
#[allow(dead_code)]
pub fn wedge_triangle_count(w: &WedgeMesh) -> usize {
    w.indices.len() / 3
}

/// Compute the approximate volume of the wedge (base area × height).
#[allow(dead_code)]
pub fn wedge_volume(base_v0: [f32; 3], base_v1: [f32; 3], base_v2: [f32; 3], height: f32) -> f32 {
    let e1 = [
        base_v1[0] - base_v0[0],
        base_v1[1] - base_v0[1],
        base_v1[2] - base_v0[2],
    ];
    let e2 = [
        base_v2[0] - base_v0[0],
        base_v2[1] - base_v0[1],
        base_v2[2] - base_v0[2],
    ];
    let cross = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let area = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt() * 0.5;
    area * height.abs()
}

/// Check that all indices are within bounds.
#[allow(dead_code)]
pub fn wedge_indices_valid(w: &WedgeMesh) -> bool {
    let n = w.positions.len() as u32;
    w.indices.iter().all(|&i| i < n)
}

/// Compute the angle (radians) at a corner of the base triangle.
#[allow(dead_code)]
pub fn wedge_base_angle(base_v0: [f32; 3], base_v1: [f32; 3], base_v2: [f32; 3]) -> f32 {
    let e1 = [
        base_v1[0] - base_v0[0],
        base_v1[1] - base_v0[1],
        base_v1[2] - base_v0[2],
    ];
    let e2 = [
        base_v2[0] - base_v0[0],
        base_v2[1] - base_v0[1],
        base_v2[2] - base_v0[2],
    ];
    let dot = e1[0] * e2[0] + e1[1] * e2[1] + e1[2] * e2[2];
    let la = (e1[0] * e1[0] + e1[1] * e1[1] + e1[2] * e1[2]).sqrt();
    let lb = (e2[0] * e2[0] + e2[1] * e2[1] + e2[2] * e2[2]).sqrt();
    if la * lb < 1e-12 {
        return 0.0;
    }
    (dot / (la * lb)).clamp(-1.0, 1.0).acos()
}

/// Serialize wedge info to JSON.
#[allow(dead_code)]
pub fn wedge_to_json(w: &WedgeMesh) -> String {
    format!(
        "{{\"vertices\":{},\"triangles\":{}}}",
        wedge_vertex_count(w),
        wedge_triangle_count(w)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_wedge() -> WedgeMesh {
        generate_wedge([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0)
    }

    #[test]
    fn test_wedge_vertex_count() {
        assert_eq!(wedge_vertex_count(&unit_wedge()), 6);
    }

    #[test]
    fn test_wedge_triangle_count() {
        // 2 + 3*2 = 8
        assert_eq!(wedge_triangle_count(&unit_wedge()), 8);
    }

    #[test]
    fn test_wedge_volume_positive() {
        let v = wedge_volume([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0);
        assert!(v > 0.0);
    }

    #[test]
    fn test_wedge_indices_valid() {
        assert!(wedge_indices_valid(&unit_wedge()));
    }

    #[test]
    fn test_wedge_base_angle_right() {
        let a = wedge_base_angle([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        assert!((a - std::f32::consts::PI * 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_wedge_to_json() {
        let j = wedge_to_json(&unit_wedge());
        assert!(j.contains("vertices"));
    }

    #[test]
    fn test_wedge_negative_height() {
        let v = wedge_volume([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], -1.0);
        assert!(v > 0.0);
    }

    #[test]
    fn test_wedge_zero_height_zero_volume() {
        let v = wedge_volume([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.0);
        assert!(v.abs() < 1e-6);
    }

    #[test]
    fn test_wedge_positions_all_finite() {
        for p in &unit_wedge().positions {
            for &c in p {
                assert!(c.is_finite());
            }
        }
    }
}
