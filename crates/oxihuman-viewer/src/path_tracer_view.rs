// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Path tracer sample visualization stub.

/// Path tracer view config.
#[derive(Debug, Clone)]
pub struct PathTracerViewConfig {
    pub samples_per_pixel: usize,
    pub max_depth: usize,
    pub exposure: f32,
    pub enabled: bool,
}

impl Default for PathTracerViewConfig {
    fn default() -> Self {
        PathTracerViewConfig {
            samples_per_pixel: 4,
            max_depth: 4,
            exposure: 1.0,
            enabled: true,
        }
    }
}

/// Create a new path tracer view config.
pub fn new_path_tracer_view() -> PathTracerViewConfig {
    PathTracerViewConfig::default()
}

/// Set samples per pixel.
pub fn ptv_set_spp(cfg: &mut PathTracerViewConfig, spp: usize) {
    cfg.samples_per_pixel = spp.max(1);
}

/// Set max depth.
pub fn ptv_set_depth(cfg: &mut PathTracerViewConfig, depth: usize) {
    cfg.max_depth = depth.max(1);
}

/// Set exposure.
pub fn ptv_set_exposure(cfg: &mut PathTracerViewConfig, exposure: f32) {
    cfg.exposure = exposure.max(0.0);
}

/// Enable or disable.
pub fn ptv_set_enabled(cfg: &mut PathTracerViewConfig, enabled: bool) {
    cfg.enabled = enabled;
}

/// Compute accumulated sample noise level (stub: decreases with spp).
pub fn ptv_noise_level(cfg: &PathTracerViewConfig) -> f32 {
    1.0 / (cfg.samples_per_pixel as f32).sqrt()
}

/// Return a JSON-like string.
pub fn ptv_to_json(cfg: &PathTracerViewConfig) -> String {
    format!(
        r#"{{"spp":{},"max_depth":{},"exposure":{:.4},"enabled":{}}}"#,
        cfg.samples_per_pixel, cfg.max_depth, cfg.exposure, cfg.enabled
    )
}

/// Return a debug color indicating the sample density.
pub fn ptv_sample_density_color(spp: usize) -> [f32; 3] {
    let v = (spp as f32 / 64.0).clamp(0.0, 1.0);
    [0.0, v, 1.0 - v]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_spp() {
        let c = new_path_tracer_view();
        assert_eq!(c.samples_per_pixel, 4 /* default spp is 4 */,);
    }

    #[test]
    fn test_set_spp() {
        let mut c = new_path_tracer_view();
        ptv_set_spp(&mut c, 16);
        assert_eq!(c.samples_per_pixel, 16 /* spp must match */,);
    }

    #[test]
    fn test_set_spp_minimum() {
        let mut c = new_path_tracer_view();
        ptv_set_spp(&mut c, 0);
        assert!(c.samples_per_pixel >= 1 /* spp must be at least 1 */,);
    }

    #[test]
    fn test_set_depth() {
        let mut c = new_path_tracer_view();
        ptv_set_depth(&mut c, 8);
        assert_eq!(c.max_depth, 8 /* depth must match */,);
    }

    #[test]
    fn test_set_exposure() {
        let mut c = new_path_tracer_view();
        ptv_set_exposure(&mut c, 2.0);
        assert!((c.exposure - 2.0).abs() < 1e-5, /* exposure must match */);
    }

    #[test]
    fn test_set_enabled_false() {
        let mut c = new_path_tracer_view();
        ptv_set_enabled(&mut c, false);
        assert!(!c.enabled /* should be disabled */,);
    }

    #[test]
    fn test_noise_level_decreases_with_spp() {
        let mut c = new_path_tracer_view();
        ptv_set_spp(&mut c, 1);
        let n1 = ptv_noise_level(&c);
        ptv_set_spp(&mut c, 16);
        let n16 = ptv_noise_level(&c);
        assert!(n1 > n16 /* noise should decrease with more samples */,);
    }

    #[test]
    fn test_to_json_contains_spp() {
        let c = new_path_tracer_view();
        let j = ptv_to_json(&c);
        assert!(j.contains("spp") /* JSON must contain spp */,);
    }

    #[test]
    fn test_sample_density_color_returns_three() {
        let color = ptv_sample_density_color(8);
        assert_eq!(color.len(), 3 /* color must be RGB */,);
    }

    #[test]
    fn test_sample_density_color_clamps() {
        let color = ptv_sample_density_color(1000);
        assert!(color[1] <= 1.0 /* green component clamped to 1 */,);
    }
}
