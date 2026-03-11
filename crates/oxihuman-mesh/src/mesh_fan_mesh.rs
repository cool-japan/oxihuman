// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fan mesh generation: create triangle fans from a center point.

use std::f32::consts::TAU;

/// Result of fan mesh generation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FanMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// Generate a planar triangle fan (disk) centered at `center` with `segments` segments.
#[allow(dead_code)]
pub fn generate_fan(center: [f32; 3], radius: f32, segments: usize) -> FanMesh {
    let n = segments.max(3);
    let mut positions = Vec::with_capacity(n + 1);
    positions.push(center);
    for i in 0..n {
        let angle = TAU * i as f32 / n as f32;
        positions.push([
            center[0] + radius * angle.cos(),
            center[1] + radius * angle.sin(),
            center[2],
        ]);
    }
    let mut indices = Vec::with_capacity(n * 3);
    for i in 0..n {
        indices.push(0);
        indices.push((i + 1) as u32);
        indices.push(((i + 1) % n + 1) as u32);
    }
    FanMesh { positions, indices }
}

/// Vertex count of fan mesh.
#[allow(dead_code)]
pub fn fan_vertex_count(fan: &FanMesh) -> usize {
    fan.positions.len()
}

/// Triangle count of fan mesh.
#[allow(dead_code)]
pub fn fan_triangle_count(fan: &FanMesh) -> usize {
    fan.indices.len() / 3
}

/// Compute the area of the fan (approximate circle area).
#[allow(dead_code)]
pub fn fan_area(fan: &FanMesh) -> f32 {
    let tri_count = fan.indices.len() / 3;
    let mut total = 0.0f32;
    for t in 0..tri_count {
        let v0 = fan.positions[fan.indices[t * 3] as usize];
        let v1 = fan.positions[fan.indices[t * 3 + 1] as usize];
        let v2 = fan.positions[fan.indices[t * 3 + 2] as usize];
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let cx = e1[1] * e2[2] - e1[2] * e2[1];
        let cy = e1[2] * e2[0] - e1[0] * e2[2];
        let cz = e1[0] * e2[1] - e1[1] * e2[0];
        total += (cx * cx + cy * cy + cz * cz).sqrt() * 0.5;
    }
    total
}

/// Generate a fan from existing boundary vertices and a center.
#[allow(dead_code)]
pub fn fan_from_boundary(center: [f32; 3], boundary: &[[f32; 3]]) -> FanMesh {
    let n = boundary.len();
    if n < 2 {
        return FanMesh {
            positions: vec![center],
            indices: vec![],
        };
    }
    let mut positions = Vec::with_capacity(n + 1);
    positions.push(center);
    positions.extend_from_slice(boundary);
    let mut indices = Vec::with_capacity(n * 3);
    for i in 0..n {
        indices.push(0);
        indices.push((i + 1) as u32);
        indices.push(((i + 1) % n + 1) as u32);
    }
    FanMesh { positions, indices }
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn fan_mesh_to_json(fan: &FanMesh) -> String {
    format!(
        "{{\"vertices\":{},\"triangles\":{}}}",
        fan.positions.len(),
        fan.indices.len() / 3,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_generate_fan_3() {
        let fan = generate_fan([0.0, 0.0, 0.0], 1.0, 3);
        assert_eq!(fan_vertex_count(&fan), 4); // center + 3
        assert_eq!(fan_triangle_count(&fan), 3);
    }

    #[test]
    fn test_generate_fan_min_segments() {
        let fan = generate_fan([0.0, 0.0, 0.0], 1.0, 1);
        assert_eq!(fan_triangle_count(&fan), 3); // clamped to 3
    }

    #[test]
    fn test_fan_area_circle() {
        let fan = generate_fan([0.0, 0.0, 0.0], 1.0, 64);
        let area = fan_area(&fan);
        assert!((area - PI).abs() < 0.05); // approximate circle area
    }

    #[test]
    fn test_fan_from_boundary() {
        let boundary = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]];
        let fan = fan_from_boundary([0.0, 0.0, 0.0], &boundary);
        assert_eq!(fan_vertex_count(&fan), 4);
        assert_eq!(fan_triangle_count(&fan), 3);
    }

    #[test]
    fn test_fan_from_boundary_too_few() {
        let fan = fan_from_boundary([0.0, 0.0, 0.0], &[[1.0, 0.0, 0.0]]);
        assert_eq!(fan_triangle_count(&fan), 0);
    }

    #[test]
    fn test_fan_mesh_to_json() {
        let fan = generate_fan([0.0, 0.0, 0.0], 1.0, 4);
        let json = fan_mesh_to_json(&fan);
        assert!(json.contains("\"vertices\":5"));
    }

    #[test]
    fn test_fan_vertex_count_6() {
        let fan = generate_fan([0.0, 0.0, 0.0], 2.0, 6);
        assert_eq!(fan_vertex_count(&fan), 7);
    }

    #[test]
    fn test_fan_area_positive() {
        let fan = generate_fan([0.0, 0.0, 0.0], 1.0, 8);
        assert!(fan_area(&fan) > 0.0);
    }

    #[test]
    fn test_center_is_first() {
        let fan = generate_fan([5.0, 3.0, 1.0], 1.0, 4);
        assert!((fan.positions[0][0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_generate_fan_large() {
        let fan = generate_fan([0.0; 3], 1.0, 100);
        assert_eq!(fan_triangle_count(&fan), 100);
    }
}
