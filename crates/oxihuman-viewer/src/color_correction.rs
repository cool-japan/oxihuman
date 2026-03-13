// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Color correction / LUT application post-process.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColorCorrectionConfig {
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub hue_shift: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColorCorrectionState {
    pub config: ColorCorrectionConfig,
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn default_color_correction_config() -> ColorCorrectionConfig {
    ColorCorrectionConfig {
        brightness: 0.0,
        contrast: 1.0,
        saturation: 1.0,
        hue_shift: 0.0,
    }
}

#[allow(dead_code)]
pub fn new_color_correction_state() -> ColorCorrectionState {
    ColorCorrectionState {
        config: default_color_correction_config(),
        enabled: false,
    }
}

#[allow(dead_code)]
pub fn cc_apply(state: &ColorCorrectionState, color: [f32; 3]) -> [f32; 3] {
    if !state.enabled {
        return color;
    }
    let cfg = &state.config;
    // Apply brightness
    let r = (color[0] + cfg.brightness).clamp(0.0, 1.0);
    let g = (color[1] + cfg.brightness).clamp(0.0, 1.0);
    let b = (color[2] + cfg.brightness).clamp(0.0, 1.0);
    // Apply contrast: (v - 0.5) * contrast + 0.5
    let r = ((r - 0.5) * cfg.contrast + 0.5).clamp(0.0, 1.0);
    let g = ((g - 0.5) * cfg.contrast + 0.5).clamp(0.0, 1.0);
    let b = ((b - 0.5) * cfg.contrast + 0.5).clamp(0.0, 1.0);
    // Apply saturation via luminance
    let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let r = (lum + (r - lum) * cfg.saturation).clamp(0.0, 1.0);
    let g = (lum + (g - lum) * cfg.saturation).clamp(0.0, 1.0);
    let b = (lum + (b - lum) * cfg.saturation).clamp(0.0, 1.0);
    [r, g, b]
}

#[allow(dead_code)]
pub fn cc_set_brightness(state: &mut ColorCorrectionState, v: f32) {
    state.config.brightness = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn cc_set_contrast(state: &mut ColorCorrectionState, v: f32) {
    state.config.contrast = v.clamp(0.0, 4.0);
}

#[allow(dead_code)]
pub fn cc_set_saturation(state: &mut ColorCorrectionState, v: f32) {
    state.config.saturation = v.clamp(0.0, 4.0);
}

#[allow(dead_code)]
pub fn cc_set_enabled(state: &mut ColorCorrectionState, enabled: bool) {
    state.enabled = enabled;
}

#[allow(dead_code)]
pub fn cc_to_json(state: &ColorCorrectionState) -> String {
    format!(
        r#"{{"enabled":{},"brightness":{:.4},"contrast":{:.4},"saturation":{:.4},"hue_shift":{:.4}}}"#,
        state.enabled,
        state.config.brightness,
        state.config.contrast,
        state.config.saturation,
        state.config.hue_shift,
    )
}

#[allow(dead_code)]
pub fn cc_reset(state: &mut ColorCorrectionState) {
    *state = new_color_correction_state();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_color_correction_config();
        assert!((cfg.brightness - 0.0).abs() < 1e-6);
        assert!((cfg.contrast - 1.0).abs() < 1e-6);
        assert!((cfg.saturation - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_disabled() {
        let s = new_color_correction_state();
        assert!(!s.enabled);
    }

    #[test]
    fn test_apply_disabled_passthrough() {
        let s = new_color_correction_state();
        let c = [0.5, 0.3, 0.7];
        let out = cc_apply(&s, c);
        assert!((out[0] - 0.5).abs() < 1e-6);
        assert!((out[1] - 0.3).abs() < 1e-6);
        assert!((out[2] - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_apply_enabled_identity() {
        let mut s = new_color_correction_state();
        cc_set_enabled(&mut s, true);
        let c = [0.5, 0.5, 0.5];
        let out = cc_apply(&s, c);
        // With default config (brightness=0, contrast=1, sat=1), grey stays grey
        assert!((out[0] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_set_brightness_clamps() {
        let mut s = new_color_correction_state();
        cc_set_brightness(&mut s, 5.0);
        assert!((s.config.brightness - 1.0).abs() < 1e-6);
        cc_set_brightness(&mut s, -5.0);
        assert!((s.config.brightness - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_set_contrast_clamps() {
        let mut s = new_color_correction_state();
        cc_set_contrast(&mut s, 10.0);
        assert!((s.config.contrast - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_saturation_clamps() {
        let mut s = new_color_correction_state();
        cc_set_saturation(&mut s, -1.0);
        assert!((s.config.saturation - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_enabled() {
        let mut s = new_color_correction_state();
        cc_set_enabled(&mut s, true);
        assert!(s.enabled);
    }

    #[test]
    fn test_to_json() {
        let s = new_color_correction_state();
        let j = cc_to_json(&s);
        assert!(j.contains("brightness"));
        assert!(j.contains("saturation"));
        assert!(j.contains("enabled"));
    }

    #[test]
    fn test_reset() {
        let mut s = new_color_correction_state();
        cc_set_enabled(&mut s, true);
        cc_set_brightness(&mut s, 0.8);
        cc_reset(&mut s);
        assert!(!s.enabled);
        assert!((s.config.brightness - 0.0).abs() < 1e-6);
    }
}
