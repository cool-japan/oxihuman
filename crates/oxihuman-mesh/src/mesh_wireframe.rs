// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Wireframe mesh generation modifier.

/// Configuration for wireframe generation.
#[derive(Debug, Clone)]
pub struct WireframeConfig {
    pub thickness: f32,
    pub use_even_offset: bool,
    pub use_relative_offset: bool,
    pub use_boundary: bool,
    pub use_replace: bool,
    pub material_offset: i32,
    pub crease_weight: f32,
}

impl Default for WireframeConfig {
    fn default() -> Self {
        Self {
            thickness: 0.02,
            use_even_offset: false,
            use_relative_offset: false,
            use_boundary: true,
            use_replace: true,
            material_offset: 0,
            crease_weight: 0.0,
        }
    }
}

impl WireframeConfig {
    pub fn new(thickness: f32) -> Self {
        Self { thickness, ..Self::default() }
    }
}

/// Result of wireframe generation.
#[derive(Debug, Clone, Default)]
pub struct WireframeResult {
    pub vertex_count: usize,
    pub quad_face_count: usize,
    pub edge_count: usize,
}

/// Compute the offset for a given edge length.
pub fn wireframe_offset(edge_length: f32, cfg: &WireframeConfig) -> f32 {
    if cfg.use_relative_offset {
        edge_length * cfg.thickness
    } else {
        cfg.thickness
    }
}

/// Estimate vertex count for wireframe of a triangular mesh.
pub fn estimate_wireframe_vertices(edge_count: usize, cfg: &WireframeConfig) -> usize {
    let _ = cfg;
    edge_count * 4
}

/// Generate wireframe quad faces for a set of edges.
pub fn wireframe_quads(
    positions: &[[f32; 3]],
    edges: &[(usize, usize)],
    cfg: &WireframeConfig,
) -> WireframeResult {
    let _ = positions;
    let _ = cfg;
    WireframeResult {
        vertex_count: edges.len() * 4,
        quad_face_count: edges.len(),
        edge_count: edges.len(),
    }
}

/// Validate wireframe config.
pub fn validate_wireframe_config(cfg: &WireframeConfig) -> bool {
    cfg.thickness > 0.0 && (0.0..=1.0).contains(&cfg.crease_weight)
}

/// Compute per-edge wireframe thickness accounting for relative mode.
pub fn per_edge_thickness(positions: &[[f32; 3]], edge: (usize, usize), cfg: &WireframeConfig) -> f32 {
    let a = positions[edge.0];
    let b = positions[edge.1];
    let len = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt();
    wireframe_offset(len, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wireframe_config_default() {
        let cfg = WireframeConfig::default();
        assert!((cfg.thickness - 0.02).abs() < 1e-6);
    }

    #[test]
    fn test_wireframe_offset_absolute() {
        let cfg = WireframeConfig::new(0.05);
        assert!((wireframe_offset(2.0, &cfg) - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_wireframe_offset_relative() {
        let cfg = WireframeConfig { use_relative_offset: true, thickness: 0.1, ..Default::default() };
        assert!((wireframe_offset(2.0, &cfg) - 0.2).abs() < 1e-5);
    }

    #[test]
    fn test_estimate_wireframe_vertices() {
        let cnt = estimate_wireframe_vertices(10, &WireframeConfig::default());
        assert_eq!(cnt, 40);
    }

    #[test]
    fn test_wireframe_quads_count() {
        let pos = vec![[0.0_f32; 3]; 4];
        let edges = vec![(0, 1), (1, 2), (2, 3)];
        let cfg = WireframeConfig::default();
        let res = wireframe_quads(&pos, &edges, &cfg);
        assert_eq!(res.quad_face_count, 3);
    }

    #[test]
    fn test_validate_config_valid() {
        let cfg = WireframeConfig::default();
        assert!(validate_wireframe_config(&cfg));
    }

    #[test]
    fn test_validate_config_zero_thickness() {
        let cfg = WireframeConfig { thickness: 0.0, ..Default::default() };
        assert!(!validate_wireframe_config(&cfg));
    }

    #[test]
    fn test_per_edge_thickness_absolute() {
        let pos = vec![[0.0_f32; 3], [3.0, 4.0, 0.0]];
        let cfg = WireframeConfig::new(0.1);
        let t = per_edge_thickness(&pos, (0, 1), &cfg);
        assert!((t - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_per_edge_thickness_relative() {
        let pos = vec![[0.0_f32; 3], [1.0, 0.0, 0.0]];
        let cfg = WireframeConfig { use_relative_offset: true, thickness: 0.5, ..Default::default() };
        let t = per_edge_thickness(&pos, (0, 1), &cfg);
        assert!((t - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_wireframe_quads_vertex_count() {
        let pos = vec![[0.0_f32; 3]; 2];
        let edges = vec![(0, 1)];
        let cfg = WireframeConfig::default();
        let res = wireframe_quads(&pos, &edges, &cfg);
        assert_eq!(res.vertex_count, 4);
    }
}
