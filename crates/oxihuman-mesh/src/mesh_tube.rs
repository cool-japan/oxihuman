// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Generate a tube/pipe mesh along a polyline (list of centre-line points).

#![allow(dead_code)]

use std::f32::consts::PI;

/// A generated tube mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TubeMesh {
    /// Vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Per-vertex normals.
    pub normals: Vec<[f32; 3]>,
    /// Triangle indices.
    pub indices: Vec<u32>,
    /// Number of cross-section rings.
    pub ring_count: usize,
}

/// Generate a tube mesh along `spine` (list of centre-line points), each ring
/// having `segments` vertices and radius `radius`.
#[allow(dead_code)]
pub fn generate_tube(spine: &[[f32; 3]], radius: f32, segments: usize) -> TubeMesh {
    let seg = segments.max(3);
    let r = radius.abs().max(f32::EPSILON);
    let n = spine.len();

    if n < 2 {
        return TubeMesh {
            positions: vec![],
            normals: vec![],
            indices: vec![],
            ring_count: 0,
        };
    }

    // Compute per-segment tangents
    let mut tangents: Vec<[f32; 3]> = Vec::with_capacity(n);
    for i in 0..n {
        let fwd = if i + 1 < n {
            sub(spine[i + 1], spine[i])
        } else {
            sub(spine[i], spine[i - 1])
        };
        tangents.push(normalize(fwd));
    }

    // Build initial basis (perpendicular to first tangent)
    let up = perpendicular(tangents[0]);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();

    // Transport frame along spine
    let mut prev_up = up;
    for i in 0..n {
        let t = tangents[i];
        // Project prev_up perpendicular to t
        let u = project_perp(prev_up, t);
        let u_norm = normalize(u);
        let v_norm = cross(t, u_norm);
        prev_up = u_norm;

        for j in 0..=seg {
            let theta = 2.0 * PI * (j as f32) / (seg as f32);
            let nx = u_norm[0] * theta.cos() + v_norm[0] * theta.sin();
            let ny = u_norm[1] * theta.cos() + v_norm[1] * theta.sin();
            let nz = u_norm[2] * theta.cos() + v_norm[2] * theta.sin();
            positions.push([
                spine[i][0] + r * nx,
                spine[i][1] + r * ny,
                spine[i][2] + r * nz,
            ]);
            normals.push([nx, ny, nz]);
        }
    }

    let stride = (seg + 1) as u32;
    let mut indices: Vec<u32> = Vec::new();
    for i in 0..(n - 1) as u32 {
        for j in 0..seg as u32 {
            let a = i * stride + j;
            let b = a + 1;
            let c = a + stride;
            let d = c + 1;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }

    TubeMesh {
        positions,
        normals,
        indices,
        ring_count: n,
    }
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = dot(v, v).sqrt().max(f32::EPSILON);
    [v[0] / len, v[1] / len, v[2] / len]
}

fn perpendicular(v: [f32; 3]) -> [f32; 3] {
    let (a, b, c) = (v[0].abs(), v[1].abs(), v[2].abs());
    let perp = if a <= b && a <= c {
        [0.0, -v[2], v[1]]
    } else if b <= c {
        [-v[2], 0.0, v[0]]
    } else {
        [-v[1], v[0], 0.0]
    };
    normalize(perp)
}

fn project_perp(v: [f32; 3], axis: [f32; 3]) -> [f32; 3] {
    let d = dot(v, axis);
    [v[0] - d * axis[0], v[1] - d * axis[1], v[2] - d * axis[2]]
}

/// Return the expected vertex count for a tube.
#[allow(dead_code)]
pub fn tube_vertex_count(spine_len: usize, segments: usize) -> usize {
    spine_len * (segments + 1)
}

/// Return the expected index count for a tube.
#[allow(dead_code)]
pub fn tube_index_count(spine_len: usize, segments: usize) -> usize {
    (spine_len - 1) * segments * 6
}

/// Total approximate surface area of the tube.
#[allow(dead_code)]
pub fn tube_surface_area(spine: &[[f32; 3]], radius: f32) -> f32 {
    if spine.len() < 2 {
        return 0.0;
    }
    let mut length = 0.0f32;
    for i in 0..spine.len() - 1 {
        let d = sub(spine[i + 1], spine[i]);
        length += dot(d, d).sqrt();
    }
    2.0 * PI * radius * length
}

/// Serialise as minimal JSON.
#[allow(dead_code)]
pub fn tube_to_json(tube: &TubeMesh) -> String {
    format!(
        "{{\"vertices\":{},\"indices\":{},\"rings\":{}}}",
        tube.positions.len(),
        tube.indices.len(),
        tube.ring_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn straight_spine() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, 2.0]]
    }

    #[test]
    fn test_generate_tube_produces_vertices() {
        let spine = straight_spine();
        let t = generate_tube(&spine, 0.5, 8);
        assert!(!t.positions.is_empty());
    }

    #[test]
    fn test_vertex_count_matches() {
        let spine = straight_spine();
        let t = generate_tube(&spine, 0.5, 8);
        let expected = tube_vertex_count(spine.len(), 8);
        assert_eq!(t.positions.len(), expected);
    }

    #[test]
    fn test_index_count_matches() {
        let spine = straight_spine();
        let t = generate_tube(&spine, 0.5, 8);
        let expected = tube_index_count(spine.len(), 8);
        assert_eq!(t.indices.len(), expected);
    }

    #[test]
    fn test_normals_count_matches() {
        let spine = straight_spine();
        let t = generate_tube(&spine, 0.5, 8);
        assert_eq!(t.normals.len(), t.positions.len());
    }

    #[test]
    fn test_single_point_returns_empty() {
        let t = generate_tube(&[[0.0, 0.0, 0.0]], 1.0, 8);
        assert!(t.positions.is_empty());
    }

    #[test]
    fn test_ring_count() {
        let spine = straight_spine();
        let t = generate_tube(&spine, 0.5, 8);
        assert_eq!(t.ring_count, spine.len());
    }

    #[test]
    fn test_surface_area_positive() {
        let spine = straight_spine();
        let sa = tube_surface_area(&spine, 0.5);
        assert!(sa > 0.0);
    }

    #[test]
    fn test_to_json() {
        let spine = straight_spine();
        let t = generate_tube(&spine, 0.5, 8);
        let json = tube_to_json(&t);
        assert!(json.contains("vertices"));
    }

    #[test]
    fn test_indices_valid() {
        let spine = straight_spine();
        let t = generate_tube(&spine, 0.5, 8);
        let n = t.positions.len() as u32;
        assert!(t.indices.iter().all(|&i| i < n));
    }

    #[test]
    fn test_curved_spine() {
        let spine = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
        let t = generate_tube(&spine, 0.2, 6);
        assert!(!t.positions.is_empty());
    }
}
