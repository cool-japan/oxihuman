// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hexagonal mesh generation on a plane.

use std::f32::consts::FRAC_1_SQRT_2;

/// A generated hexagonal grid mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HexMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// Generate a hexagonal grid mesh with `rows` x `cols` hex cells.
/// Each hex cell is a regular hexagon tessellated into 6 triangles.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn generate_hex_mesh(rows: usize, cols: usize, cell_size: f32) -> HexMesh {
    if rows == 0 || cols == 0 {
        return HexMesh {
            positions: vec![],
            indices: vec![],
        };
    }
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    let h = cell_size * 3.0_f32.sqrt() * 0.5;

    for r in 0..rows {
        for c in 0..cols {
            let offset_x = if r.is_multiple_of(2) {
                0.0
            } else {
                cell_size * 0.75
            };
            let cx = c as f32 * cell_size * 1.5 + offset_x;
            let cy = r as f32 * h;
            let base = positions.len() as u32;
            // Center vertex
            positions.push([cx, cy, 0.0]);
            // 6 corners
            for i in 0..6 {
                let angle = std::f32::consts::PI / 3.0 * i as f32;
                positions.push([
                    cx + cell_size * 0.5 * angle.cos(),
                    cy + cell_size * 0.5 * angle.sin(),
                    0.0,
                ]);
            }
            // 6 triangles
            for i in 0..6u32 {
                indices.push(base);
                indices.push(base + 1 + i);
                indices.push(base + 1 + (i + 1) % 6);
            }
        }
    }

    HexMesh { positions, indices }
}

/// Vertex count.
#[allow(dead_code)]
pub fn hex_vertex_count(mesh: &HexMesh) -> usize {
    mesh.positions.len()
}

/// Triangle count.
#[allow(dead_code)]
pub fn hex_triangle_count(mesh: &HexMesh) -> usize {
    mesh.indices.len() / 3
}

/// Cell count (each hex = 7 vertices, 6 triangles).
#[allow(dead_code)]
pub fn hex_cell_count(rows: usize, cols: usize) -> usize {
    rows * cols
}

/// Approximate total area of hex mesh.
#[allow(dead_code)]
pub fn hex_mesh_area(mesh: &HexMesh) -> f32 {
    let tri_count = mesh.indices.len() / 3;
    let mut area = 0.0f32;
    for t in 0..tri_count {
        let v0 = mesh.positions[mesh.indices[t * 3] as usize];
        let v1 = mesh.positions[mesh.indices[t * 3 + 1] as usize];
        let v2 = mesh.positions[mesh.indices[t * 3 + 2] as usize];
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let cx = e1[1] * e2[2] - e1[2] * e2[1];
        let cy = e1[2] * e2[0] - e1[0] * e2[2];
        let cz = e1[0] * e2[1] - e1[1] * e2[0];
        area += (cx * cx + cy * cy + cz * cz).sqrt() * 0.5;
    }
    area
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn hex_mesh_to_json(mesh: &HexMesh) -> String {
    format!(
        "{{\"vertices\":{},\"triangles\":{}}}",
        mesh.positions.len(),
        mesh.indices.len() / 3,
    )
}

/// Check FRAC_1_SQRT_2 usage (utility constant).
#[allow(dead_code)]
pub fn hex_diagonal_factor() -> f32 {
    FRAC_1_SQRT_2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_1x1() {
        let mesh = generate_hex_mesh(1, 1, 1.0);
        assert_eq!(hex_vertex_count(&mesh), 7);
        assert_eq!(hex_triangle_count(&mesh), 6);
    }

    #[test]
    fn test_generate_2x2() {
        let mesh = generate_hex_mesh(2, 2, 1.0);
        assert_eq!(hex_vertex_count(&mesh), 28); // 4 * 7
        assert_eq!(hex_triangle_count(&mesh), 24); // 4 * 6
    }

    #[test]
    fn test_generate_empty() {
        let mesh = generate_hex_mesh(0, 5, 1.0);
        assert!(mesh.positions.is_empty());
    }

    #[test]
    fn test_hex_cell_count() {
        assert_eq!(hex_cell_count(3, 4), 12);
    }

    #[test]
    fn test_hex_mesh_area_positive() {
        let mesh = generate_hex_mesh(1, 1, 2.0);
        assert!(hex_mesh_area(&mesh) > 0.0);
    }

    #[test]
    fn test_hex_mesh_to_json() {
        let mesh = generate_hex_mesh(1, 1, 1.0);
        let json = hex_mesh_to_json(&mesh);
        assert!(json.contains("\"vertices\":7"));
    }

    #[test]
    fn test_center_at_origin() {
        let mesh = generate_hex_mesh(1, 1, 1.0);
        // First cell center near origin
        assert!((mesh.positions[0][0]).abs() < 1e-6);
    }

    #[test]
    fn test_hex_diagonal_factor() {
        let f = hex_diagonal_factor();
        assert!((f - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
    }

    #[test]
    fn test_large_grid() {
        let mesh = generate_hex_mesh(5, 5, 1.0);
        assert_eq!(hex_vertex_count(&mesh), 175);
    }

    #[test]
    fn test_area_scales_with_cell_size() {
        let m1 = generate_hex_mesh(1, 1, 1.0);
        let m2 = generate_hex_mesh(1, 1, 2.0);
        assert!(hex_mesh_area(&m2) > hex_mesh_area(&m1));
    }
}
