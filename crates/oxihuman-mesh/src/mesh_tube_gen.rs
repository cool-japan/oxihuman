// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Open/closed tube (cylinder) mesh generation.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Configuration for tube generation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TubeGenConfig {
    /// Radius of the tube.
    pub radius: f32,
    /// Height of the tube.
    pub height: f32,
    /// Number of longitudinal segments.
    pub segments: usize,
    /// Whether to add circular caps.
    pub capped: bool,
}

/// Result of tube generation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TubeGenResult {
    /// Vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Per-vertex normals.
    pub normals: Vec<[f32; 3]>,
    /// Triangle indices.
    pub indices: Vec<u32>,
}

/// Returns the default [`TubeGenConfig`].
#[allow(dead_code)]
pub fn default_tube_gen_config() -> TubeGenConfig {
    TubeGenConfig {
        radius: 0.5,
        height: 1.0,
        segments: 16,
        capped: true,
    }
}

/// Generates a tube (cylinder) mesh from `config`.
#[allow(dead_code)]
pub fn generate_tube(config: &TubeGenConfig) -> TubeGenResult {
    let seg = config.segments.max(3);
    let r = config.radius.abs().max(f32::EPSILON);
    let half_h = (config.height * 0.5).abs();

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Side vertices: two rings (bottom + top)
    for ring in 0..=1u32 {
        let y = if ring == 0 { -half_h } else { half_h };
        for j in 0..=seg {
            let theta = 2.0 * PI * (j as f32) / (seg as f32);
            let nx = theta.cos();
            let nz = theta.sin();
            positions.push([r * nx, y, r * nz]);
            normals.push([nx, 0.0, nz]);
        }
    }

    let stride = (seg + 1) as u32;
    for j in 0..seg as u32 {
        let a = j;
        let b = j + 1;
        let c = stride + j;
        let d = stride + j + 1;
        indices.extend_from_slice(&[a, c, b, b, c, d]);
    }

    if config.capped {
        // Bottom cap centre
        let bot_center = positions.len() as u32;
        positions.push([0.0, -half_h, 0.0]);
        normals.push([0.0, -1.0, 0.0]);
        for j in 0..=seg {
            let theta = 2.0 * PI * (j as f32) / (seg as f32);
            positions.push([r * theta.cos(), -half_h, r * theta.sin()]);
            normals.push([0.0, -1.0, 0.0]);
        }
        let bot_rim_start = bot_center + 1;
        for j in 0..seg as u32 {
            indices.extend_from_slice(&[bot_center, bot_rim_start + j + 1, bot_rim_start + j]);
        }

        // Top cap centre
        let top_center = positions.len() as u32;
        positions.push([0.0, half_h, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
        for j in 0..=seg {
            let theta = 2.0 * PI * (j as f32) / (seg as f32);
            positions.push([r * theta.cos(), half_h, r * theta.sin()]);
            normals.push([0.0, 1.0, 0.0]);
        }
        let top_rim_start = top_center + 1;
        for j in 0..seg as u32 {
            indices.extend_from_slice(&[top_center, top_rim_start + j, top_rim_start + j + 1]);
        }
    }

    TubeGenResult { positions, normals, indices }
}

/// Returns the lateral (side) surface area of a tube.
#[allow(dead_code)]
pub fn tube_lateral_area(radius: f32, height: f32) -> f32 {
    2.0 * PI * radius * height
}

/// Returns the volume of a tube/cylinder.
#[allow(dead_code)]
pub fn tube_volume(radius: f32, height: f32) -> f32 {
    PI * radius * radius * height
}

/// Returns the approximate vertex count of an uncapped tube.
#[allow(dead_code)]
pub fn tube_vertex_count(segments: usize, capped: bool) -> usize {
    let side = 2 * (segments + 1);
    if capped {
        side + 2 * (segments + 2)
    } else {
        side
    }
}

/// Returns the approximate index count of a tube.
#[allow(dead_code)]
pub fn tube_index_count(segments: usize, capped: bool) -> usize {
    let side = segments * 6;
    if capped {
        side + 2 * segments * 3
    } else {
        side
    }
}

/// Serialises the result to a minimal JSON string.
#[allow(dead_code)]
pub fn tube_gen_to_json(result: &TubeGenResult) -> String {
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
        let cfg = default_tube_gen_config();
        assert_eq!(cfg.radius, 0.5);
        assert!(cfg.capped);
    }

    #[test]
    fn test_generate_tube() {
        let cfg = default_tube_gen_config();
        let result = generate_tube(&cfg);
        assert!(!result.positions.is_empty());
    }

    #[test]
    fn test_normals_count() {
        let cfg = default_tube_gen_config();
        let result = generate_tube(&cfg);
        assert_eq!(result.normals.len(), result.positions.len());
    }

    #[test]
    fn test_open_tube() {
        let cfg = TubeGenConfig { radius: 1.0, height: 2.0, segments: 8, capped: false };
        let result = generate_tube(&cfg);
        let expected_v = tube_vertex_count(cfg.segments, false);
        assert_eq!(result.positions.len(), expected_v);
    }

    #[test]
    fn test_lateral_area() {
        let area = tube_lateral_area(1.0, 1.0);
        assert!((area - 2.0 * std::f32::consts::PI).abs() < 1e-4);
    }

    #[test]
    fn test_volume() {
        let vol = tube_volume(1.0, 1.0);
        assert!((vol - std::f32::consts::PI).abs() < 1e-4);
    }

    #[test]
    fn test_index_count_open() {
        let cfg = TubeGenConfig { radius: 1.0, height: 1.0, segments: 8, capped: false };
        let result = generate_tube(&cfg);
        assert_eq!(result.indices.len(), tube_index_count(cfg.segments, false));
    }

    #[test]
    fn test_to_json() {
        let cfg = default_tube_gen_config();
        let result = generate_tube(&cfg);
        let json = tube_gen_to_json(&result);
        assert!(json.contains("vertices"));
    }

    #[test]
    fn test_indices_valid() {
        let cfg = default_tube_gen_config();
        let result = generate_tube(&cfg);
        let n = result.positions.len() as u32;
        assert!(result.indices.iter().all(|&i| i < n));
    }
}
