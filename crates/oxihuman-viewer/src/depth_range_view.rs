// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Depth buffer range visualization debug view.

/// Configuration for depth range view.
#[derive(Debug, Clone)]
pub struct DepthRangeViewConfig {
    pub near: f32,
    pub far: f32,
    pub gamma: f32,
    pub invert: bool,
}

impl Default for DepthRangeViewConfig {
    fn default() -> Self {
        Self { near: 0.1, far: 1000.0, gamma: 1.0, invert: false }
    }
}

/// State for depth range visualization.
#[derive(Debug, Clone)]
pub struct DepthRangeView {
    pub config: DepthRangeViewConfig,
    pub enabled: bool,
}

impl Default for DepthRangeView {
    fn default() -> Self {
        Self { config: DepthRangeViewConfig::default(), enabled: false }
    }
}

/// Enable depth range visualization.
pub fn drv_enable(view: &mut DepthRangeView) {
    view.enabled = true;
}

/// Disable depth range visualization.
pub fn drv_disable(view: &mut DepthRangeView) {
    view.enabled = false;
}

/// Set the near and far clip distances.
pub fn drv_set_range(view: &mut DepthRangeView, near: f32, far: f32) {
    view.config.near = near.max(0.0001);
    view.config.far = far.max(view.config.near + 0.001);
}

/// Convert a non-linear depth value to a linear [0, 1] visualization value.
pub fn drv_linearize(depth: f32, config: &DepthRangeViewConfig) -> f32 {
    let n = config.near;
    let f = config.far;
    let linear = (2.0 * n) / (f + n - depth * (f - n));
    let v = linear.clamp(0.0, 1.0).powf(1.0 / config.gamma.max(0.001));
    if config.invert { 1.0 - v } else { v }
}

/// Return the linearized depth as a greyscale RGBA color.
pub fn drv_depth_to_color(depth: f32, config: &DepthRangeViewConfig) -> [f32; 4] {
    let v = drv_linearize(depth, config);
    [v, v, v, 1.0]
}

/// Export config to JSON string (stub).
pub fn drv_to_json(view: &DepthRangeView) -> String {
    format!(
        r#"{{"near":{:.4},"far":{:.2},"enabled":{}}}"#,
        view.config.near, view.config.far, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled() {
        /* default should be disabled */
        let v = DepthRangeView::default();
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable_disable() {
        /* enable/disable should toggle */
        let mut v = DepthRangeView::default();
        drv_enable(&mut v);
        assert!(v.enabled);
        drv_disable(&mut v);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_range() {
        /* near and far should be set */
        let mut v = DepthRangeView::default();
        drv_set_range(&mut v, 0.5, 500.0);
        assert!((v.config.near - 0.5).abs() < 1e-6);
        assert!((v.config.far - 500.0).abs() < 1e-5);
    }

    #[test]
    fn test_near_minimum() {
        /* near should have a positive minimum */
        let mut v = DepthRangeView::default();
        drv_set_range(&mut v, 0.0, 100.0);
        assert!(v.config.near > 0.0);
    }

    #[test]
    fn test_linearize_range() {
        /* linearized depth should be in [0, 1] */
        let cfg = DepthRangeViewConfig::default();
        let v = drv_linearize(0.5, &cfg);
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn test_linearize_invert() {
        /* inverted output should be 1 - normal output */
        let cfg = DepthRangeViewConfig::default();
        let normal = drv_linearize(0.5, &cfg);
        let inv_cfg = DepthRangeViewConfig { invert: true, ..cfg };
        let inverted = drv_linearize(0.5, &inv_cfg);
        assert!((normal + inverted - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_depth_to_color_greyscale() {
        /* RGB channels should all be equal for greyscale */
        let cfg = DepthRangeViewConfig::default();
        let c = drv_depth_to_color(0.5, &cfg);
        assert!((c[0] - c[1]).abs() < 1e-6);
        assert!((c[1] - c[2]).abs() < 1e-6);
    }

    #[test]
    fn test_depth_to_color_alpha() {
        /* alpha should always be 1.0 */
        let cfg = DepthRangeViewConfig::default();
        let c = drv_depth_to_color(0.5, &cfg);
        assert!((c[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json_contains_near() {
        /* JSON should contain near value */
        let v = DepthRangeView::default();
        let json = drv_to_json(&v);
        assert!(json.contains("near"));
    }
}
