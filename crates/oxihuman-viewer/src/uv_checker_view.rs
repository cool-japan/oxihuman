// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! UV checker map overlay visualization.

/// UV checker visualization mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UvCheckerMode {
    Grid,
    ColorZones,
    Gradient,
    Numbered,
}

impl UvCheckerMode {
    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            UvCheckerMode::Grid => "grid",
            UvCheckerMode::ColorZones => "color_zones",
            UvCheckerMode::Gradient => "gradient",
            UvCheckerMode::Numbered => "numbered",
        }
    }
}

/// UV checker view configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvCheckerConfig {
    pub mode: UvCheckerMode,
    pub grid_divisions: u32,
    pub opacity: f32,
    pub show_seams: bool,
    pub enabled: bool,
}

impl Default for UvCheckerConfig {
    fn default() -> Self {
        UvCheckerConfig {
            mode: UvCheckerMode::Grid,
            grid_divisions: 8,
            opacity: 1.0,
            show_seams: true,
            enabled: false,
        }
    }
}

#[allow(dead_code)]
pub fn default_uv_checker_config() -> UvCheckerConfig {
    UvCheckerConfig::default()
}

#[allow(dead_code)]
pub fn uvc_set_mode(cfg: &mut UvCheckerConfig, mode: UvCheckerMode) {
    cfg.mode = mode;
}

#[allow(dead_code)]
pub fn uvc_set_grid_divisions(cfg: &mut UvCheckerConfig, n: u32) {
    cfg.grid_divisions = n.clamp(1, 64);
}

#[allow(dead_code)]
pub fn uvc_set_opacity(cfg: &mut UvCheckerConfig, v: f32) {
    cfg.opacity = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn uvc_enable(cfg: &mut UvCheckerConfig) {
    cfg.enabled = true;
}

#[allow(dead_code)]
pub fn uvc_disable(cfg: &mut UvCheckerConfig) {
    cfg.enabled = false;
}

/// Sample a grid checker color at UV coordinate (u, v).
#[allow(dead_code)]
pub fn uvc_grid_sample(u: f32, v: f32, divisions: u32) -> [f32; 3] {
    let d = divisions as f32;
    let cu = (u * d).floor() as i32;
    let cv = (v * d).floor() as i32;
    let dark = (cu + cv) % 2 == 0;
    if dark {
        [0.2, 0.2, 0.2]
    } else {
        [0.9, 0.9, 0.9]
    }
}

/// Sample a gradient color at UV.
#[allow(dead_code)]
pub fn uvc_gradient_sample(u: f32, v: f32) -> [f32; 3] {
    [u.clamp(0.0, 1.0), v.clamp(0.0, 1.0), 0.5]
}

/// Sample color zones (quadrant-based).
#[allow(dead_code)]
pub fn uvc_zone_sample(u: f32, v: f32) -> [f32; 3] {
    let hu = u >= 0.5;
    let hv = v >= 0.5;
    match (hu, hv) {
        (false, false) => [1.0, 0.0, 0.0],
        (true, false) => [0.0, 1.0, 0.0],
        (false, true) => [0.0, 0.0, 1.0],
        (true, true) => [1.0, 1.0, 0.0],
    }
}

#[allow(dead_code)]
pub fn uvc_to_json(cfg: &UvCheckerConfig) -> String {
    format!(
        r#"{{"mode":"{}","grid_divisions":{},"opacity":{:.4},"enabled":{}}}"#,
        cfg.mode.name(),
        cfg.grid_divisions,
        cfg.opacity,
        cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        assert!(!default_uv_checker_config().enabled);
    }

    #[test]
    fn grid_sample_origin_dark() {
        let c = uvc_grid_sample(0.0, 0.0, 8);
        assert!(c[0] < 0.5);
    }

    #[test]
    fn grid_sample_alternates() {
        let a = uvc_grid_sample(0.0, 0.0, 8);
        let b = uvc_grid_sample(0.126, 0.0, 8);
        assert!((a[0] - b[0]).abs() > 0.1);
    }

    #[test]
    fn gradient_sample_corner() {
        let c = uvc_gradient_sample(1.0, 0.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn zone_sample_quadrants_distinct() {
        let q0 = uvc_zone_sample(0.0, 0.0);
        let q1 = uvc_zone_sample(1.0, 0.0);
        let q2 = uvc_zone_sample(0.0, 1.0);
        let q3 = uvc_zone_sample(1.0, 1.0);
        assert_ne!(q0[0], q1[0]);
        assert_ne!(q2[0], q3[0]);
    }

    #[test]
    fn set_grid_divisions_clamps() {
        let mut cfg = default_uv_checker_config();
        uvc_set_grid_divisions(&mut cfg, 0);
        assert_eq!(cfg.grid_divisions, 1);
    }

    #[test]
    fn set_opacity_clamps() {
        let mut cfg = default_uv_checker_config();
        uvc_set_opacity(&mut cfg, 2.0);
        assert!((cfg.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn enable_disable() {
        let mut cfg = default_uv_checker_config();
        uvc_enable(&mut cfg);
        assert!(cfg.enabled);
        uvc_disable(&mut cfg);
        assert!(!cfg.enabled);
    }

    #[test]
    fn to_json_has_mode() {
        assert!(uvc_to_json(&default_uv_checker_config()).contains("mode"));
    }
}
