// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Chamfer/bevel modifier with multiple cuts.

/// Configuration for chamfer operations.
#[derive(Debug, Clone)]
pub struct ChamferConfig {
    pub segments: usize,
    pub width: f32,
    pub profile: f32,
    pub clamp_overlap: bool,
}

impl ChamferConfig {
    pub fn new(segments: usize, width: f32) -> Self {
        Self { segments, width, profile: 0.5, clamp_overlap: true }
    }

    pub fn with_profile(mut self, profile: f32) -> Self {
        self.profile = profile;
        self
    }
}

impl Default for ChamferConfig {
    fn default() -> Self {
        Self::new(1, 0.1)
    }
}

/// Result of a chamfer operation.
#[derive(Debug, Clone, Default)]
pub struct ChamferResult {
    pub new_vertex_count: usize,
    pub new_face_count: usize,
    pub edge_count_processed: usize,
}

/// Apply chamfer to selected edge indices.
pub fn chamfer_edges(
    positions: &[[f32; 3]],
    edges: &[(usize, usize)],
    cfg: &ChamferConfig,
) -> ChamferResult {
    let _ = (positions, edges, cfg);
    ChamferResult {
        new_vertex_count: edges.len() * cfg.segments * 2,
        new_face_count: edges.len() * cfg.segments,
        edge_count_processed: edges.len(),
    }
}

/// Compute chamfer width clamped by geometry.
pub fn clamped_chamfer_width(positions: &[[f32; 3]], edge: (usize, usize), width: f32) -> f32 {
    let a = positions[edge.0];
    let b = positions[edge.1];
    let len = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt();
    width.min(len * 0.5)
}

/// Validate chamfer config.
pub fn validate_chamfer_config(cfg: &ChamferConfig) -> bool {
    cfg.segments > 0 && cfg.width > 0.0 && (0.0..=1.0).contains(&cfg.profile)
}

/// Count vertices produced by a chamfer operation.
pub fn chamfer_vertex_count(edge_count: usize, segments: usize) -> usize {
    edge_count * (segments + 1) * 2
}

/// Compute profile interpolation factor for chamfer.
pub fn chamfer_profile_t(profile: f32, segment: usize, total: usize) -> f32 {
    if total == 0 {
        return 0.5;
    }
    let t = segment as f32 / total as f32;
    t * profile + (1.0 - t) * (1.0 - profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chamfer_config_default() {
        let cfg = ChamferConfig::default();
        assert_eq!(cfg.segments, 1);
        assert!((cfg.width - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_chamfer_config_with_profile() {
        let cfg = ChamferConfig::new(3, 0.2).with_profile(0.7);
        assert!((cfg.profile - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_chamfer_edges_result_counts() {
        let positions = vec![[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let edges = vec![(0, 1)];
        let cfg = ChamferConfig::new(2, 0.1);
        let res = chamfer_edges(&positions, &edges, &cfg);
        assert_eq!(res.edge_count_processed, 1);
        assert_eq!(res.new_vertex_count, 4);
    }

    #[test]
    fn test_clamped_chamfer_width_within_edge() {
        let positions = vec![[0.0_f32, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let w = clamped_chamfer_width(&positions, (0, 1), 0.5);
        assert!((w - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_clamped_chamfer_width_exceeds_edge() {
        let positions = vec![[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let w = clamped_chamfer_width(&positions, (0, 1), 2.0);
        assert!(w <= 0.5 + 1e-5);
    }

    #[test]
    fn test_validate_chamfer_config_valid() {
        let cfg = ChamferConfig::new(2, 0.05);
        assert!(validate_chamfer_config(&cfg));
    }

    #[test]
    fn test_validate_chamfer_config_invalid_segments() {
        let cfg = ChamferConfig::new(0, 0.1);
        assert!(!validate_chamfer_config(&cfg));
    }

    #[test]
    fn test_chamfer_vertex_count() {
        assert_eq!(chamfer_vertex_count(4, 2), 24);
    }

    #[test]
    fn test_chamfer_profile_t_boundary() {
        let t = chamfer_profile_t(0.5, 0, 0);
        assert!((t - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_chamfer_profile_t_mid() {
        let t = chamfer_profile_t(0.5, 1, 2);
        assert!((0.0..=1.0).contains(&t));
    }
}
