// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! UV sphere (lat/lon grid) mesh generation.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Configuration for UV sphere generation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvSphereConfig {
    /// Sphere radius.
    pub radius: f32,
    /// Number of longitude segments.
    pub longitude_segments: usize,
    /// Number of latitude segments (rings between poles).
    pub latitude_segments: usize,
}

/// Result of UV sphere generation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvSphereResult {
    /// Vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Per-vertex normals.
    pub normals: Vec<[f32; 3]>,
    /// Per-vertex UV coordinates.
    pub uvs: Vec<[f32; 2]>,
    /// Triangle indices.
    pub indices: Vec<u32>,
}

/// Default UV sphere configuration.
#[allow(dead_code)]
pub fn default_uv_sphere_config() -> UvSphereConfig {
    UvSphereConfig {
        radius: 1.0,
        longitude_segments: 32,
        latitude_segments: 16,
    }
}

/// Generate a UV sphere mesh.
#[allow(dead_code)]
pub fn generate_uv_sphere(config: &UvSphereConfig) -> UvSphereResult {
    let lon = config.longitude_segments.max(3);
    let lat = config.latitude_segments.max(2);
    let r = config.radius.abs().max(f32::EPSILON);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();

    for i in 0..=lat {
        let phi = PI * (i as f32) / (lat as f32); // 0 = north pole, PI = south pole
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();
        for j in 0..=lon {
            let theta = 2.0 * PI * (j as f32) / (lon as f32);
            let nx = sin_phi * theta.cos();
            let ny = cos_phi;
            let nz = sin_phi * theta.sin();
            positions.push([r * nx, r * ny, r * nz]);
            normals.push([nx, ny, nz]);
            uvs.push([(j as f32) / (lon as f32), (i as f32) / (lat as f32)]);
        }
    }

    let stride = (lon + 1) as u32;
    let mut indices: Vec<u32> = Vec::new();
    for i in 0..lat as u32 {
        for j in 0..lon as u32 {
            let a = i * stride + j;
            let b = a + 1;
            let c = a + stride;
            let d = c + 1;
            if i != 0 {
                indices.extend_from_slice(&[a, b, c]);
            }
            if i + 1 != lat as u32 {
                indices.extend_from_slice(&[b, d, c]);
            }
        }
    }

    UvSphereResult {
        positions,
        normals,
        uvs,
        indices,
    }
}

/// Expected vertex count for a UV sphere.
#[allow(dead_code)]
pub fn uv_sphere_vertex_count(longitude: usize, latitude: usize) -> usize {
    (latitude + 1) * (longitude + 1)
}

/// Surface area of a sphere.
#[allow(dead_code)]
pub fn sphere_surface_area(radius: f32) -> f32 {
    4.0 * PI * radius * radius
}

/// Volume of a sphere.
#[allow(dead_code)]
pub fn sphere_volume(radius: f32) -> f32 {
    (4.0 / 3.0) * PI * radius * radius * radius
}

/// Check that all normals are approximately unit length.
#[allow(dead_code)]
pub fn normals_unit(result: &UvSphereResult) -> bool {
    result.normals.iter().all(|n| {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        (len - 1.0).abs() < 1e-4
    })
}

/// Check that all indices are in range.
#[allow(dead_code)]
pub fn indices_valid(result: &UvSphereResult) -> bool {
    let n = result.positions.len() as u32;
    result.indices.iter().all(|&i| i < n)
}

/// Serialise result as minimal JSON.
#[allow(dead_code)]
pub fn uv_sphere_to_json(result: &UvSphereResult) -> String {
    format!(
        "{{\"vertices\":{},\"indices\":{}}}",
        result.positions.len(),
        result.indices.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_radius() {
        let cfg = default_uv_sphere_config();
        assert_eq!(cfg.radius, 1.0);
    }

    #[test]
    fn test_generate_produces_vertices() {
        let cfg = default_uv_sphere_config();
        let result = generate_uv_sphere(&cfg);
        assert!(!result.positions.is_empty());
    }

    #[test]
    fn test_vertex_count_matches() {
        let cfg = default_uv_sphere_config();
        let result = generate_uv_sphere(&cfg);
        let expected = uv_sphere_vertex_count(cfg.longitude_segments, cfg.latitude_segments);
        assert_eq!(result.positions.len(), expected);
    }

    #[test]
    fn test_normals_are_unit() {
        let cfg = UvSphereConfig {
            radius: 1.0,
            longitude_segments: 8,
            latitude_segments: 4,
        };
        let result = generate_uv_sphere(&cfg);
        assert!(normals_unit(&result));
    }

    #[test]
    fn test_indices_valid() {
        let cfg = UvSphereConfig {
            radius: 1.0,
            longitude_segments: 8,
            latitude_segments: 4,
        };
        let result = generate_uv_sphere(&cfg);
        assert!(indices_valid(&result));
    }

    #[test]
    fn test_surface_area_positive() {
        assert!(sphere_surface_area(1.0) > 0.0);
    }

    #[test]
    fn test_volume_positive() {
        assert!(sphere_volume(1.0) > 0.0);
    }

    #[test]
    fn test_uvs_in_range() {
        let cfg = UvSphereConfig {
            radius: 2.0,
            longitude_segments: 8,
            latitude_segments: 4,
        };
        let result = generate_uv_sphere(&cfg);
        for uv in &result.uvs {
            assert!((0.0..=1.0).contains(&uv[0]));
            assert!((0.0..=1.0).contains(&uv[1]));
        }
    }

    #[test]
    fn test_to_json() {
        let cfg = default_uv_sphere_config();
        let result = generate_uv_sphere(&cfg);
        let json = uv_sphere_to_json(&result);
        assert!(json.contains("vertices"));
    }

    #[test]
    fn test_normals_count_matches_positions() {
        let cfg = default_uv_sphere_config();
        let result = generate_uv_sphere(&cfg);
        assert_eq!(result.normals.len(), result.positions.len());
    }
}
