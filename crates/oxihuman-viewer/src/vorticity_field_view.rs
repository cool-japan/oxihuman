// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Vorticity field visualization — shows curl of velocity (rotational component).

/// Vorticity field view configuration.
#[derive(Debug, Clone)]
pub struct VorticityFieldView {
    pub enabled: bool,
    pub vorticity_scale: f32,
    pub show_direction: bool,
    pub magnitude_threshold: f32,
    pub opacity: f32,
}

impl VorticityFieldView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            vorticity_scale: 0.1,
            show_direction: true,
            magnitude_threshold: 0.01,
            opacity: 0.8,
        }
    }
}

impl Default for VorticityFieldView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new vorticity field view.
pub fn new_vorticity_field_view() -> VorticityFieldView {
    VorticityFieldView::new()
}

/// Enable or disable vorticity display.
pub fn vvf_set_enabled(v: &mut VorticityFieldView, enabled: bool) {
    v.enabled = enabled;
}

/// Set vorticity arrow scale.
pub fn vvf_set_vorticity_scale(v: &mut VorticityFieldView, scale: f32) {
    v.vorticity_scale = scale.clamp(0.001, 10.0);
}

/// Toggle direction arrow display.
pub fn vvf_set_show_direction(v: &mut VorticityFieldView, show: bool) {
    v.show_direction = show;
}

/// Set magnitude threshold below which vorticity is hidden.
pub fn vvf_set_magnitude_threshold(v: &mut VorticityFieldView, thresh: f32) {
    v.magnitude_threshold = thresh.max(0.0);
}

/// Set overlay opacity.
pub fn vvf_set_opacity(v: &mut VorticityFieldView, o: f32) {
    v.opacity = o.clamp(0.0, 1.0);
}

/// Check if a vorticity magnitude should be displayed.
pub fn vvf_is_visible(v: &VorticityFieldView, magnitude: f32) -> bool {
    magnitude >= v.magnitude_threshold
}

/// Serialize to JSON-like string.
pub fn vorticity_field_view_to_json(v: &VorticityFieldView) -> String {
    format!(
        r#"{{"enabled":{},"vorticity_scale":{:.4},"show_direction":{},"magnitude_threshold":{:.6},"opacity":{:.4}}}"#,
        v.enabled, v.vorticity_scale, v.show_direction, v.magnitude_threshold, v.opacity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_vorticity_field_view();
        assert!(!v.enabled);
        assert!(v.show_direction);
    }

    #[test]
    fn test_enable() {
        let mut v = new_vorticity_field_view();
        vvf_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_scale_clamp_low() {
        let mut v = new_vorticity_field_view();
        vvf_set_vorticity_scale(&mut v, 0.0);
        assert_eq!(v.vorticity_scale, 0.001);
    }

    #[test]
    fn test_scale_set() {
        let mut v = new_vorticity_field_view();
        vvf_set_vorticity_scale(&mut v, 0.5);
        assert!((v.vorticity_scale - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_direction_toggle() {
        let mut v = new_vorticity_field_view();
        vvf_set_show_direction(&mut v, false);
        assert!(!v.show_direction);
    }

    #[test]
    fn test_threshold_set() {
        let mut v = new_vorticity_field_view();
        vvf_set_magnitude_threshold(&mut v, 0.05);
        assert!((v.magnitude_threshold - 0.05).abs() < 1e-8);
    }

    #[test]
    fn test_visibility_above_threshold() {
        let v = new_vorticity_field_view();
        assert!(vvf_is_visible(&v, 1.0)); /* above default threshold */
    }

    #[test]
    fn test_visibility_below_threshold() {
        let v = new_vorticity_field_view();
        assert!(!vvf_is_visible(&v, 0.0)); /* below threshold */
    }

    #[test]
    fn test_json_keys() {
        let v = new_vorticity_field_view();
        let s = vorticity_field_view_to_json(&v);
        assert!(s.contains("magnitude_threshold"));
    }

    #[test]
    fn test_clone() {
        let v = new_vorticity_field_view();
        let v2 = v.clone();
        assert!((v2.opacity - v.opacity).abs() < 1e-6);
    }
}
