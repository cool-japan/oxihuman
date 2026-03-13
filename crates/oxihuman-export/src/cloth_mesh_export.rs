// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cloth mesh export: per-vertex cloth simulation metadata.

/// Per-vertex cloth properties.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ClothVertex {
    pub mass: f32,
    pub drag: f32,
    pub pinned: bool,
}

/// Cloth mesh export bundle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClothMeshExport {
    pub vertices: Vec<ClothVertex>,
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// Create a cloth mesh export from a triangle mesh.
#[allow(dead_code)]
pub fn new_cloth_mesh_export(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> ClothMeshExport {
    let n = positions.len();
    ClothMeshExport {
        vertices: vec![
            ClothVertex {
                mass: 1.0,
                drag: 0.02,
                pinned: false
            };
            n
        ],
        positions,
        indices,
    }
}

/// Pin a vertex (mass = 0, pinned = true).
#[allow(dead_code)]
pub fn pin_vertex(exp: &mut ClothMeshExport, idx: usize) {
    if idx < exp.vertices.len() {
        exp.vertices[idx].pinned = true;
        exp.vertices[idx].mass = 0.0;
    }
}

/// Unpin a vertex.
#[allow(dead_code)]
pub fn unpin_vertex(exp: &mut ClothMeshExport, idx: usize) {
    if idx < exp.vertices.len() {
        exp.vertices[idx].pinned = false;
        exp.vertices[idx].mass = 1.0;
    }
}

/// Pinned vertex count.
#[allow(dead_code)]
pub fn pinned_count(exp: &ClothMeshExport) -> usize {
    exp.vertices.iter().filter(|v| v.pinned).count()
}

/// Free (unpinned) vertex count.
#[allow(dead_code)]
pub fn free_count(exp: &ClothMeshExport) -> usize {
    exp.vertices.iter().filter(|v| !v.pinned).count()
}

/// Total mass.
#[allow(dead_code)]
pub fn total_mass(exp: &ClothMeshExport) -> f32 {
    exp.vertices.iter().map(|v| v.mass).sum()
}

/// Set drag for all vertices.
#[allow(dead_code)]
pub fn set_uniform_drag(exp: &mut ClothMeshExport, drag: f32) {
    for v in &mut exp.vertices {
        v.drag = drag;
    }
}

/// Vertex count.
#[allow(dead_code)]
pub fn cloth_vertex_count(exp: &ClothMeshExport) -> usize {
    exp.vertices.len()
}

/// Serialise to JSON.
#[allow(dead_code)]
pub fn cloth_mesh_to_json(exp: &ClothMeshExport) -> String {
    format!(
        "{{\"vertex_count\":{},\"pinned_count\":{}}}",
        cloth_vertex_count(exp),
        pinned_count(exp)
    )
}

/// Validate: no negative mass on free vertices.
#[allow(dead_code)]
pub fn validate_cloth_mesh(exp: &ClothMeshExport) -> bool {
    exp.vertices
        .iter()
        .all(|v| v.mass >= 0.0 && (0.0..=1.0).contains(&v.drag))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_tri_cloth() -> ClothMeshExport {
        let pos = vec![
            [0.0f32; 3],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let idx = vec![0u32, 1, 2, 1, 3, 2];
        new_cloth_mesh_export(pos, idx)
    }

    #[test]
    fn vertex_count() {
        let exp = two_tri_cloth();
        assert_eq!(cloth_vertex_count(&exp), 4);
    }

    #[test]
    fn no_pinned_initially() {
        let exp = two_tri_cloth();
        assert_eq!(pinned_count(&exp), 0);
    }

    #[test]
    fn pin_vertex_works() {
        let mut exp = two_tri_cloth();
        pin_vertex(&mut exp, 0);
        assert_eq!(pinned_count(&exp), 1);
    }

    #[test]
    fn unpin_works() {
        let mut exp = two_tri_cloth();
        pin_vertex(&mut exp, 0);
        unpin_vertex(&mut exp, 0);
        assert_eq!(pinned_count(&exp), 0);
    }

    #[test]
    fn total_mass_four_verts() {
        let exp = two_tri_cloth();
        assert!((total_mass(&exp) - 4.0).abs() < 1e-5);
    }

    #[test]
    fn free_count_after_pin() {
        let mut exp = two_tri_cloth();
        pin_vertex(&mut exp, 1);
        assert_eq!(free_count(&exp), 3);
    }

    #[test]
    fn set_uniform_drag_test() {
        let mut exp = two_tri_cloth();
        set_uniform_drag(&mut exp, 0.1);
        assert!(exp.vertices.iter().all(|v| (v.drag - 0.1).abs() < 1e-5));
    }

    #[test]
    fn validate_valid() {
        let exp = two_tri_cloth();
        assert!(validate_cloth_mesh(&exp));
    }

    #[test]
    fn json_contains_vertex_count() {
        let exp = two_tri_cloth();
        let j = cloth_mesh_to_json(&exp);
        assert!(j.contains("vertex_count"));
    }

    #[test]
    fn drag_in_range() {
        let v = ClothVertex {
            mass: 1.0,
            drag: 0.02,
            pinned: false,
        };
        assert!((0.0..=1.0).contains(&v.drag));
    }
}
