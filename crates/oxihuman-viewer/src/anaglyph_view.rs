// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Red-cyan anaglyph 3D view stub.

/// Anaglyph color method.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnaglyphMethod {
    TrueAnaglyph,
    GrayAnaglyph,
    ColorAnaglyph,
    HalfColor,
    Optimized,
}

/// Anaglyph view configuration.
#[derive(Debug, Clone)]
pub struct AnaglyphView {
    pub method: AnaglyphMethod,
    pub eye_separation: f32,
    pub convergence: f32,
    pub enabled: bool,
}

impl AnaglyphView {
    pub fn new() -> Self {
        AnaglyphView {
            method: AnaglyphMethod::ColorAnaglyph,
            eye_separation: 0.065,
            convergence: 1.0,
            enabled: true,
        }
    }
}

impl Default for AnaglyphView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new anaglyph view.
pub fn new_anaglyph_view() -> AnaglyphView {
    AnaglyphView::new()
}

/// Set anaglyph method.
pub fn agv_set_method(view: &mut AnaglyphView, method: AnaglyphMethod) {
    view.method = method;
}

/// Set eye separation distance.
pub fn agv_set_eye_separation(view: &mut AnaglyphView, separation: f32) {
    view.eye_separation = separation.clamp(0.01, 0.3);
}

/// Set convergence plane distance.
pub fn agv_set_convergence(view: &mut AnaglyphView, convergence: f32) {
    view.convergence = convergence.clamp(0.1, 100.0);
}

/// Enable or disable.
pub fn agv_set_enabled(view: &mut AnaglyphView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn agv_to_json(view: &AnaglyphView) -> String {
    let method = match view.method {
        AnaglyphMethod::TrueAnaglyph => "true_anaglyph",
        AnaglyphMethod::GrayAnaglyph => "gray_anaglyph",
        AnaglyphMethod::ColorAnaglyph => "color_anaglyph",
        AnaglyphMethod::HalfColor => "half_color",
        AnaglyphMethod::Optimized => "optimized",
    };
    format!(
        r#"{{"method":"{}","eye_separation":{},"convergence":{},"enabled":{}}}"#,
        method, view.eye_separation, view.convergence, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_method() {
        let v = new_anaglyph_view();
        assert_eq!(
            v.method,
            AnaglyphMethod::ColorAnaglyph /* default method must be ColorAnaglyph */
        );
    }

    #[test]
    fn test_set_method() {
        let mut v = new_anaglyph_view();
        agv_set_method(&mut v, AnaglyphMethod::Optimized);
        assert_eq!(
            v.method,
            AnaglyphMethod::Optimized /* method must be set */
        );
    }

    #[test]
    fn test_eye_separation_clamped_min() {
        let mut v = new_anaglyph_view();
        agv_set_eye_separation(&mut v, 0.0);
        assert!((v.eye_separation - 0.01).abs() < 1e-6 /* eye_separation clamped to 0.01 */);
    }

    #[test]
    fn test_eye_separation_clamped_max() {
        let mut v = new_anaglyph_view();
        agv_set_eye_separation(&mut v, 1.0);
        assert!((v.eye_separation - 0.3).abs() < 1e-6 /* eye_separation clamped to 0.3 */);
    }

    #[test]
    fn test_convergence_clamped() {
        let mut v = new_anaglyph_view();
        agv_set_convergence(&mut v, 0.0);
        assert!((v.convergence - 0.1).abs() < 1e-6 /* convergence clamped to 0.1 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_anaglyph_view();
        agv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_method() {
        let v = new_anaglyph_view();
        let j = agv_to_json(&v);
        assert!(j.contains("\"method\"") /* JSON must have method */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_anaglyph_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_default_eye_separation() {
        let v = new_anaglyph_view();
        assert!((v.eye_separation - 0.065).abs() < 1e-5 /* default eye_separation must be 0.065 */);
    }

    #[test]
    fn test_default_convergence() {
        let v = new_anaglyph_view();
        assert!((v.convergence - 1.0).abs() < 1e-6 /* default convergence must be 1.0 */);
    }
}
