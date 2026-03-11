// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Shadow volume mesh — generates shadow volume geometry for stencil shadow rendering.

/// A shadow volume edge (used for fin extrusion).
#[derive(Debug, Clone, Copy)]
pub struct ShadowEdge {
    pub v0: u32,
    pub v1: u32,
    pub face_a: u32,
    pub face_b: u32,
}

/// The shadow volume mesh containing caps and side fins.
#[derive(Debug, Default, Clone)]
pub struct ShadowVolumeMesh {
    pub front_cap: Vec<[f32; 3]>,
    pub back_cap: Vec<[f32; 3]>,
    pub fins: Vec<[f32; 3]>,
    pub light_direction: [f32; 3],
    pub extrude_length: f32,
}

impl ShadowVolumeMesh {
    /// Creates a new shadow volume mesh.
    pub fn new(light_direction: [f32; 3], extrude_length: f32) -> Self {
        Self {
            light_direction,
            extrude_length,
            ..Default::default()
        }
    }

    /// Returns the total vertex count.
    pub fn total_vertex_count(&self) -> usize {
        self.front_cap.len() + self.back_cap.len() + self.fins.len()
    }

    /// Clears all geometry.
    pub fn clear(&mut self) {
        self.front_cap.clear();
        self.back_cap.clear();
        self.fins.clear();
    }
}

/// Extrudes a vertex in the light direction by `length`.
pub fn extrude_in_light_dir(position: [f32; 3], light_dir: [f32; 3], length: f32) -> [f32; 3] {
    [
        position[0] + light_dir[0] * length,
        position[1] + light_dir[1] * length,
        position[2] + light_dir[2] * length,
    ]
}

/// Checks if a face faces the light source (dot product of face normal and light dir > 0).
pub fn face_faces_light(normal: [f32; 3], light_dir: [f32; 3]) -> bool {
    let dot = normal[0] * light_dir[0] + normal[1] * light_dir[1] + normal[2] * light_dir[2];
    dot > 0.0
}

/// Finds silhouette edges of a shadow volume (shared between lit and unlit faces).
pub fn find_shadow_silhouette_edges(edges: &[ShadowEdge], lit_faces: &[bool]) -> Vec<ShadowEdge> {
    edges
        .iter()
        .copied()
        .filter(|e| {
            let fa = e.face_a as usize;
            let fb = e.face_b as usize;
            let a_lit = lit_faces.get(fa).copied().unwrap_or(false);
            let b_lit = lit_faces.get(fb).copied().unwrap_or(false);
            a_lit != b_lit
        })
        .collect()
}

/// Generates a simple shadow volume for a triangle list.
pub fn generate_shadow_volume(
    positions: &[[f32; 3]],
    light_dir: [f32; 3],
    extrude_length: f32,
) -> ShadowVolumeMesh {
    let mut sv = ShadowVolumeMesh::new(light_dir, extrude_length);
    sv.front_cap.extend_from_slice(positions);
    sv.back_cap = positions
        .iter()
        .map(|&p| extrude_in_light_dir(p, light_dir, extrude_length))
        .collect();
    sv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_shadow_volume_empty() {
        /* New shadow volume should have zero vertices */
        let sv = ShadowVolumeMesh::new([0.0, -1.0, 0.0], 5.0);
        assert_eq!(sv.total_vertex_count(), 0);
    }

    #[test]
    fn test_extrude_in_light_dir() {
        /* Extruding along Y by 2 should add 2 to y component */
        let pos = [0.0f32, 0.0, 0.0];
        let dir = [0.0f32, 1.0, 0.0];
        let result = extrude_in_light_dir(pos, dir, 2.0);
        assert!((result[1] - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_face_faces_light_true() {
        /* Normal and light in same direction → faces light */
        assert!(face_faces_light([0.0, 1.0, 0.0], [0.0, 1.0, 0.0]));
    }

    #[test]
    fn test_face_faces_light_false() {
        /* Normal opposite to light → does not face light */
        assert!(!face_faces_light([0.0, -1.0, 0.0], [0.0, 1.0, 0.0]));
    }

    #[test]
    fn test_generate_shadow_volume_caps() {
        /* Front and back caps should have same length as input */
        let pos = vec![[0.0f32; 3]; 6];
        let sv = generate_shadow_volume(&pos, [0.0, -1.0, 0.0], 3.0);
        assert_eq!(sv.front_cap.len(), 6);
        assert_eq!(sv.back_cap.len(), 6);
    }

    #[test]
    fn test_clear() {
        /* Clear should zero all caps and fins */
        let pos = vec![[0.0f32; 3]; 3];
        let mut sv = generate_shadow_volume(&pos, [0.0, -1.0, 0.0], 1.0);
        sv.clear();
        assert_eq!(sv.total_vertex_count(), 0);
    }

    #[test]
    fn test_find_shadow_silhouette_edges_empty() {
        /* No edges → empty result */
        assert!(find_shadow_silhouette_edges(&[], &[true, false]).is_empty());
    }

    #[test]
    fn test_find_shadow_silhouette_edges_filter() {
        /* Only edge between lit and unlit face is silhouette */
        let edges = vec![ShadowEdge {
            v0: 0,
            v1: 1,
            face_a: 0,
            face_b: 1,
        }];
        let lit_faces = vec![true, false];
        let result = find_shadow_silhouette_edges(&edges, &lit_faces);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_extrude_zero_length() {
        /* Zero extrude length should return original position */
        let p = [1.0f32, 2.0, 3.0];
        assert_eq!(extrude_in_light_dir(p, [1.0, 0.0, 0.0], 0.0), p);
    }

    #[test]
    fn test_total_vertex_count() {
        /* total_vertex_count should sum all three vecs */
        let mut sv = ShadowVolumeMesh::new([0.0, -1.0, 0.0], 1.0);
        sv.front_cap.push([0.0; 3]);
        sv.back_cap.push([0.0; 3]);
        sv.fins.push([0.0; 3]);
        assert_eq!(sv.total_vertex_count(), 3);
    }
}
