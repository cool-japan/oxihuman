// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Edge fade — soft alpha falloff at screen and mesh edges.

use std::f32::consts::FRAC_2_PI;

/// Edge fade mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum EdgeFadeMode {
    Linear,
    Smooth,
    Cosine,
    Exponential,
}

/// Edge fade configuration.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct EdgeFadeConfig {
    pub mode: EdgeFadeMode,
    pub fade_width: f32,
    pub inner_opacity: f32,
    pub outer_opacity: f32,
    pub enabled: bool,
}

impl Default for EdgeFadeConfig {
    fn default() -> Self {
        Self {
            mode: EdgeFadeMode::Smooth,
            fade_width: 0.1,
            inner_opacity: 1.0,
            outer_opacity: 0.0,
            enabled: true,
        }
    }
}

/// Create default config.
#[allow(dead_code)]
pub fn default_edge_fade_config() -> EdgeFadeConfig {
    EdgeFadeConfig::default()
}

/// Compute edge fade alpha for a normalized edge distance `[0,1]`.
/// distance=0 is at the edge, distance=1 is fully inside.
#[allow(dead_code)]
pub fn edge_fade_alpha(distance: f32, cfg: &EdgeFadeConfig) -> f32 {
    if !cfg.enabled {
        return cfg.inner_opacity;
    }
    let t = (distance / cfg.fade_width.max(1e-6)).clamp(0.0, 1.0);
    let alpha = match cfg.mode {
        EdgeFadeMode::Linear => t,
        EdgeFadeMode::Smooth => t * t * (3.0 - 2.0 * t),
        EdgeFadeMode::Cosine => (1.0 - (std::f32::consts::PI * t).cos()) * 0.5,
        EdgeFadeMode::Exponential => 1.0 - (-5.0 * t).exp(),
    };
    cfg.outer_opacity + (cfg.inner_opacity - cfg.outer_opacity) * alpha
}

/// Rim fade factor using FRAC_2_PI for cosine normalization.
#[allow(dead_code)]
pub fn rim_fade(n_dot_v: f32, power: f32) -> f32 {
    let rim = 1.0 - n_dot_v.clamp(0.0, 1.0);
    let raw = rim.powf(power);
    // normalize by FRAC_2_PI so full rim integrates nicely
    (raw * FRAC_2_PI).min(1.0)
}

/// Screen edge fade for UV coordinates `[0,1]`.
#[allow(dead_code)]
pub fn screen_edge_fade(uv: [f32; 2], margin: f32) -> f32 {
    let fx = ((uv[0].min(1.0 - uv[0])) / margin.max(1e-6)).clamp(0.0, 1.0);
    let fy = ((uv[1].min(1.0 - uv[1])) / margin.max(1e-6)).clamp(0.0, 1.0);
    let t = fx.min(fy);
    t * t * (3.0 - 2.0 * t)
}

/// Check if a UV is within the fade zone.
#[allow(dead_code)]
pub fn is_in_fade_zone(uv: [f32; 2], margin: f32) -> bool {
    uv[0] < margin || uv[0] > 1.0 - margin || uv[1] < margin || uv[1] > 1.0 - margin
}

/// Export config to JSON-like string.
#[allow(dead_code)]
pub fn edge_fade_to_json(cfg: &EdgeFadeConfig) -> String {
    let mode = match cfg.mode {
        EdgeFadeMode::Linear => "linear",
        EdgeFadeMode::Smooth => "smooth",
        EdgeFadeMode::Cosine => "cosine",
        EdgeFadeMode::Exponential => "exponential",
    };
    format!(
        r#"{{"mode":"{}","fade_width":{:.4},"enabled":{}}}"#,
        mode, cfg.fade_width, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_fully_inside() {
        let cfg = default_edge_fade_config();
        let a = edge_fade_alpha(1.0, &cfg);
        assert!((a - cfg.inner_opacity).abs() < 1e-4);
    }

    #[test]
    fn alpha_at_edge_zero() {
        let cfg = default_edge_fade_config();
        let a = edge_fade_alpha(0.0, &cfg);
        assert!((a - cfg.outer_opacity).abs() < 1e-4);
    }

    #[test]
    fn alpha_disabled_returns_inner() {
        let cfg = EdgeFadeConfig {
            enabled: false,
            ..Default::default()
        };
        assert!((edge_fade_alpha(0.0, &cfg) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn linear_midpoint() {
        let cfg = EdgeFadeConfig {
            mode: EdgeFadeMode::Linear,
            fade_width: 1.0,
            ..Default::default()
        };
        let a = edge_fade_alpha(0.5, &cfg);
        assert!((a - 0.5).abs() < 1e-4);
    }

    #[test]
    fn screen_edge_center_is_one() {
        let a = screen_edge_fade([0.5, 0.5], 0.1);
        assert!((a - 1.0).abs() < 1e-4);
    }

    #[test]
    fn screen_edge_corner_near_zero() {
        let a = screen_edge_fade([0.0, 0.0], 0.1);
        assert!(a < 1e-4);
    }

    #[test]
    fn is_in_fade_zone_edge() {
        assert!(is_in_fade_zone([0.05, 0.5], 0.1));
    }

    #[test]
    fn is_in_fade_zone_center_false() {
        assert!(!is_in_fade_zone([0.5, 0.5], 0.1));
    }

    #[test]
    fn rim_fade_uses_frac2pi() {
        let r = rim_fade(0.0, 1.0);
        assert!((0.0..=1.0).contains(&r));
    }

    #[test]
    fn json_contains_mode() {
        let cfg = default_edge_fade_config();
        assert!(edge_fade_to_json(&cfg).contains("mode"));
    }
}
