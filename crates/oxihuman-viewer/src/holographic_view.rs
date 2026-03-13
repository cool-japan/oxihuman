// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Holographic diffraction overlay stub.

/// Holographic color mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HoloColorMode {
    Rainbow,
    SingleWave,
    Iridescent,
}

/// Holographic view configuration.
#[derive(Debug, Clone)]
pub struct HolographicView {
    pub color_mode: HoloColorMode,
    pub diffraction_scale: f32,
    pub brightness: f32,
    pub rotation_speed: f32,
    pub enabled: bool,
}

impl HolographicView {
    pub fn new() -> Self {
        HolographicView {
            color_mode: HoloColorMode::Rainbow,
            diffraction_scale: 0.5,
            brightness: 1.0,
            rotation_speed: 0.0,
            enabled: true,
        }
    }
}

impl Default for HolographicView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new holographic view.
pub fn new_holographic_view() -> HolographicView {
    HolographicView::new()
}

/// Set color mode.
pub fn hgv_set_color_mode(view: &mut HolographicView, mode: HoloColorMode) {
    view.color_mode = mode;
}

/// Set diffraction scale.
pub fn hgv_set_diffraction_scale(view: &mut HolographicView, scale: f32) {
    view.diffraction_scale = scale.clamp(0.0, 2.0);
}

/// Set brightness.
pub fn hgv_set_brightness(view: &mut HolographicView, brightness: f32) {
    view.brightness = brightness.clamp(0.0, 4.0);
}

/// Set rotation speed (degrees per frame).
pub fn hgv_set_rotation_speed(view: &mut HolographicView, speed: f32) {
    view.rotation_speed = speed.clamp(-360.0, 360.0);
}

/// Enable or disable.
pub fn hgv_set_enabled(view: &mut HolographicView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn hgv_to_json(view: &HolographicView) -> String {
    let mode = match view.color_mode {
        HoloColorMode::Rainbow => "rainbow",
        HoloColorMode::SingleWave => "single_wave",
        HoloColorMode::Iridescent => "iridescent",
    };
    format!(
        r#"{{"color_mode":"{}","diffraction_scale":{},"brightness":{},"rotation_speed":{},"enabled":{}}}"#,
        mode, view.diffraction_scale, view.brightness, view.rotation_speed, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_color_mode() {
        let v = new_holographic_view();
        assert_eq!(
            v.color_mode,
            HoloColorMode::Rainbow /* default color_mode must be Rainbow */
        );
    }

    #[test]
    fn test_set_color_mode() {
        let mut v = new_holographic_view();
        hgv_set_color_mode(&mut v, HoloColorMode::Iridescent);
        assert_eq!(
            v.color_mode,
            HoloColorMode::Iridescent /* color_mode must be set */
        );
    }

    #[test]
    fn test_diffraction_scale_clamped() {
        let mut v = new_holographic_view();
        hgv_set_diffraction_scale(&mut v, 5.0);
        assert!((v.diffraction_scale - 2.0).abs() < 1e-6 /* diffraction_scale clamped to 2.0 */);
    }

    #[test]
    fn test_brightness_clamped() {
        let mut v = new_holographic_view();
        hgv_set_brightness(&mut v, -1.0);
        assert!((v.brightness).abs() < 1e-6 /* brightness clamped to 0.0 */);
    }

    #[test]
    fn test_rotation_speed_clamped() {
        let mut v = new_holographic_view();
        hgv_set_rotation_speed(&mut v, 1000.0);
        assert!((v.rotation_speed - 360.0).abs() < 1e-6 /* rotation_speed clamped to 360.0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_holographic_view();
        hgv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_color_mode() {
        let v = new_holographic_view();
        let j = hgv_to_json(&v);
        assert!(j.contains("\"color_mode\"") /* JSON must have color_mode */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_holographic_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_default_brightness() {
        let v = new_holographic_view();
        assert!((v.brightness - 1.0).abs() < 1e-6 /* default brightness must be 1.0 */);
    }

    #[test]
    fn test_rotation_speed_negative_clamped() {
        let mut v = new_holographic_view();
        hgv_set_rotation_speed(&mut v, -1000.0);
        assert!((v.rotation_speed + 360.0).abs() < 1e-6 /* rotation_speed clamped to -360.0 */);
    }
}
