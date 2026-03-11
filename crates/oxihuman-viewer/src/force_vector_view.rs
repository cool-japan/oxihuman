// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Force/torque vector visualization — renders arrow glyphs for applied forces.

/// Force vector view configuration.
#[derive(Debug, Clone)]
pub struct ForceVectorView {
    pub enabled: bool,
    pub scale: f32,
    pub min_display_magnitude: f32,
    pub color_force: [f32; 4],
    pub color_torque: [f32; 4],
    pub show_torque: bool,
}

impl ForceVectorView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            scale: 0.01,
            min_display_magnitude: 0.1,
            color_force: [1.0, 0.5, 0.0, 1.0],
            color_torque: [0.5, 0.0, 1.0, 1.0],
            show_torque: true,
        }
    }
}

impl Default for ForceVectorView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new force vector view.
pub fn new_force_vector_view() -> ForceVectorView {
    ForceVectorView::new()
}

/// Enable or disable force vector display.
pub fn fvv_set_enabled(v: &mut ForceVectorView, enabled: bool) {
    v.enabled = enabled;
}

/// Set arrow scale factor.
pub fn fvv_set_scale(v: &mut ForceVectorView, scale: f32) {
    v.scale = scale.clamp(0.0001, 1.0);
}

/// Set minimum magnitude to display.
pub fn fvv_set_min_magnitude(v: &mut ForceVectorView, mag: f32) {
    v.min_display_magnitude = mag.max(0.0);
}

/// Toggle torque vector display.
pub fn fvv_set_show_torque(v: &mut ForceVectorView, show: bool) {
    v.show_torque = show;
}

/// Returns scaled display length for a given force magnitude.
pub fn fvv_display_length(v: &ForceVectorView, magnitude: f32) -> f32 {
    if magnitude < v.min_display_magnitude {
        return 0.0;
    }
    magnitude * v.scale
}

/// Serialize to JSON-like string.
pub fn force_vector_view_to_json(v: &ForceVectorView) -> String {
    format!(
        r#"{{"enabled":{},"scale":{:.6},"min_display_magnitude":{:.4},"show_torque":{}}}"#,
        v.enabled, v.scale, v.min_display_magnitude, v.show_torque
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_force_vector_view();
        assert!(!v.enabled);
        assert!(v.show_torque);
    }

    #[test]
    fn test_enable() {
        let mut v = new_force_vector_view();
        fvv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_scale_clamp() {
        let mut v = new_force_vector_view();
        fvv_set_scale(&mut v, 0.0);
        assert_eq!(v.scale, 0.0001);
    }

    #[test]
    fn test_min_magnitude() {
        let mut v = new_force_vector_view();
        fvv_set_min_magnitude(&mut v, 1.0);
        assert!((v.min_display_magnitude - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_display_length_below_min() {
        let v = new_force_vector_view();
        assert_eq!(fvv_display_length(&v, 0.0), 0.0);
    }

    #[test]
    fn test_display_length_above_min() {
        let v = new_force_vector_view();
        let len = fvv_display_length(&v, 100.0);
        assert!(len > 0.0);
    }

    #[test]
    fn test_toggle_torque() {
        let mut v = new_force_vector_view();
        fvv_set_show_torque(&mut v, false);
        assert!(!v.show_torque);
    }

    #[test]
    fn test_json_keys() {
        let v = new_force_vector_view();
        let s = force_vector_view_to_json(&v);
        assert!(s.contains("show_torque"));
    }

    #[test]
    fn test_clone() {
        let v = new_force_vector_view();
        let v2 = v.clone();
        assert_eq!(v2.show_torque, v.show_torque);
    }
}
