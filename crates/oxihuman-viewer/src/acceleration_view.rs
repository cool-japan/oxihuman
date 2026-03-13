// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Acceleration vector visualization — renders acceleration as scaled arrow glyphs.

/// Acceleration view configuration.
#[derive(Debug, Clone)]
pub struct AccelerationView {
    pub enabled: bool,
    pub scale: f32,
    pub color: [f32; 4],
    pub min_display_magnitude: f32,
}

impl AccelerationView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            scale: 0.005,
            color: [1.0, 0.8, 0.0, 1.0],
            min_display_magnitude: 0.01,
        }
    }
}

impl Default for AccelerationView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new acceleration view.
pub fn new_acceleration_view() -> AccelerationView {
    AccelerationView::new()
}

/// Enable or disable acceleration display.
pub fn acv_set_enabled(v: &mut AccelerationView, enabled: bool) {
    v.enabled = enabled;
}

/// Set arrow scale per unit acceleration.
pub fn acv_set_scale(v: &mut AccelerationView, scale: f32) {
    v.scale = scale.clamp(0.0001, 1.0);
}

/// Set display color.
pub fn acv_set_color(v: &mut AccelerationView, color: [f32; 4]) {
    v.color = color;
}

/// Set minimum magnitude threshold.
pub fn acv_set_min_magnitude(v: &mut AccelerationView, mag: f32) {
    v.min_display_magnitude = mag.max(0.0);
}

/// Compute display length for given acceleration magnitude.
pub fn acv_display_length(v: &AccelerationView, magnitude: f32) -> f32 {
    if magnitude < v.min_display_magnitude {
        return 0.0;
    }
    magnitude * v.scale
}

/// Serialize to JSON-like string.
pub fn acceleration_view_to_json(v: &AccelerationView) -> String {
    format!(
        r#"{{"enabled":{},"scale":{:.6},"min_display_magnitude":{:.4}}}"#,
        v.enabled, v.scale, v.min_display_magnitude
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_acceleration_view();
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable() {
        let mut v = new_acceleration_view();
        acv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_scale_clamp() {
        let mut v = new_acceleration_view();
        acv_set_scale(&mut v, 0.0);
        assert_eq!(v.scale, 0.0001);
    }

    #[test]
    fn test_display_length_below_min() {
        let v = new_acceleration_view();
        assert_eq!(acv_display_length(&v, 0.0), 0.0);
    }

    #[test]
    fn test_display_length_above_min() {
        let v = new_acceleration_view();
        let len = acv_display_length(&v, 10.0);
        assert!(len > 0.0);
    }

    #[test]
    fn test_color_set() {
        let mut v = new_acceleration_view();
        acv_set_color(&mut v, [0.5, 0.5, 0.5, 1.0]);
        assert!((v.color[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_min_magnitude_clamp() {
        let mut v = new_acceleration_view();
        acv_set_min_magnitude(&mut v, -5.0);
        assert_eq!(v.min_display_magnitude, 0.0);
    }

    #[test]
    fn test_json_keys() {
        let v = new_acceleration_view();
        let s = acceleration_view_to_json(&v);
        assert!(s.contains("min_display_magnitude"));
    }

    #[test]
    fn test_clone() {
        let v = new_acceleration_view();
        let v2 = v.clone();
        assert!((v2.scale - v.scale).abs() < 1e-6);
    }
}
