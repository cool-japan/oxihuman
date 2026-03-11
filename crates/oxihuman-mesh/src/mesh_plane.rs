// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Flat grid plane mesh generation.

#![allow(dead_code)]

/// Configuration for plane mesh generation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PlaneConfig {
    /// Width along the X axis.
    pub width: f32,
    /// Depth along the Z axis.
    pub depth: f32,
    /// Number of subdivisions along X.
    pub subdivisions_x: usize,
    /// Number of subdivisions along Z.
    pub subdivisions_z: usize,
}

/// Result of plane generation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PlaneMesh {
    /// Vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Per-vertex normals (all pointing +Y).
    pub normals: Vec<[f32; 3]>,
    /// Per-vertex UV coordinates.
    pub uvs: Vec<[f32; 2]>,
    /// Triangle indices.
    pub indices: Vec<u32>,
}

/// Default plane configuration (1×1 with 1 subdivision each).
#[allow(dead_code)]
pub fn default_plane_config() -> PlaneConfig {
    PlaneConfig {
        width: 1.0,
        depth: 1.0,
        subdivisions_x: 1,
        subdivisions_z: 1,
    }
}

/// Generate a flat grid plane mesh.
#[allow(dead_code)]
pub fn generate_plane(config: &PlaneConfig) -> PlaneMesh {
    let nx = config.subdivisions_x.max(1);
    let nz = config.subdivisions_z.max(1);
    let w = config.width.abs().max(f32::EPSILON);
    let d = config.depth.abs().max(f32::EPSILON);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();

    for iz in 0..=nz {
        let z = -d * 0.5 + d * (iz as f32) / (nz as f32);
        let v = (iz as f32) / (nz as f32);
        for ix in 0..=nx {
            let x = -w * 0.5 + w * (ix as f32) / (nx as f32);
            let u = (ix as f32) / (nx as f32);
            positions.push([x, 0.0, z]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([u, v]);
        }
    }

    let stride = (nx + 1) as u32;
    let mut indices: Vec<u32> = Vec::new();
    for iz in 0..nz as u32 {
        for ix in 0..nx as u32 {
            let a = iz * stride + ix;
            let b = a + 1;
            let c = a + stride;
            let d = c + 1;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }

    PlaneMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}

/// Expected vertex count for a plane.
#[allow(dead_code)]
pub fn plane_vertex_count(subdivisions_x: usize, subdivisions_z: usize) -> usize {
    (subdivisions_x + 1) * (subdivisions_z + 1)
}

/// Expected index count for a plane.
#[allow(dead_code)]
pub fn plane_index_count(subdivisions_x: usize, subdivisions_z: usize) -> usize {
    subdivisions_x * subdivisions_z * 6
}

/// Surface area of the plane.
#[allow(dead_code)]
pub fn plane_area(config: &PlaneConfig) -> f32 {
    config.width * config.depth
}

/// Check all indices are in range.
#[allow(dead_code)]
pub fn plane_indices_valid(plane: &PlaneMesh) -> bool {
    let n = plane.positions.len() as u32;
    plane.indices.iter().all(|&i| i < n)
}

/// Check all UVs are in [0, 1].
#[allow(dead_code)]
pub fn plane_uvs_in_range(plane: &PlaneMesh) -> bool {
    plane
        .uvs
        .iter()
        .all(|uv| (0.0..=1.0).contains(&uv[0]) && (0.0..=1.0).contains(&uv[1]))
}

/// Serialise as minimal JSON.
#[allow(dead_code)]
pub fn plane_to_json(plane: &PlaneMesh) -> String {
    format!(
        "{{\"vertices\":{},\"indices\":{}}}",
        plane.positions.len(),
        plane.indices.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_plane_config();
        assert_eq!(cfg.width, 1.0);
        assert_eq!(cfg.depth, 1.0);
    }

    #[test]
    fn test_generate_plane_vertices() {
        let cfg = default_plane_config();
        let plane = generate_plane(&cfg);
        assert!(!plane.positions.is_empty());
    }

    #[test]
    fn test_vertex_count_matches() {
        let cfg = PlaneConfig {
            width: 2.0,
            depth: 2.0,
            subdivisions_x: 4,
            subdivisions_z: 4,
        };
        let plane = generate_plane(&cfg);
        assert_eq!(plane.positions.len(), plane_vertex_count(4, 4));
    }

    #[test]
    fn test_index_count_matches() {
        let cfg = PlaneConfig {
            width: 2.0,
            depth: 2.0,
            subdivisions_x: 4,
            subdivisions_z: 4,
        };
        let plane = generate_plane(&cfg);
        assert_eq!(plane.indices.len(), plane_index_count(4, 4));
    }

    #[test]
    fn test_normals_all_up() {
        let cfg = default_plane_config();
        let plane = generate_plane(&cfg);
        for n in &plane.normals {
            assert!((n[1] - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn test_indices_valid() {
        let cfg = PlaneConfig {
            width: 1.0,
            depth: 1.0,
            subdivisions_x: 3,
            subdivisions_z: 3,
        };
        let plane = generate_plane(&cfg);
        assert!(plane_indices_valid(&plane));
    }

    #[test]
    fn test_uvs_in_range() {
        let cfg = default_plane_config();
        let plane = generate_plane(&cfg);
        assert!(plane_uvs_in_range(&plane));
    }

    #[test]
    fn test_area() {
        let cfg = PlaneConfig {
            width: 3.0,
            depth: 4.0,
            subdivisions_x: 1,
            subdivisions_z: 1,
        };
        assert!((plane_area(&cfg) - 12.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let cfg = default_plane_config();
        let plane = generate_plane(&cfg);
        let json = plane_to_json(&plane);
        assert!(json.contains("vertices"));
    }

    #[test]
    fn test_normals_count_matches() {
        let cfg = default_plane_config();
        let plane = generate_plane(&cfg);
        assert_eq!(plane.normals.len(), plane.positions.len());
    }
}
