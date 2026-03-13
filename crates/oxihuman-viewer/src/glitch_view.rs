// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Glitch artifact effect view stub.

/// Glitch artifact type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GlitchType {
    RgbShift,
    ScanlineShift,
    BlockCorrupt,
    Combined,
}

/// Glitch view configuration.
#[derive(Debug, Clone)]
pub struct GlitchView {
    pub glitch_type: GlitchType,
    pub intensity: f32,
    pub frequency: f32,
    pub seed: u64,
    pub enabled: bool,
}

impl GlitchView {
    pub fn new() -> Self {
        GlitchView {
            glitch_type: GlitchType::RgbShift,
            intensity: 0.3,
            frequency: 0.1,
            seed: 42,
            enabled: true,
        }
    }
}

impl Default for GlitchView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new glitch view.
pub fn new_glitch_view() -> GlitchView {
    GlitchView::new()
}

/// Set glitch type.
pub fn glv_set_type(view: &mut GlitchView, glitch_type: GlitchType) {
    view.glitch_type = glitch_type;
}

/// Set glitch intensity.
pub fn glv_set_intensity(view: &mut GlitchView, intensity: f32) {
    view.intensity = intensity.clamp(0.0, 1.0);
}

/// Set glitch trigger frequency (0=never, 1=every frame).
pub fn glv_set_frequency(view: &mut GlitchView, frequency: f32) {
    view.frequency = frequency.clamp(0.0, 1.0);
}

/// Set random seed.
pub fn glv_set_seed(view: &mut GlitchView, seed: u64) {
    view.seed = seed;
}

/// Enable or disable.
pub fn glv_set_enabled(view: &mut GlitchView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn glv_to_json(view: &GlitchView) -> String {
    let gtype = match view.glitch_type {
        GlitchType::RgbShift => "rgb_shift",
        GlitchType::ScanlineShift => "scanline_shift",
        GlitchType::BlockCorrupt => "block_corrupt",
        GlitchType::Combined => "combined",
    };
    format!(
        r#"{{"glitch_type":"{}","intensity":{},"frequency":{},"seed":{},"enabled":{}}}"#,
        gtype, view.intensity, view.frequency, view.seed, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_type() {
        let v = new_glitch_view();
        assert_eq!(
            v.glitch_type,
            GlitchType::RgbShift /* default type must be RgbShift */
        );
    }

    #[test]
    fn test_set_type() {
        let mut v = new_glitch_view();
        glv_set_type(&mut v, GlitchType::Combined);
        assert_eq!(
            v.glitch_type,
            GlitchType::Combined /* type must be set */
        );
    }

    #[test]
    fn test_intensity_clamped() {
        let mut v = new_glitch_view();
        glv_set_intensity(&mut v, 2.0);
        assert!((v.intensity - 1.0).abs() < 1e-6 /* intensity clamped to 1.0 */);
    }

    #[test]
    fn test_frequency_clamped() {
        let mut v = new_glitch_view();
        glv_set_frequency(&mut v, -0.5);
        assert!((v.frequency).abs() < 1e-6 /* frequency clamped to 0.0 */);
    }

    #[test]
    fn test_set_seed() {
        let mut v = new_glitch_view();
        glv_set_seed(&mut v, 12345);
        assert_eq!(v.seed, 12345 /* seed must be set */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_glitch_view();
        glv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_type() {
        let v = new_glitch_view();
        let j = glv_to_json(&v);
        assert!(j.contains("\"glitch_type\"") /* JSON must have glitch_type */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_glitch_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_default_intensity() {
        let v = new_glitch_view();
        assert!((v.intensity - 0.3).abs() < 1e-6 /* default intensity must be 0.3 */);
    }

    #[test]
    fn test_default_seed() {
        let v = new_glitch_view();
        assert_eq!(v.seed, 42 /* default seed must be 42 */);
    }
}
