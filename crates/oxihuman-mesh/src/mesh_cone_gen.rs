// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cone mesh generation.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Configuration for cone generation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConeGenConfig {
    /// Radius of the base circle.
    pub base_radius: f32,
    /// Height from base to apex.
    pub height: f32,
    /// Number of segments around the base.
    pub segments: usize,
    /// Whether to include a base cap.
    pub capped: bool,
}

/// Result of cone generation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConeGenResult {
    /// Vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Per-vertex normals.
    pub normals: Vec<[f32; 3]>,
    /// Triangle indices.
    pub indices: Vec<u32>,
}

/// Returns the default [`ConeGenConfig`].
#[allow(dead_code)]
pub fn default_cone_gen_config() -> ConeGenConfig {
    ConeGenConfig {
        base_radius: 0.5,
        height: 1.0,
        segments: 16,
        capped: true,
    }
}

/// Generates a cone mesh from `config`.
#[allow(dead_code)]
pub fn generate_cone(config: &ConeGenConfig) -> ConeGenResult {
    let seg = config.segments.max(3);
    let r = config.base_radius.abs().max(f32::EPSILON);
    let h = config.height.abs().max(f32::EPSILON);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Slant normal tilt factor
    let slant = (r / h).atan();
    let ny_side = slant.sin();
    let nlen_side = slant.cos();

    // Side faces: apex + base ring
    let apex_idx = positions.len() as u32;
    positions.push([0.0, h, 0.0]);
    normals.push([0.0, ny_side, 0.0]); // approximate apex normal

    let base_start = positions.len() as u32;
    for j in 0..=seg {
        let theta = 2.0 * PI * (j as f32) / (seg as f32);
        let nx = theta.cos() * nlen_side;
        let nz = theta.sin() * nlen_side;
        positions.push([r * theta.cos(), 0.0, r * theta.sin()]);
        normals.push([nx, ny_side, nz]);
    }

    for j in 0..seg as u32 {
        indices.extend_from_slice(&[apex_idx, base_start + j, base_start + j + 1]);
    }

    // Base cap
    if config.capped {
        let center_idx = positions.len() as u32;
        positions.push([0.0, 0.0, 0.0]);
        normals.push([0.0, -1.0, 0.0]);
        let rim_start = positions.len() as u32;
        for j in 0..=seg {
            let theta = 2.0 * PI * (j as f32) / (seg as f32);
            positions.push([r * theta.cos(), 0.0, r * theta.sin()]);
            normals.push([0.0, -1.0, 0.0]);
        }
        for j in 0..seg as u32 {
            indices.extend_from_slice(&[center_idx, rim_start + j + 1, rim_start + j]);
        }
    }

    ConeGenResult {
        positions,
        normals,
        indices,
    }
}

/// Returns the vertex count for a cone.
#[allow(dead_code)]
pub fn cone_vertex_count(segments: usize, capped: bool) -> usize {
    let side = 1 + (segments + 1); // apex + base ring
    if capped {
        side + 1 + (segments + 1)
    } else {
        side
    }
}

/// Returns the index count for a cone.
#[allow(dead_code)]
pub fn cone_index_count(segments: usize, capped: bool) -> usize {
    let side = segments * 3;
    if capped {
        side + segments * 3
    } else {
        side
    }
}

/// Returns the slant height of the cone.
#[allow(dead_code)]
pub fn cone_slant_height(config: &ConeGenConfig) -> f32 {
    (config.base_radius * config.base_radius + config.height * config.height).sqrt()
}

/// Returns the volume of the cone.
#[allow(dead_code)]
pub fn cone_volume(config: &ConeGenConfig) -> f32 {
    (1.0 / 3.0) * PI * config.base_radius * config.base_radius * config.height
}

/// Serialises the result to a minimal JSON string.
#[allow(dead_code)]
pub fn cone_gen_to_json(result: &ConeGenResult) -> String {
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
        let cfg = default_cone_gen_config();
        assert_eq!(cfg.base_radius, 0.5);
        assert_eq!(cfg.height, 1.0);
    }

    #[test]
    fn test_generate_cone() {
        let cfg = default_cone_gen_config();
        let result = generate_cone(&cfg);
        assert!(!result.positions.is_empty());
    }

    #[test]
    fn test_vertex_count_capped() {
        let cfg = default_cone_gen_config();
        let result = generate_cone(&cfg);
        let expected = cone_vertex_count(cfg.segments, cfg.capped);
        assert_eq!(result.positions.len(), expected);
    }

    #[test]
    fn test_index_count_capped() {
        let cfg = default_cone_gen_config();
        let result = generate_cone(&cfg);
        let expected = cone_index_count(cfg.segments, cfg.capped);
        assert_eq!(result.indices.len(), expected);
    }

    #[test]
    fn test_normals_count() {
        let cfg = default_cone_gen_config();
        let result = generate_cone(&cfg);
        assert_eq!(result.normals.len(), result.positions.len());
    }

    #[test]
    fn test_slant_height() {
        let cfg = ConeGenConfig {
            base_radius: 3.0,
            height: 4.0,
            segments: 8,
            capped: false,
        };
        let s = cone_slant_height(&cfg);
        assert!((s - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_volume() {
        let cfg = default_cone_gen_config();
        assert!(cone_volume(&cfg) > 0.0);
    }

    #[test]
    fn test_open_cone() {
        let cfg = ConeGenConfig {
            base_radius: 1.0,
            height: 2.0,
            segments: 8,
            capped: false,
        };
        let result = generate_cone(&cfg);
        let expected_idx = cone_index_count(cfg.segments, false);
        assert_eq!(result.indices.len(), expected_idx);
    }

    #[test]
    fn test_to_json() {
        let cfg = default_cone_gen_config();
        let result = generate_cone(&cfg);
        let json = cone_gen_to_json(&result);
        assert!(json.contains("vertices"));
    }

    #[test]
    fn test_indices_valid() {
        let cfg = default_cone_gen_config();
        let result = generate_cone(&cfg);
        let n = result.positions.len() as u32;
        assert!(result.indices.iter().all(|&i| i < n));
    }
}
