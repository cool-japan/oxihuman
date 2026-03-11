// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Torus mesh generation.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Configuration for torus generation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TorusGenConfig {
    /// Distance from the centre of the tube to the centre of the torus.
    pub major_radius: f32,
    /// Radius of the tube.
    pub minor_radius: f32,
    /// Number of segments around the major ring.
    pub major_segments: usize,
    /// Number of segments around the tube cross-section.
    pub minor_segments: usize,
}

/// Result of torus generation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TorusGenResult {
    /// Vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Per-vertex normals.
    pub normals: Vec<[f32; 3]>,
    /// Triangle indices.
    pub indices: Vec<u32>,
}

/// Returns the default [`TorusGenConfig`].
#[allow(dead_code)]
pub fn default_torus_gen_config() -> TorusGenConfig {
    TorusGenConfig {
        major_radius: 1.0,
        minor_radius: 0.25,
        major_segments: 32,
        minor_segments: 16,
    }
}

/// Generates a torus mesh from `config`.
#[allow(dead_code)]
pub fn generate_torus(config: &TorusGenConfig) -> TorusGenResult {
    let maj = config.major_segments.max(3);
    let min = config.minor_segments.max(3);
    let r = config.major_radius.abs().max(f32::EPSILON);
    let t = config.minor_radius.abs().max(f32::EPSILON);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();

    for i in 0..=maj {
        let phi = 2.0 * PI * (i as f32) / (maj as f32);
        let cp = phi.cos();
        let sp = phi.sin();
        for j in 0..=min {
            let theta = 2.0 * PI * (j as f32) / (min as f32);
            let ct = theta.cos();
            let st = theta.sin();
            let nx = cp * ct;
            let ny = st;
            let nz = sp * ct;
            positions.push([(r + t * ct) * cp, t * st, (r + t * ct) * sp]);
            normals.push([nx, ny, nz]);
        }
    }

    let stride = (min + 1) as u32;
    let mut indices: Vec<u32> = Vec::new();
    for i in 0..maj as u32 {
        for j in 0..min as u32 {
            let a = i * stride + j;
            let b = a + 1;
            let c = a + stride;
            let d = c + 1;
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }

    TorusGenResult {
        positions,
        normals,
        indices,
    }
}

/// Returns the vertex count for a torus with given segments.
#[allow(dead_code)]
pub fn torus_vertex_count(major_segments: usize, minor_segments: usize) -> usize {
    (major_segments + 1) * (minor_segments + 1)
}

/// Returns the index count for a torus with given segments.
#[allow(dead_code)]
pub fn torus_index_count(major_segments: usize, minor_segments: usize) -> usize {
    major_segments * minor_segments * 6
}

/// Returns the surface area of a torus.
#[allow(dead_code)]
pub fn torus_surface_area(major_radius: f32, minor_radius: f32) -> f32 {
    4.0 * PI * PI * major_radius * minor_radius
}

/// Returns the volume of a torus.
#[allow(dead_code)]
pub fn torus_volume(major_radius: f32, minor_radius: f32) -> f32 {
    2.0 * PI * PI * major_radius * minor_radius * minor_radius
}

/// Serialises the result to a minimal JSON string.
#[allow(dead_code)]
pub fn torus_gen_to_json(result: &TorusGenResult) -> String {
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
    fn test_default_config() {
        let cfg = default_torus_gen_config();
        assert_eq!(cfg.major_radius, 1.0);
        assert_eq!(cfg.minor_radius, 0.25);
    }

    #[test]
    fn test_generate_produces_vertices() {
        let cfg = default_torus_gen_config();
        let result = generate_torus(&cfg);
        assert!(!result.positions.is_empty());
    }

    #[test]
    fn test_vertex_count() {
        let cfg = default_torus_gen_config();
        let result = generate_torus(&cfg);
        let expected = torus_vertex_count(cfg.major_segments, cfg.minor_segments);
        assert_eq!(result.positions.len(), expected);
    }

    #[test]
    fn test_index_count() {
        let cfg = default_torus_gen_config();
        let result = generate_torus(&cfg);
        let expected = torus_index_count(cfg.major_segments, cfg.minor_segments);
        assert_eq!(result.indices.len(), expected);
    }

    #[test]
    fn test_normals_count() {
        let cfg = default_torus_gen_config();
        let result = generate_torus(&cfg);
        assert_eq!(result.normals.len(), result.positions.len());
    }

    #[test]
    fn test_surface_area_positive() {
        let area = torus_surface_area(1.0, 0.25);
        assert!(area > 0.0);
    }

    #[test]
    fn test_volume_positive() {
        let vol = torus_volume(1.0, 0.25);
        assert!(vol > 0.0);
    }

    #[test]
    fn test_to_json() {
        let cfg = default_torus_gen_config();
        let result = generate_torus(&cfg);
        let json = torus_gen_to_json(&result);
        assert!(json.contains("vertices"));
    }

    #[test]
    fn test_small_torus() {
        let cfg = TorusGenConfig {
            major_radius: 2.0,
            minor_radius: 0.5,
            major_segments: 4,
            minor_segments: 4,
        };
        let result = generate_torus(&cfg);
        assert!(!result.indices.is_empty());
    }

    #[test]
    fn test_indices_are_valid() {
        let cfg = default_torus_gen_config();
        let result = generate_torus(&cfg);
        let n = result.positions.len() as u32;
        assert!(result.indices.iter().all(|&i| i < n));
    }
}
