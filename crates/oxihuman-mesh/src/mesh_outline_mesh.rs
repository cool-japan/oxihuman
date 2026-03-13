// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Outline/silhouette mesh generation — produces extruded edge meshes for stylized outlines.

/// Configuration for outline mesh generation.
#[derive(Debug, Clone, Copy)]
pub struct OutlineConfig {
    pub extrude_distance: f32,
    pub only_silhouette: bool,
    pub cull_backface: bool,
}

impl Default for OutlineConfig {
    fn default() -> Self {
        Self {
            extrude_distance: 0.02,
            only_silhouette: true,
            cull_backface: true,
        }
    }
}

/// A single outline edge (pair of vertex indices).
#[derive(Debug, Clone, Copy)]
pub struct OutlineEdge {
    pub v0: u32,
    pub v1: u32,
    pub is_silhouette: bool,
}

/// The outline mesh containing extruded geometry.
#[derive(Debug, Default, Clone)]
pub struct OutlineMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub edges: Vec<OutlineEdge>,
    pub config: OutlineConfig,
}

impl OutlineMesh {
    /// Creates a new outline mesh with the given config.
    pub fn new(config: OutlineConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Returns the number of outline edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Returns silhouette edges only.
    pub fn silhouette_edges(&self) -> Vec<&OutlineEdge> {
        self.edges.iter().filter(|e| e.is_silhouette).collect()
    }
}

/// Extrudes a vertex position along its normal by `distance`.
pub fn extrude_vertex(position: [f32; 3], normal: [f32; 3], distance: f32) -> [f32; 3] {
    [
        position[0] + normal[0] * distance,
        position[1] + normal[1] * distance,
        position[2] + normal[2] * distance,
    ]
}

/// Determines if an edge is a silhouette edge given two face normals and view direction.
pub fn is_silhouette_edge(n0: [f32; 3], n1: [f32; 3], view_dir: [f32; 3]) -> bool {
    let dot0 = n0[0] * view_dir[0] + n0[1] * view_dir[1] + n0[2] * view_dir[2];
    let dot1 = n1[0] * view_dir[0] + n1[1] * view_dir[1] + n1[2] * view_dir[2];
    (dot0 >= 0.0) != (dot1 >= 0.0)
}

/// Generates an outline mesh by extruding boundary edges outward.
pub fn generate_outline_mesh(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    config: OutlineConfig,
) -> OutlineMesh {
    let n = positions.len().min(normals.len());
    let extruded: Vec<[f32; 3]> = (0..n)
        .map(|i| extrude_vertex(positions[i], normals[i], config.extrude_distance))
        .collect();
    let mut mesh = OutlineMesh::new(config);
    mesh.positions = extruded;
    mesh.normals = normals[..n].to_vec();
    mesh
}

/// Normalizes a 3D vector (returns zero vector for degenerate input).
pub fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < f32::EPSILON {
        return [0.0; 3];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        /* Default extrude distance should be 0.02 */
        assert!((OutlineConfig::default().extrude_distance - 0.02).abs() < f32::EPSILON);
    }

    #[test]
    fn test_extrude_vertex_along_z() {
        /* Extruding along Z should add distance to z component */
        let pos = [0.0f32, 0.0, 0.0];
        let norm = [0.0f32, 0.0, 1.0];
        let result = extrude_vertex(pos, norm, 0.5);
        assert!((result[2] - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_extrude_zero_distance() {
        /* Zero extrude distance should return original position */
        let pos = [1.0f32, 2.0, 3.0];
        let norm = [0.0f32, 1.0, 0.0];
        assert_eq!(extrude_vertex(pos, norm, 0.0), pos);
    }

    #[test]
    fn test_is_silhouette_edge_true() {
        /* Normals facing opposite sides of view → silhouette */
        let n0 = [0.0f32, 0.0, 1.0];
        let n1 = [0.0f32, 0.0, -1.0];
        let view = [0.0f32, 0.0, 1.0];
        assert!(is_silhouette_edge(n0, n1, view));
    }

    #[test]
    fn test_is_silhouette_edge_false() {
        /* Both normals facing viewer → not silhouette */
        let n = [0.0f32, 0.0, 1.0];
        let view = [0.0f32, 0.0, 1.0];
        assert!(!is_silhouette_edge(n, n, view));
    }

    #[test]
    fn test_generate_outline_mesh_vertex_count() {
        /* Generated mesh should have same vertex count as input */
        let pos = vec![[0.0f32; 3]; 5];
        let norm = vec![[0.0f32, 0.0, 1.0]; 5];
        let mesh = generate_outline_mesh(&pos, &norm, OutlineConfig::default());
        assert_eq!(mesh.positions.len(), 5);
    }

    #[test]
    fn test_edge_count_empty() {
        /* New outline mesh should have zero edges */
        assert_eq!(OutlineMesh::new(OutlineConfig::default()).edge_count(), 0);
    }

    #[test]
    fn test_normalize_unit_vector() {
        /* Normalizing a unit vector should return the same vector */
        let v = [1.0f32, 0.0, 0.0];
        let n = normalize(v);
        assert!((n[0] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_normalize_zero_vector() {
        /* Normalizing zero vector should return zero vector */
        let n = normalize([0.0; 3]);
        assert_eq!(n, [0.0; 3]);
    }

    #[test]
    fn test_silhouette_edges_filter() {
        /* silhouette_edges should return only silhouette edges */
        let mut mesh = OutlineMesh::new(OutlineConfig::default());
        mesh.edges.push(OutlineEdge {
            v0: 0,
            v1: 1,
            is_silhouette: true,
        });
        mesh.edges.push(OutlineEdge {
            v0: 1,
            v1: 2,
            is_silhouette: false,
        });
        assert_eq!(mesh.silhouette_edges().len(), 1);
    }
}
