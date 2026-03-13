// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Grid fade — distance-based grid opacity fade parameters.

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct GridFadeConfig {
    /// Distance at which the grid begins to fade (world units).
    pub fade_start: f32,
    /// Distance at which the grid is fully transparent.
    pub fade_end: f32,
    /// Minimum opacity at full fade (0 = invisible, 1 = no fade).
    pub min_opacity: f32,
    /// Base opacity of the grid.
    pub base_opacity: f32,
}

impl Default for GridFadeConfig {
    fn default() -> Self {
        Self {
            fade_start: 5.0,
            fade_end: 20.0,
            min_opacity: 0.0,
            base_opacity: 0.6,
        }
    }
}

#[allow(dead_code)]
pub fn new_grid_fade_config() -> GridFadeConfig {
    GridFadeConfig::default()
}

/// Compute opacity for a grid line at the given distance from the camera.
#[allow(dead_code)]
pub fn grid_fade_opacity(distance: f32, cfg: &GridFadeConfig) -> f32 {
    if distance <= cfg.fade_start {
        cfg.base_opacity
    } else if distance >= cfg.fade_end {
        cfg.min_opacity
    } else {
        let t = (distance - cfg.fade_start) / (cfg.fade_end - cfg.fade_start);
        let t = t.clamp(0.0, 1.0);
        cfg.base_opacity * (1.0 - t) + cfg.min_opacity * t
    }
}

#[allow(dead_code)]
pub fn gf_set_fade_start(cfg: &mut GridFadeConfig, v: f32) {
    cfg.fade_start = v.max(0.0);
}

#[allow(dead_code)]
pub fn gf_set_fade_end(cfg: &mut GridFadeConfig, v: f32) {
    cfg.fade_end = v.max(cfg.fade_start + 0.001);
}

#[allow(dead_code)]
pub fn gf_set_base_opacity(cfg: &mut GridFadeConfig, v: f32) {
    cfg.base_opacity = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn gf_set_min_opacity(cfg: &mut GridFadeConfig, v: f32) {
    cfg.min_opacity = v.clamp(0.0, 1.0);
}

/// Fade range (end - start).
#[allow(dead_code)]
pub fn gf_fade_range(cfg: &GridFadeConfig) -> f32 {
    (cfg.fade_end - cfg.fade_start).max(0.0)
}

/// Returns the opacity at the midpoint of the fade range.
#[allow(dead_code)]
pub fn gf_mid_opacity(cfg: &GridFadeConfig) -> f32 {
    let mid = (cfg.fade_start + cfg.fade_end) * 0.5;
    grid_fade_opacity(mid, cfg)
}

#[allow(dead_code)]
pub fn gf_to_json(cfg: &GridFadeConfig) -> String {
    format!(
        "{{\"fade_start\":{:.2},\"fade_end\":{:.2},\"base_opacity\":{:.3},\"min_opacity\":{:.3}}}",
        cfg.fade_start, cfg.fade_end, cfg.base_opacity, cfg.min_opacity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opacity_before_fade_start() {
        let cfg = new_grid_fade_config();
        let op = grid_fade_opacity(1.0, &cfg);
        assert!((op - cfg.base_opacity).abs() < 1e-5);
    }

    #[test]
    fn opacity_after_fade_end() {
        let cfg = new_grid_fade_config();
        let op = grid_fade_opacity(100.0, &cfg);
        assert!((op - cfg.min_opacity).abs() < 1e-5);
    }

    #[test]
    fn opacity_decreases_with_distance() {
        let cfg = new_grid_fade_config();
        let a = grid_fade_opacity(cfg.fade_start + 1.0, &cfg);
        let b = grid_fade_opacity(cfg.fade_end - 1.0, &cfg);
        assert!(a > b);
    }

    #[test]
    fn set_base_opacity_clamps() {
        let mut cfg = new_grid_fade_config();
        gf_set_base_opacity(&mut cfg, 5.0);
        assert!((cfg.base_opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_min_opacity_clamps_negative() {
        let mut cfg = new_grid_fade_config();
        gf_set_min_opacity(&mut cfg, -1.0);
        assert!(cfg.min_opacity < 1e-6);
    }

    #[test]
    fn fade_range_positive() {
        let cfg = new_grid_fade_config();
        assert!(gf_fade_range(&cfg) > 0.0);
    }

    #[test]
    fn mid_opacity_between_bounds() {
        let cfg = new_grid_fade_config();
        let m = gf_mid_opacity(&cfg);
        assert!(m >= cfg.min_opacity && m <= cfg.base_opacity);
    }

    #[test]
    fn set_fade_start_non_negative() {
        let mut cfg = new_grid_fade_config();
        gf_set_fade_start(&mut cfg, -5.0);
        assert!(cfg.fade_start >= 0.0);
    }

    #[test]
    fn json_has_keys() {
        let j = gf_to_json(&new_grid_fade_config());
        assert!(j.contains("fade_start") && j.contains("base_opacity"));
    }
}
