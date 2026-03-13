// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Velocity arrow visualization — renders velocity vectors as coloured arrows.

/// Velocity arrow view configuration.
#[derive(Debug, Clone)]
pub struct VelocityArrowView {
    pub enabled: bool,
    pub scale: f32,
    pub max_speed_clamp: f32,
    pub color_low: [f32; 4],
    pub color_high: [f32; 4],
}

impl VelocityArrowView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            scale: 0.05,
            max_speed_clamp: 10.0,
            color_low: [0.0, 0.5, 1.0, 1.0],
            color_high: [1.0, 0.0, 0.0, 1.0],
        }
    }
}

impl Default for VelocityArrowView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new velocity arrow view.
pub fn new_velocity_arrow_view() -> VelocityArrowView {
    VelocityArrowView::new()
}

/// Enable or disable the view.
pub fn vav_set_enabled(v: &mut VelocityArrowView, enabled: bool) {
    v.enabled = enabled;
}

/// Set arrow scale per unit speed.
pub fn vav_set_scale(v: &mut VelocityArrowView, scale: f32) {
    v.scale = scale.clamp(0.0001, 1.0);
}

/// Set maximum speed for colour ramp clamping.
pub fn vav_set_max_speed(v: &mut VelocityArrowView, max: f32) {
    v.max_speed_clamp = max.max(0.001);
}

/// Compute display arrow length for a given speed.
pub fn vav_arrow_length(v: &VelocityArrowView, speed: f32) -> f32 {
    speed.clamp(0.0, v.max_speed_clamp) * v.scale
}

/// Interpolate colour between low and high based on normalised speed.
pub fn vav_color_at_speed(v: &VelocityArrowView, speed: f32) -> [f32; 4] {
    let t = (speed / v.max_speed_clamp).clamp(0.0, 1.0);
    [
        v.color_low[0] + t * (v.color_high[0] - v.color_low[0]),
        v.color_low[1] + t * (v.color_high[1] - v.color_low[1]),
        v.color_low[2] + t * (v.color_high[2] - v.color_low[2]),
        1.0,
    ]
}

/// Serialize to JSON-like string.
pub fn velocity_arrow_view_to_json(v: &VelocityArrowView) -> String {
    format!(
        r#"{{"enabled":{},"scale":{:.6},"max_speed_clamp":{:.4}}}"#,
        v.enabled, v.scale, v.max_speed_clamp
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_velocity_arrow_view();
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable() {
        let mut v = new_velocity_arrow_view();
        vav_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_scale_clamp() {
        let mut v = new_velocity_arrow_view();
        vav_set_scale(&mut v, 0.0);
        assert_eq!(v.scale, 0.0001);
    }

    #[test]
    fn test_arrow_length_zero() {
        let v = new_velocity_arrow_view();
        assert_eq!(vav_arrow_length(&v, 0.0), 0.0);
    }

    #[test]
    fn test_arrow_length_clamped() {
        let v = new_velocity_arrow_view();
        let l1 = vav_arrow_length(&v, v.max_speed_clamp);
        let l2 = vav_arrow_length(&v, v.max_speed_clamp * 2.0);
        assert!((l1 - l2).abs() < 1e-6); /* clamped */
    }

    #[test]
    fn test_color_at_zero() {
        let v = new_velocity_arrow_view();
        let c = vav_color_at_speed(&v, 0.0);
        assert!((c[0] - v.color_low[0]).abs() < 1e-6);
    }

    #[test]
    fn test_color_at_max() {
        let v = new_velocity_arrow_view();
        let c = vav_color_at_speed(&v, v.max_speed_clamp);
        assert!((c[0] - v.color_high[0]).abs() < 1e-5);
    }

    #[test]
    fn test_json_keys() {
        let v = new_velocity_arrow_view();
        let s = velocity_arrow_view_to_json(&v);
        assert!(s.contains("max_speed_clamp"));
    }

    #[test]
    fn test_clone() {
        let v = new_velocity_arrow_view();
        let v2 = v.clone();
        assert!((v2.scale - v.scale).abs() < 1e-6);
    }
}
