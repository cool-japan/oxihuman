// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Depth-of-field v2 — physically-based CoC calculation with bokeh shape hint.

use std::f32::consts::PI;

/// Bokeh shape.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BokehShape {
    Circle,
    Hexagon,
    Octagon,
}

/// DOF v2 configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct DofV2Config {
    /// Focal length in mm.
    pub focal_length_mm: f32,
    /// Aperture (f-number).
    pub f_number: f32,
    /// Focus distance in metres.
    pub focus_distance_m: f32,
    /// Sensor height in mm (full-frame = 24 mm).
    pub sensor_height_mm: f32,
    pub bokeh: BokehShape,
    pub max_coc_px: f32,
}

impl Default for DofV2Config {
    fn default() -> Self {
        Self {
            focal_length_mm: 50.0,
            f_number: 2.8,
            focus_distance_m: 2.0,
            sensor_height_mm: 24.0,
            bokeh: BokehShape::Circle,
            max_coc_px: 16.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_dof_v2() -> DofV2Config {
    DofV2Config::default()
}

#[allow(dead_code)]
pub fn dv2_set_focal(cfg: &mut DofV2Config, mm: f32) {
    cfg.focal_length_mm = mm.max(1.0);
}

#[allow(dead_code)]
pub fn dv2_set_aperture(cfg: &mut DofV2Config, f: f32) {
    cfg.f_number = f.max(0.7);
}

#[allow(dead_code)]
pub fn dv2_set_focus(cfg: &mut DofV2Config, m: f32) {
    cfg.focus_distance_m = m.max(0.01);
}

#[allow(dead_code)]
pub fn dv2_reset(cfg: &mut DofV2Config) {
    *cfg = DofV2Config::default();
}

/// Circle-of-confusion radius in mm for a given scene depth in metres.
#[allow(dead_code)]
pub fn dv2_coc_mm(cfg: &DofV2Config, depth_m: f32) -> f32 {
    let f = cfg.focal_length_mm * 1e-3; // metres
    let d = cfg.focus_distance_m.max(1e-6);
    let z = depth_m.max(1e-6);
    let aperture = f / cfg.f_number;
    let coc = aperture * f * (1.0 / d - 1.0 / z).abs() / (1.0 - f / z).abs().max(1e-6);
    coc * 1000.0 // back to mm
}

/// Circle-of-confusion radius in normalised pixels (0..=max_coc_px).
#[allow(dead_code)]
pub fn dv2_coc_px(cfg: &DofV2Config, depth_m: f32, screen_height_px: f32) -> f32 {
    let coc_mm = dv2_coc_mm(cfg, depth_m);
    let px = coc_mm / cfg.sensor_height_mm * screen_height_px;
    px.min(cfg.max_coc_px)
}

/// Number of bokeh blades (hint for rendering).
#[allow(dead_code)]
pub fn dv2_blade_count(cfg: &DofV2Config) -> u32 {
    match cfg.bokeh {
        BokehShape::Circle => 0,
        BokehShape::Hexagon => 6,
        BokehShape::Octagon => 8,
    }
}

/// Depth-of-field range (near, far) in metres.
#[allow(dead_code)]
pub fn dv2_depth_range(cfg: &DofV2Config) -> (f32, f32) {
    let f_m = cfg.focal_length_mm * 1e-3;
    let nc = cfg.sensor_height_mm * 1e-3 / 1000.0; // very small NC
    let hyper = f_m * f_m / (cfg.f_number * nc) + f_m;
    let d = cfg.focus_distance_m;
    let near = (d * (hyper - f_m)) / (hyper + d - 2.0 * f_m);
    let far = if d >= hyper {
        f32::INFINITY
    } else {
        (d * (hyper - f_m)) / (hyper - d)
    };
    (near.max(0.0), far)
}

/// Bokeh area (circle or polygon approximation) in mm².
#[allow(dead_code)]
pub fn dv2_bokeh_area_mm2(cfg: &DofV2Config, depth_m: f32) -> f32 {
    let r = dv2_coc_mm(cfg, depth_m);
    match cfg.bokeh {
        BokehShape::Circle => PI * r * r,
        BokehShape::Hexagon => 3.0 * (3.0f32).sqrt() / 2.0 * r * r,
        BokehShape::Octagon => 2.0 * (2.0f32).sqrt() * r * r,
    }
}

#[allow(dead_code)]
pub fn dv2_to_json(cfg: &DofV2Config) -> String {
    format!(
        "{{\"focal_mm\":{:.1},\"f_number\":{:.2},\"focus_m\":{:.3},\"max_coc_px\":{:.1}}}",
        cfg.focal_length_mm, cfg.f_number, cfg.focus_distance_m, cfg.max_coc_px
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_valid() {
        let cfg = new_dof_v2();
        assert!(cfg.focal_length_mm > 0.0);
        assert!(cfg.f_number > 0.0);
    }

    #[test]
    fn coc_zero_at_focus_distance() {
        let cfg = new_dof_v2();
        let coc = dv2_coc_px(&cfg, cfg.focus_distance_m, 720.0);
        assert!(coc < 0.5, "coc at focus should be near zero, got {coc}");
    }

    #[test]
    fn coc_grows_with_distance() {
        let cfg = new_dof_v2();
        let near = dv2_coc_px(&cfg, 0.5, 720.0);
        let far = dv2_coc_px(&cfg, 10.0, 720.0);
        assert!(far > near || near > 0.0);
    }

    #[test]
    fn blade_count_circle_zero() {
        assert_eq!(dv2_blade_count(&new_dof_v2()), 0);
    }

    #[test]
    fn blade_count_hexagon() {
        let mut cfg = new_dof_v2();
        cfg.bokeh = BokehShape::Hexagon;
        assert_eq!(dv2_blade_count(&cfg), 6);
    }

    #[test]
    fn depth_range_near_less_than_focus() {
        let cfg = new_dof_v2();
        let (near, _) = dv2_depth_range(&cfg);
        assert!(near < cfg.focus_distance_m);
    }

    #[test]
    fn set_focal_clamps() {
        let mut cfg = new_dof_v2();
        dv2_set_focal(&mut cfg, 0.0);
        assert!(cfg.focal_length_mm >= 1.0);
    }

    #[test]
    fn reset_restores_defaults() {
        let mut cfg = new_dof_v2();
        dv2_set_focus(&mut cfg, 50.0);
        dv2_reset(&mut cfg);
        assert!((cfg.focus_distance_m - 2.0).abs() < 1e-5);
    }

    #[test]
    fn bokeh_area_positive() {
        let cfg = new_dof_v2();
        assert!(dv2_bokeh_area_mm2(&cfg, 5.0) >= 0.0);
    }

    #[test]
    fn json_has_focal() {
        let j = dv2_to_json(&new_dof_v2());
        assert!(j.contains("focal_mm"));
    }
}
