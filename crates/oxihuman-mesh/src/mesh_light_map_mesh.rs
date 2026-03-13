// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Lightmap UV second channel — stores and validates a second UV set for lightmapping.

/// A vertex entry with a primary UV and a lightmap UV.
#[derive(Debug, Clone, Copy)]
pub struct LightmapVertex {
    pub position: [f32; 3],
    pub uv0: [f32; 2],
    pub uv1: [f32; 2],
}

/// Configuration for lightmap UV generation.
#[derive(Debug, Clone, Copy)]
pub struct LightmapConfig {
    pub padding: f32,
    pub texel_density: f32,
    pub atlas_size: u32,
}

impl Default for LightmapConfig {
    fn default() -> Self {
        Self {
            padding: 0.005,
            texel_density: 10.0,
            atlas_size: 1024,
        }
    }
}

/// A mesh with a second UV channel for lightmapping.
#[derive(Debug, Default, Clone)]
pub struct LightmapMesh {
    pub vertices: Vec<LightmapVertex>,
    pub indices: Vec<u32>,
    pub config: LightmapConfig,
}

impl LightmapMesh {
    /// Creates a new lightmap mesh.
    pub fn new(config: LightmapConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Returns the vertex count.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the triangle count.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

/// Validates that all lightmap UVs (uv1) are within [0, 1].
pub fn validate_lightmap_uvs(mesh: &LightmapMesh) -> bool {
    mesh.vertices
        .iter()
        .all(|v| (0.0..=1.0).contains(&v.uv1[0]) && (0.0..=1.0).contains(&v.uv1[1]))
}

/// Scales lightmap UVs by a factor (for texel density adjustment).
pub fn scale_lightmap_uvs(vertices: &mut [LightmapVertex], scale: f32) {
    for v in vertices.iter_mut() {
        v.uv1[0] = (v.uv1[0] * scale).clamp(0.0, 1.0);
        v.uv1[1] = (v.uv1[1] * scale).clamp(0.0, 1.0);
    }
}

/// Checks for UV overlap (simplified: checks if any two UVs are identical).
pub fn has_uv1_overlap(vertices: &[LightmapVertex]) -> bool {
    for i in 0..vertices.len() {
        for j in (i + 1)..vertices.len() {
            let a = vertices[i].uv1;
            let b = vertices[j].uv1;
            if (a[0] - b[0]).abs() < 1e-5 && (a[1] - b[1]).abs() < 1e-5 {
                return true;
            }
        }
    }
    false
}

/// Generates a simple planar lightmap UV layout.
pub fn generate_planar_lightmap_uvs(positions: &[[f32; 3]]) -> Vec<[f32; 2]> {
    let min_x = positions.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
    let max_x = positions
        .iter()
        .map(|p| p[0])
        .fold(f32::NEG_INFINITY, f32::max);
    let min_z = positions.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min);
    let max_z = positions
        .iter()
        .map(|p| p[2])
        .fold(f32::NEG_INFINITY, f32::max);
    let range_x = (max_x - min_x).max(f32::EPSILON);
    let range_z = (max_z - min_z).max(f32::EPSILON);
    positions
        .iter()
        .map(|p| {
            [
                ((p[0] - min_x) / range_x).clamp(0.0, 1.0),
                ((p[2] - min_z) / range_z).clamp(0.0, 1.0),
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vertex(u1: f32, v1: f32) -> LightmapVertex {
        LightmapVertex {
            position: [0.0; 3],
            uv0: [0.0; 2],
            uv1: [u1, v1],
        }
    }

    #[test]
    fn test_new_lightmap_mesh_empty() {
        /* New mesh should have zero vertices */
        assert_eq!(
            LightmapMesh::new(LightmapConfig::default()).vertex_count(),
            0
        );
    }

    #[test]
    fn test_validate_uvs_valid() {
        /* UVs in [0,1] should validate */
        let mut mesh = LightmapMesh::new(LightmapConfig::default());
        mesh.vertices.push(make_vertex(0.5, 0.5));
        assert!(validate_lightmap_uvs(&mesh));
    }

    #[test]
    fn test_validate_uvs_invalid() {
        /* UVs outside [0,1] should fail validation */
        let mut mesh = LightmapMesh::new(LightmapConfig::default());
        mesh.vertices.push(make_vertex(1.5, 0.5));
        assert!(!validate_lightmap_uvs(&mesh));
    }

    #[test]
    fn test_scale_lightmap_uvs() {
        /* Scaling by 0.5 should halve UV values */
        let mut verts = vec![make_vertex(1.0, 1.0)];
        scale_lightmap_uvs(&mut verts, 0.5);
        assert!((verts[0].uv1[0] - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_scale_clamps() {
        /* Scaling above 1 should clamp to 1 */
        let mut verts = vec![make_vertex(0.9, 0.9)];
        scale_lightmap_uvs(&mut verts, 2.0);
        assert!((0.0..=1.0).contains(&verts[0].uv1[0]));
    }

    #[test]
    fn test_has_uv1_overlap_false() {
        /* Distinct UVs should not overlap */
        let verts = vec![make_vertex(0.1, 0.1), make_vertex(0.9, 0.9)];
        assert!(!has_uv1_overlap(&verts));
    }

    #[test]
    fn test_has_uv1_overlap_true() {
        /* Identical UVs should report overlap */
        let verts = vec![make_vertex(0.5, 0.5), make_vertex(0.5, 0.5)];
        assert!(has_uv1_overlap(&verts));
    }

    #[test]
    fn test_generate_planar_uvs_count() {
        /* Output UV count should match input position count */
        let pos = vec![[0.0f32; 3]; 4];
        assert_eq!(generate_planar_lightmap_uvs(&pos).len(), 4);
    }

    #[test]
    fn test_generate_planar_uvs_range() {
        /* All generated UVs should be in [0,1] */
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        for uv in generate_planar_lightmap_uvs(&pos) {
            assert!((0.0..=1.0).contains(&uv[0]));
            assert!((0.0..=1.0).contains(&uv[1]));
        }
    }

    #[test]
    fn test_default_config_atlas_size() {
        /* Default atlas size should be 1024 */
        assert_eq!(LightmapConfig::default().atlas_size, 1024);
    }
}
