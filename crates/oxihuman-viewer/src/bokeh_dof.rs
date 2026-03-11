// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Bokeh depth-of-field post-processing effect.

use std::f32::consts::PI;

/// Configuration for bokeh dof.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BokehDofConfig {
    pub focal_distance: f32,
    pub aperture: f32,
    pub blade_count: f32,
    pub rotation: f32,
    pub intensity: f32,
}

#[allow(dead_code)]
pub fn default_bokeh_dof() -> BokehDofConfig {
    BokehDofConfig { focal_distance: 5.0, aperture: 2.8, blade_count: 6.0, rotation: 0.0, intensity: 0.5 }
}

#[allow(dead_code)]
pub fn set_bokeh_dof_focal_distance(cfg: &mut BokehDofConfig, value: f32) {
    cfg.focal_distance = value.clamp(0.1_f32, 1000.0_f32);
}

#[allow(dead_code)]
pub fn set_bokeh_dof_aperture(cfg: &mut BokehDofConfig, value: f32) {
    cfg.aperture = value.clamp(0.5_f32, 64.0_f32);
}

#[allow(dead_code)]
pub fn set_bokeh_dof_blade_count(cfg: &mut BokehDofConfig, value: f32) {
    cfg.blade_count = value.clamp(1.0_f32, 128.0_f32);
}

#[allow(dead_code)]
pub fn set_bokeh_dof_rotation(cfg: &mut BokehDofConfig, value: f32) {
    cfg.rotation = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_bokeh_dof_intensity(cfg: &mut BokehDofConfig, value: f32) {
    cfg.intensity = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn bokeh_dof_weight(cfg: &BokehDofConfig) -> f32 {
    (cfg.focal_distance * (PI * 0.25).sin()).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn blend_bokeh_dof(a: &BokehDofConfig, b: &BokehDofConfig, t: f32) -> BokehDofConfig {
    let t = t.clamp(0.0, 1.0);
    BokehDofConfig {
        focal_distance: a.focal_distance + (b.focal_distance - a.focal_distance) * t,
        aperture: a.aperture + (b.aperture - a.aperture) * t,
        blade_count: a.blade_count + (b.blade_count - a.blade_count) * t,
        rotation: a.rotation + (b.rotation - a.rotation) * t,
        intensity: a.intensity + (b.intensity - a.intensity) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cfg = default_bokeh_dof();
        assert!((cfg.focal_distance - 5.0_f32).abs() < 1e-3);
    }

    #[test]
    fn test_set_focal_distance() {
        let mut cfg = default_bokeh_dof();
        set_bokeh_dof_focal_distance(&mut cfg, 0.7);
        assert!((cfg.focal_distance - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_aperture() {
        let mut cfg = default_bokeh_dof();
        set_bokeh_dof_aperture(&mut cfg, 0.8);
        assert!((cfg.aperture - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_blade_count() {
        let mut cfg = default_bokeh_dof();
        set_bokeh_dof_blade_count(&mut cfg, 0.6);
        assert!((cfg.blade_count - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_rotation() {
        let mut cfg = default_bokeh_dof();
        set_bokeh_dof_rotation(&mut cfg, 0.5);
        assert!((cfg.rotation - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_intensity() {
        let mut cfg = default_bokeh_dof();
        set_bokeh_dof_intensity(&mut cfg, 0.4);
        assert!((cfg.intensity - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_weight() {
        let cfg = default_bokeh_dof();
        let w = bokeh_dof_weight(&cfg);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend() {
        let a = default_bokeh_dof();
        let mut b = default_bokeh_dof();
        b.focal_distance = 1.0;
        let mid = blend_bokeh_dof(&a, &b, 0.5);
        assert!((mid.focal_distance - 3.0_f32).abs() < 1e-3);
    }

    #[test]
    fn test_blend_zero() {
        let a = default_bokeh_dof();
        let b = default_bokeh_dof();
        let r = blend_bokeh_dof(&a, &b, 0.0);
        assert!((r.focal_distance - a.focal_distance).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = default_bokeh_dof();
        let b = default_bokeh_dof();
        let r = blend_bokeh_dof(&a, &b, 1.0);
        assert!((r.focal_distance - b.focal_distance).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamp() {
        let a = default_bokeh_dof();
        let b = default_bokeh_dof();
        let r = blend_bokeh_dof(&a, &b, 2.0);
        assert!((r.focal_distance - b.focal_distance).abs() < 1e-6);
    }
}
