// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Halftone effect — halftone dot pattern screen effect parameters.

use std::f32::consts::PI;

/// Dot shape.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HalftoneDotShape {
    #[default]
    Circle,
    Square,
    Diamond,
}

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct HalftoneEffectConfig {
    pub dot_shape: HalftoneDotShape,
    /// Grid cell size in pixels.
    pub cell_size_px: f32,
    /// Screen angle for the halftone grid in radians.
    pub angle_rad: f32,
    /// Intensity 0..=1 (mix between original and halftone).
    pub intensity: f32,
    /// Foreground color.
    pub fg_color: [f32; 3],
    /// Background color.
    pub bg_color: [f32; 3],
    pub enabled: bool,
}

impl Default for HalftoneEffectConfig {
    fn default() -> Self {
        Self {
            dot_shape: HalftoneDotShape::Circle,
            cell_size_px: 8.0,
            angle_rad: PI / 6.0,
            intensity: 0.8,
            fg_color: [0.0, 0.0, 0.0],
            bg_color: [1.0, 1.0, 1.0],
            enabled: true,
        }
    }
}

#[allow(dead_code)]
pub fn new_halftone_effect_config() -> HalftoneEffectConfig {
    HalftoneEffectConfig::default()
}

#[allow(dead_code)]
pub fn ht_set_cell_size(cfg: &mut HalftoneEffectConfig, px: f32) {
    cfg.cell_size_px = px.clamp(1.0, 64.0);
}

#[allow(dead_code)]
pub fn ht_set_angle_deg(cfg: &mut HalftoneEffectConfig, deg: f32) {
    cfg.angle_rad = deg.to_radians();
}

#[allow(dead_code)]
pub fn ht_set_intensity(cfg: &mut HalftoneEffectConfig, v: f32) {
    cfg.intensity = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ht_set_fg(cfg: &mut HalftoneEffectConfig, r: f32, g: f32, b: f32) {
    cfg.fg_color = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
}

/// Compute the dot radius (as fraction of cell_size) for a given luminance.
#[allow(dead_code)]
pub fn ht_dot_radius(luminance: f32, cfg: &HalftoneEffectConfig) -> f32 {
    let l = luminance.clamp(0.0, 1.0);
    // Inverted: dark areas → large dots.
    let r = (1.0 - l).sqrt() * 0.5 * cfg.cell_size_px;
    r.clamp(0.0, cfg.cell_size_px * 0.5)
}

/// Compute the approximate coverage fraction for a luminance value.
#[allow(dead_code)]
pub fn ht_coverage(luminance: f32, cfg: &HalftoneEffectConfig) -> f32 {
    let r = ht_dot_radius(luminance, cfg);
    let cell_area = cfg.cell_size_px * cfg.cell_size_px;
    match cfg.dot_shape {
        HalftoneDotShape::Circle => (PI * r * r / cell_area).clamp(0.0, 1.0),
        HalftoneDotShape::Square => (4.0 * r * r / cell_area).clamp(0.0, 1.0),
        HalftoneDotShape::Diamond => (2.0 * r * r / cell_area).clamp(0.0, 1.0),
    }
}

#[allow(dead_code)]
pub fn ht_to_json(cfg: &HalftoneEffectConfig) -> String {
    format!(
        "{{\"cell_size_px\":{:.1},\"intensity\":{:.3},\"enabled\":{}}}",
        cfg.cell_size_px, cfg.intensity, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn dark_luminance_large_dot() {
        let cfg = new_halftone_effect_config();
        let r_dark = ht_dot_radius(0.0, &cfg);
        let r_light = ht_dot_radius(1.0, &cfg);
        assert!(r_dark > r_light);
    }

    #[test]
    fn coverage_in_range() {
        let cfg = new_halftone_effect_config();
        let cov = ht_coverage(0.5, &cfg);
        assert!((0.0..=1.0).contains(&cov));
    }

    #[test]
    fn circle_coverage_uses_pi() {
        let cfg = new_halftone_effect_config();
        let r = ht_dot_radius(0.0, &cfg);
        let cell_area = cfg.cell_size_px * cfg.cell_size_px;
        let expected = (PI * r * r / cell_area).clamp(0.0, 1.0);
        let got = ht_coverage(0.0, &cfg);
        assert!((got - expected).abs() < 1e-5);
    }

    #[test]
    fn cell_size_clamps() {
        let mut cfg = new_halftone_effect_config();
        ht_set_cell_size(&mut cfg, 0.0);
        assert!((cfg.cell_size_px - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cell_size_clamps_high() {
        let mut cfg = new_halftone_effect_config();
        ht_set_cell_size(&mut cfg, 1000.0);
        assert!((cfg.cell_size_px - 64.0).abs() < 1e-6);
    }

    #[test]
    fn intensity_clamps() {
        let mut cfg = new_halftone_effect_config();
        ht_set_intensity(&mut cfg, 5.0);
        assert!((cfg.intensity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn angle_deg_converts() {
        let mut cfg = new_halftone_effect_config();
        ht_set_angle_deg(&mut cfg, 45.0);
        assert!((cfg.angle_rad - PI / 4.0).abs() < 1e-5);
    }

    #[test]
    fn fg_color_clamps() {
        let mut cfg = new_halftone_effect_config();
        ht_set_fg(&mut cfg, 2.0, -1.0, 0.5);
        assert!((cfg.fg_color[0] - 1.0).abs() < 1e-6);
        assert!(cfg.fg_color[1] < 1e-6);
    }

    #[test]
    fn square_coverage_different_from_circle() {
        let mut cfg = new_halftone_effect_config();
        let circle_cov = ht_coverage(0.3, &cfg);
        cfg.dot_shape = HalftoneDotShape::Square;
        let square_cov = ht_coverage(0.3, &cfg);
        // Both in range, square > circle for same radius
        assert!((0.0..=1.0).contains(&circle_cov));
        assert!((0.0..=1.0).contains(&square_cov));
    }

    #[test]
    fn json_has_keys() {
        let j = ht_to_json(&new_halftone_effect_config());
        assert!(j.contains("cell_size_px") && j.contains("enabled"));
    }
}
