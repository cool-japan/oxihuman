// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Edge outline post-process — selection highlight / toon outline.

/// Outline config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeOutlineConfig {
    pub color: [f32; 4],
    pub thickness_px: f32,
    pub depth_threshold: f32,
    pub normal_threshold: f32,
    pub enabled: bool,
}

impl Default for EdgeOutlineConfig {
    fn default() -> Self {
        Self {
            color: [1.0, 0.5, 0.0, 1.0],
            thickness_px: 2.0,
            depth_threshold: 0.01,
            normal_threshold: 0.8,
            enabled: true,
        }
    }
}

#[allow(dead_code)]
pub fn new_edge_outline() -> EdgeOutlineConfig {
    EdgeOutlineConfig::default()
}

#[allow(dead_code)]
pub fn eo_set_color(cfg: &mut EdgeOutlineConfig, rgba: [f32; 4]) {
    cfg.color = rgba;
}

#[allow(dead_code)]
pub fn eo_set_thickness(cfg: &mut EdgeOutlineConfig, px: f32) {
    cfg.thickness_px = px.max(0.0);
}

#[allow(dead_code)]
pub fn eo_set_depth_threshold(cfg: &mut EdgeOutlineConfig, v: f32) {
    cfg.depth_threshold = v.max(0.0);
}

#[allow(dead_code)]
pub fn eo_set_normal_threshold(cfg: &mut EdgeOutlineConfig, v: f32) {
    cfg.normal_threshold = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn eo_set_enabled(cfg: &mut EdgeOutlineConfig, en: bool) {
    cfg.enabled = en;
}

#[allow(dead_code)]
pub fn eo_reset(cfg: &mut EdgeOutlineConfig) {
    *cfg = EdgeOutlineConfig::default();
}

/// Determine if a depth discontinuity is an edge.
#[allow(dead_code)]
pub fn eo_is_depth_edge(cfg: &EdgeOutlineConfig, depth_center: f32, depth_neighbour: f32) -> bool {
    if !cfg.enabled {
        return false;
    }
    (depth_center - depth_neighbour).abs() > cfg.depth_threshold
}

/// Determine if a normal discontinuity is an edge.
#[allow(dead_code)]
pub fn eo_is_normal_edge(
    cfg: &EdgeOutlineConfig,
    normal_center: [f32; 3],
    normal_neighbour: [f32; 3],
) -> bool {
    if !cfg.enabled {
        return false;
    }
    let dot = normal_center[0] * normal_neighbour[0]
        + normal_center[1] * normal_neighbour[1]
        + normal_center[2] * normal_neighbour[2];
    dot < cfg.normal_threshold
}

/// Combine depth and normal tests.
#[allow(dead_code)]
pub fn eo_is_edge(
    cfg: &EdgeOutlineConfig,
    depth_c: f32,
    depth_n: f32,
    normal_c: [f32; 3],
    normal_n: [f32; 3],
) -> bool {
    eo_is_depth_edge(cfg, depth_c, depth_n) || eo_is_normal_edge(cfg, normal_c, normal_n)
}

/// Outline alpha (1.0 if edge, 0.0 otherwise), modulated by thickness.
#[allow(dead_code)]
pub fn eo_outline_alpha(cfg: &EdgeOutlineConfig, is_edge: bool) -> f32 {
    if is_edge && cfg.enabled {
        cfg.color[3] * (cfg.thickness_px / 8.0).min(1.0)
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn eo_to_json(cfg: &EdgeOutlineConfig) -> String {
    format!(
        "{{\"thickness_px\":{:.1},\"enabled\":{}}}",
        cfg.thickness_px, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_enabled() {
        assert!(new_edge_outline().enabled);
    }

    #[test]
    fn depth_edge_detected() {
        let cfg = new_edge_outline();
        assert!(eo_is_depth_edge(&cfg, 0.5, 0.6));
    }

    #[test]
    fn depth_no_edge_close_values() {
        let cfg = new_edge_outline();
        assert!(!eo_is_depth_edge(&cfg, 0.5, 0.5001));
    }

    #[test]
    fn disabled_never_edge() {
        let mut cfg = new_edge_outline();
        eo_set_enabled(&mut cfg, false);
        assert!(!eo_is_depth_edge(&cfg, 0.0, 1.0));
    }

    #[test]
    fn normal_edge_perpendicular() {
        let cfg = new_edge_outline();
        let n_c = [1.0, 0.0, 0.0];
        let n_n = [0.0, 1.0, 0.0];
        assert!(eo_is_normal_edge(&cfg, n_c, n_n));
    }

    #[test]
    fn normal_no_edge_same() {
        let cfg = new_edge_outline();
        let n = [0.0, 0.0, 1.0];
        assert!(!eo_is_normal_edge(&cfg, n, n));
    }

    #[test]
    fn thickness_clamps_low() {
        let mut cfg = new_edge_outline();
        eo_set_thickness(&mut cfg, -10.0);
        assert!(cfg.thickness_px >= 0.0);
    }

    #[test]
    fn outline_alpha_zero_no_edge() {
        let cfg = new_edge_outline();
        assert!(eo_outline_alpha(&cfg, false) < 1e-6);
    }

    #[test]
    fn outline_alpha_positive_edge() {
        let cfg = new_edge_outline();
        assert!(eo_outline_alpha(&cfg, true) > 0.0);
    }

    #[test]
    fn json_has_thickness() {
        let j = eo_to_json(&new_edge_outline());
        assert!(j.contains("thickness_px"));
    }
}
