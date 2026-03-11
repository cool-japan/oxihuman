// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Angle distortion heat-map (deviation from ideal right-angle quad corner).
#[derive(Debug, Clone)]
pub struct AngleDistortionView {
    pub enabled: bool,
    /// Maximum distortion angle in degrees shown as hot colour (clamp point).
    pub max_deg: f32,
    /// Opacity (0.0 … 1.0).
    pub opacity: f32,
}

pub fn new_angle_distortion_view() -> AngleDistortionView {
    AngleDistortionView {
        enabled: false,
        max_deg: 45.0,
        opacity: 0.8,
    }
}

pub fn adv_enable(v: &mut AngleDistortionView) {
    v.enabled = true;
}

pub fn adv_set_max_deg(v: &mut AngleDistortionView, d: f32) {
    v.max_deg = d.clamp(1.0, 90.0);
}

pub fn adv_set_opacity(v: &mut AngleDistortionView, o: f32) {
    v.opacity = o.clamp(0.0, 1.0);
}

/// Maps a corner angle (degrees) to a heat-map colour.
/// 90° → green (no distortion), deviation → red.
pub fn adv_angle_color(v: &AngleDistortionView, angle_deg: f32) -> [f32; 3] {
    let dev = (angle_deg - 90.0).abs();
    let t = (dev / v.max_deg).clamp(0.0, 1.0);
    [t, 1.0 - t, 0.0]
}

pub fn adv_distortion_score(angle_deg: f32) -> f32 {
    (angle_deg - 90.0).abs() / 90.0
}

pub fn adv_to_json(v: &AngleDistortionView) -> String {
    format!(
        r#"{{"enabled":{},"max_deg":{:.4},"opacity":{:.4}}}"#,
        v.enabled, v.max_deg, v.opacity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* max_deg=45, opacity=0.8 */
        let v = new_angle_distortion_view();
        assert!((v.max_deg - 45.0).abs() < 1e-6);
        assert!((v.opacity - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_enable() {
        /* enable */
        let mut v = new_angle_distortion_view();
        adv_enable(&mut v);
        assert!(v.enabled);
    }

    #[test]
    fn test_color_90_green() {
        /* 90 degrees = no distortion = green dominant */
        let v = new_angle_distortion_view();
        let c = adv_angle_color(&v, 90.0);
        assert_eq!(c[0], 0.0);
        assert_eq!(c[1], 1.0);
    }

    #[test]
    fn test_color_hot() {
        /* large deviation -> red dominant */
        let v = new_angle_distortion_view();
        let c = adv_angle_color(&v, 135.0);
        assert!(c[0] > c[1]);
    }

    #[test]
    fn test_set_max_deg() {
        /* valid value */
        let mut v = new_angle_distortion_view();
        adv_set_max_deg(&mut v, 30.0);
        assert!((v.max_deg - 30.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_deg_clamp() {
        /* clamped at 90 */
        let mut v = new_angle_distortion_view();
        adv_set_max_deg(&mut v, 180.0);
        assert_eq!(v.max_deg, 90.0);
    }

    #[test]
    fn test_distortion_score_zero() {
        /* perfect right angle */
        assert_eq!(adv_distortion_score(90.0), 0.0);
    }

    #[test]
    fn test_distortion_score_nonzero() {
        /* non-perfect angle */
        assert!(adv_distortion_score(45.0) > 0.0);
    }

    #[test]
    fn test_to_json() {
        /* JSON has max_deg */
        assert!(adv_to_json(&new_angle_distortion_view()).contains("max_deg"));
    }
}
