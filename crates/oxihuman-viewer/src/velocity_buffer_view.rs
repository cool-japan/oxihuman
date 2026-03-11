// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Velocity / motion buffer debug visualization.

/// Configuration for velocity buffer view.
#[derive(Debug, Clone)]
pub struct VelocityBufferViewConfig {
    pub scale: f32,
    pub show_magnitude: bool,
    pub clamp_max: f32,
}

impl Default for VelocityBufferViewConfig {
    fn default() -> Self {
        Self { scale: 1.0, show_magnitude: false, clamp_max: 10.0 }
    }
}

/// State for velocity buffer visualization.
#[derive(Debug, Clone)]
pub struct VelocityBufferView {
    pub config: VelocityBufferViewConfig,
    pub enabled: bool,
}

impl Default for VelocityBufferView {
    fn default() -> Self {
        Self { config: VelocityBufferViewConfig::default(), enabled: false }
    }
}

/// Enable the velocity buffer view.
pub fn vbv_enable(view: &mut VelocityBufferView) {
    view.enabled = true;
}

/// Disable the velocity buffer view.
pub fn vbv_disable(view: &mut VelocityBufferView) {
    view.enabled = false;
}

/// Set the velocity scale factor.
pub fn vbv_set_scale(view: &mut VelocityBufferView, scale: f32) {
    view.config.scale = scale.clamp(0.001, 100.0);
}

/// Map a 2D velocity vector to an RGBA color for display.
pub fn vbv_velocity_to_color(vx: f32, vy: f32, config: &VelocityBufferViewConfig) -> [f32; 4] {
    let sx = (vx * config.scale).clamp(-config.clamp_max, config.clamp_max) / config.clamp_max;
    let sy = (vy * config.scale).clamp(-config.clamp_max, config.clamp_max) / config.clamp_max;
    let r = sx * 0.5 + 0.5;
    let g = sy * 0.5 + 0.5;
    let mag = (sx * sx + sy * sy).sqrt().min(1.0);
    [r, g, if config.show_magnitude { mag } else { 0.5 }, 1.0]
}

/// Return the magnitude of a velocity vector.
pub fn vbv_magnitude(vx: f32, vy: f32) -> f32 {
    (vx * vx + vy * vy).sqrt()
}

/// Export config to JSON string (stub).
pub fn vbv_to_json(view: &VelocityBufferView) -> String {
    format!(
        r#"{{"scale":{:.3},"clamp_max":{:.2},"enabled":{}}}"#,
        view.config.scale, view.config.clamp_max, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled() {
        /* default should be disabled */
        let v = VelocityBufferView::default();
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable_disable() {
        /* enable/disable should toggle */
        let mut v = VelocityBufferView::default();
        vbv_enable(&mut v);
        assert!(v.enabled);
        vbv_disable(&mut v);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_scale() {
        /* scale should be stored */
        let mut v = VelocityBufferView::default();
        vbv_set_scale(&mut v, 2.5);
        assert!((v.config.scale - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_scale_min_clamp() {
        /* scale below minimum should be clamped */
        let mut v = VelocityBufferView::default();
        vbv_set_scale(&mut v, 0.0);
        assert!(v.config.scale >= 0.001);
    }

    #[test]
    fn test_velocity_to_color_neutral() {
        /* zero velocity should map to (0.5, 0.5, _, 1.0) */
        let cfg = VelocityBufferViewConfig::default();
        let c = vbv_velocity_to_color(0.0, 0.0, &cfg);
        assert!((c[0] - 0.5).abs() < 1e-6);
        assert!((c[1] - 0.5).abs() < 1e-6);
        assert!((c[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_magnitude_zero() {
        /* zero velocity should have zero magnitude */
        assert_eq!(vbv_magnitude(0.0, 0.0), 0.0);
    }

    #[test]
    fn test_magnitude_pythagorean() {
        /* 3-4-5 triangle should yield magnitude 5 */
        assert!((vbv_magnitude(3.0, 4.0) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_json_scale() {
        /* JSON should contain scale value */
        let v = VelocityBufferView::default();
        let json = vbv_to_json(&v);
        assert!(json.contains("scale"));
    }

    #[test]
    fn test_color_alpha_one() {
        /* alpha should always be 1.0 */
        let cfg = VelocityBufferViewConfig::default();
        let c = vbv_velocity_to_color(5.0, -3.0, &cfg);
        assert!((c[3] - 1.0).abs() < 1e-6);
    }
}
