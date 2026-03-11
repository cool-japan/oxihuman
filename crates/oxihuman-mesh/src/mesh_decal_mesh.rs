// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Decal projection mesh — projects a decal texture onto surface geometry.

/// A 3D box used to project a decal onto surfaces.
#[derive(Debug, Clone, Copy)]
pub struct DecalProjector {
    pub center: [f32; 3],
    pub half_extents: [f32; 3],
    pub normal: [f32; 3],
}

/// A projected decal vertex with position and UV.
#[derive(Debug, Clone, Copy)]
pub struct DecalVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub depth: f32,
}

/// The resulting decal mesh after projection.
#[derive(Debug, Default, Clone)]
pub struct DecalMesh {
    pub vertices: Vec<DecalVertex>,
    pub indices: Vec<u32>,
}

impl DecalMesh {
    /// Creates an empty decal mesh.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of projected vertices.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of triangles.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Clears all geometry.
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }
}

/// Projects a world-space point into decal UV space.
pub fn project_to_decal_uv(point: [f32; 3], projector: &DecalProjector) -> [f32; 2] {
    let local = [
        point[0] - projector.center[0],
        point[1] - projector.center[1],
        point[2] - projector.center[2],
    ];
    let u = (local[0] / (projector.half_extents[0] * 2.0) + 0.5).clamp(0.0, 1.0);
    let v = (local[1] / (projector.half_extents[1] * 2.0) + 0.5).clamp(0.0, 1.0);
    [u, v]
}

/// Checks if a point is inside the decal projector box.
pub fn point_in_projector(point: [f32; 3], projector: &DecalProjector) -> bool {
    let local = [
        (point[0] - projector.center[0]).abs(),
        (point[1] - projector.center[1]).abs(),
        (point[2] - projector.center[2]).abs(),
    ];
    local[0] <= projector.half_extents[0]
        && local[1] <= projector.half_extents[1]
        && local[2] <= projector.half_extents[2]
}

/// Computes the decal depth for a projected point (normalized 0..1).
pub fn decal_depth(point: [f32; 3], projector: &DecalProjector) -> f32 {
    let dz = (point[2] - projector.center[2]) / projector.half_extents[2].max(f32::EPSILON);
    (dz * 0.5 + 0.5).clamp(0.0, 1.0)
}

/// Projects surface vertices through the decal projector, returning a DecalMesh.
pub fn project_decal(
    positions: &[[f32; 3]],
    indices: &[u32],
    projector: &DecalProjector,
) -> DecalMesh {
    let mut mesh = DecalMesh::new();
    for &pos in positions {
        if point_in_projector(pos, projector) {
            mesh.vertices.push(DecalVertex {
                position: pos,
                uv: project_to_decal_uv(pos, projector),
                depth: decal_depth(pos, projector),
            });
        }
    }
    mesh.indices.extend_from_slice(indices);
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_projector() -> DecalProjector {
        DecalProjector {
            center: [0.0; 3],
            half_extents: [1.0; 3],
            normal: [0.0, 0.0, 1.0],
        }
    }

    #[test]
    fn test_new_decal_mesh_empty() {
        /* New mesh should have zero vertices */
        assert_eq!(DecalMesh::new().vertex_count(), 0);
    }

    #[test]
    fn test_point_in_projector_center() {
        /* Center point must be inside */
        assert!(point_in_projector([0.0; 3], &unit_projector()));
    }

    #[test]
    fn test_point_outside_projector() {
        /* Far point must be outside */
        assert!(!point_in_projector([5.0, 0.0, 0.0], &unit_projector()));
    }

    #[test]
    fn test_project_uv_center() {
        /* Center maps to UV (0.5, 0.5) */
        let uv = project_to_decal_uv([0.0; 3], &unit_projector());
        assert!((uv[0] - 0.5).abs() < 0.001);
        assert!((uv[1] - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_project_uv_clamps() {
        /* UV should be clamped to [0,1] */
        let uv = project_to_decal_uv([10.0, 10.0, 0.0], &unit_projector());
        assert!((0.0..=1.0).contains(&uv[0]));
        assert!((0.0..=1.0).contains(&uv[1]));
    }

    #[test]
    fn test_decal_depth_center() {
        /* Center depth should be 0.5 */
        let d = decal_depth([0.0, 0.0, 0.0], &unit_projector());
        assert!((d - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_clear() {
        /* Clear should remove all vertices and indices */
        let mut m = DecalMesh::new();
        m.vertices.push(DecalVertex {
            position: [0.0; 3],
            uv: [0.0; 2],
            depth: 0.0,
        });
        m.clear();
        assert_eq!(m.vertex_count(), 0);
    }

    #[test]
    fn test_triangle_count_empty() {
        /* Empty mesh should have zero triangles */
        assert_eq!(DecalMesh::new().triangle_count(), 0);
    }

    #[test]
    fn test_project_decal_filters_outside() {
        /* Points outside projector should be excluded */
        let positions = vec![[0.0f32; 3], [10.0, 0.0, 0.0]];
        let indices = vec![0u32];
        let mesh = project_decal(&positions, &indices, &unit_projector());
        assert_eq!(mesh.vertex_count(), 1);
    }

    #[test]
    fn test_depth_clamps() {
        /* Depth must be within [0,1] */
        let d = decal_depth([0.0, 0.0, 100.0], &unit_projector());
        assert!((0.0..=1.0).contains(&d));
    }
}
