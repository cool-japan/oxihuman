// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export a triangle-based collision mesh.

/// A single collision triangle.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CollisionTriangle {
    pub v0: [f32; 3],
    pub v1: [f32; 3],
    pub v2: [f32; 3],
    pub material_id: u32,
}

/// A collision triangle mesh export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionTriangleExport {
    pub triangles: Vec<CollisionTriangle>,
}

/// Create a new collision triangle export from positions and indices.
#[allow(dead_code)]
pub fn from_mesh(
    positions: &[[f32; 3]],
    indices: &[u32],
    material_id: u32,
) -> CollisionTriangleExport {
    let tri_count = indices.len() / 3;
    let mut triangles = Vec::with_capacity(tri_count);
    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        if i0 < positions.len() && i1 < positions.len() && i2 < positions.len() {
            triangles.push(CollisionTriangle {
                v0: positions[i0],
                v1: positions[i1],
                v2: positions[i2],
                material_id,
            });
        }
    }
    CollisionTriangleExport { triangles }
}

/// Count triangles.
#[allow(dead_code)]
pub fn collision_tri_count(export: &CollisionTriangleExport) -> usize {
    export.triangles.len()
}

/// Compute the area of a single collision triangle.
#[allow(dead_code)]
pub fn triangle_area_ct(tri: &CollisionTriangle) -> f32 {
    let e1 = [
        tri.v1[0] - tri.v0[0],
        tri.v1[1] - tri.v0[1],
        tri.v1[2] - tri.v0[2],
    ];
    let e2 = [
        tri.v2[0] - tri.v0[0],
        tri.v2[1] - tri.v0[1],
        tri.v2[2] - tri.v0[2],
    ];
    let cross = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt() * 0.5
}

/// Total surface area.
#[allow(dead_code)]
pub fn total_collision_area(export: &CollisionTriangleExport) -> f32 {
    export.triangles.iter().map(triangle_area_ct).sum()
}

/// Count triangles by material id.
#[allow(dead_code)]
pub fn triangles_for_material(export: &CollisionTriangleExport, material_id: u32) -> usize {
    export
        .triangles
        .iter()
        .filter(|t| t.material_id == material_id)
        .count()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn collision_triangle_to_json(export: &CollisionTriangleExport) -> String {
    format!(
        "{{\"triangle_count\":{},\"total_area\":{:.4}}}",
        export.triangles.len(),
        total_collision_area(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_tri_export() -> CollisionTriangleExport {
        from_mesh(
            &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            &[0, 1, 2],
            0,
        )
    }

    #[test]
    fn test_from_mesh_count() {
        assert_eq!(collision_tri_count(&unit_tri_export()), 1);
    }

    #[test]
    fn test_triangle_area_positive() {
        let t = CollisionTriangle {
            v0: [0.0, 0.0, 0.0],
            v1: [1.0, 0.0, 0.0],
            v2: [0.0, 1.0, 0.0],
            material_id: 0,
        };
        assert!(triangle_area_ct(&t) > 0.0);
    }

    #[test]
    fn test_total_area_positive() {
        let e = unit_tri_export();
        assert!(total_collision_area(&e) > 0.0);
    }

    #[test]
    fn test_triangles_for_material() {
        let e = unit_tri_export();
        assert_eq!(triangles_for_material(&e, 0), 1);
    }

    #[test]
    fn test_triangles_for_wrong_material() {
        let e = unit_tri_export();
        assert_eq!(triangles_for_material(&e, 1), 0);
    }

    #[test]
    fn test_empty_export() {
        let e = from_mesh(&[], &[], 0);
        assert_eq!(collision_tri_count(&e), 0);
    }

    #[test]
    fn test_collision_triangle_to_json() {
        let e = unit_tri_export();
        let j = collision_triangle_to_json(&e);
        assert!(j.contains("triangle_count"));
    }

    #[test]
    fn test_oob_indices_skipped() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let idx = vec![0u32, 1, 99];
        let e = from_mesh(&pos, &idx, 0);
        assert_eq!(collision_tri_count(&e), 0);
    }

    #[test]
    fn test_area_unit_right_triangle() {
        let t = CollisionTriangle {
            v0: [0.0, 0.0, 0.0],
            v1: [2.0, 0.0, 0.0],
            v2: [0.0, 2.0, 0.0],
            material_id: 0,
        };
        assert!((triangle_area_ct(&t) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_total_area_two_tris() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
        ];
        let idx = vec![0u32, 1, 2, 3, 4, 5];
        let e = from_mesh(&pos, &idx, 0);
        assert!(total_collision_area(&e) > 0.5);
    }
}
