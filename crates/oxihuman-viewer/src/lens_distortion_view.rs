// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Lens distortion mesh debug view stub.

/// Lens distortion view config.
#[derive(Debug, Clone)]
pub struct LensDistortionViewConfig {
    pub grid_divisions: usize,
    pub k1: f32,
    pub k2: f32,
    pub enabled: bool,
    pub show_grid: bool,
}

impl Default for LensDistortionViewConfig {
    fn default() -> Self {
        LensDistortionViewConfig {
            grid_divisions: 16,
            k1: 0.0,
            k2: 0.0,
            enabled: true,
            show_grid: true,
        }
    }
}

/// Create a new lens distortion view config.
pub fn new_lens_distortion_view() -> LensDistortionViewConfig {
    LensDistortionViewConfig::default()
}

/// Set k1 coefficient.
pub fn ldv_set_k1(cfg: &mut LensDistortionViewConfig, k1: f32) {
    cfg.k1 = k1;
}

/// Set k2 coefficient.
pub fn ldv_set_k2(cfg: &mut LensDistortionViewConfig, k2: f32) {
    cfg.k2 = k2;
}

/// Set grid divisions.
pub fn ldv_set_divisions(cfg: &mut LensDistortionViewConfig, divs: usize) {
    cfg.grid_divisions = divs.max(2);
}

/// Enable or disable.
pub fn ldv_set_enabled(cfg: &mut LensDistortionViewConfig, enabled: bool) {
    cfg.enabled = enabled;
}

/// Toggle grid display.
pub fn ldv_toggle_grid(cfg: &mut LensDistortionViewConfig) {
    cfg.show_grid = !cfg.show_grid;
}

/// Apply barrel/pincushion distortion to a UV coordinate.
pub fn ldv_distort_uv(cfg: &LensDistortionViewConfig, uv: [f32; 2]) -> [f32; 2] {
    let cx = uv[0] - 0.5;
    let cy = uv[1] - 0.5;
    let r2 = cx * cx + cy * cy;
    let factor = 1.0 + cfg.k1 * r2 + cfg.k2 * r2 * r2;
    [0.5 + cx * factor, 0.5 + cy * factor]
}

/// Return a JSON-like string.
pub fn ldv_to_json(cfg: &LensDistortionViewConfig) -> String {
    format!(
        r#"{{"k1":{:.4},"k2":{:.4},"divisions":{},"enabled":{}}}"#,
        cfg.k1, cfg.k2, cfg.grid_divisions, cfg.enabled
    )
}

/// Return number of grid vertices.
pub fn ldv_grid_vertex_count(cfg: &LensDistortionViewConfig) -> usize {
    (cfg.grid_divisions + 1) * (cfg.grid_divisions + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_k1_zero() {
        let c = new_lens_distortion_view();
        assert!((c.k1).abs() < 1e-6 /* default k1 is 0 */,);
    }

    #[test]
    fn test_set_k1() {
        let mut c = new_lens_distortion_view();
        ldv_set_k1(&mut c, 0.2);
        assert!((c.k1 - 0.2).abs() < 1e-5 /* k1 must match */,);
    }

    #[test]
    fn test_set_k2() {
        let mut c = new_lens_distortion_view();
        ldv_set_k2(&mut c, 0.05);
        assert!((c.k2 - 0.05).abs() < 1e-5 /* k2 must match */,);
    }

    #[test]
    fn test_set_divisions_minimum() {
        let mut c = new_lens_distortion_view();
        ldv_set_divisions(&mut c, 1);
        assert!(c.grid_divisions >= 2, /* divisions must be at least 2 */);
    }

    #[test]
    fn test_set_enabled_false() {
        let mut c = new_lens_distortion_view();
        ldv_set_enabled(&mut c, false);
        assert!(!c.enabled /* should be disabled */,);
    }

    #[test]
    fn test_toggle_grid() {
        let mut c = new_lens_distortion_view();
        ldv_toggle_grid(&mut c);
        assert!(!c.show_grid /* grid toggled off */,);
    }

    #[test]
    fn test_distort_uv_center_unchanged() {
        let c = new_lens_distortion_view();
        let uv = ldv_distort_uv(&c, [0.5, 0.5]);
        assert!((uv[0] - 0.5).abs() < 1e-5, /* center should remain unchanged */);
    }

    #[test]
    fn test_distort_uv_with_k1_barrel() {
        let mut c = new_lens_distortion_view();
        ldv_set_k1(&mut c, 0.5);
        let uv = ldv_distort_uv(&c, [1.0, 0.5]);
        assert_ne!(uv[0], 1.0 /* barrel distortion should move corner */,);
    }

    #[test]
    fn test_grid_vertex_count() {
        let c = new_lens_distortion_view();
        let n = ldv_grid_vertex_count(&c);
        assert_eq!(n, 17 * 17 /* 16 divisions gives 17x17 vertices */,);
    }

    #[test]
    fn test_to_json_contains_k1() {
        let c = new_lens_distortion_view();
        let j = ldv_to_json(&c);
        assert!(j.contains("k1") /* JSON must contain k1 */,);
    }
}
