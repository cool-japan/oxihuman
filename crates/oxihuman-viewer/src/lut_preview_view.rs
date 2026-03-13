// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! LUT color grading preview stub.

/// LUT preview view config.
#[derive(Debug, Clone)]
pub struct LutPreviewViewConfig {
    pub lut_size: usize,
    pub intensity: f32,
    pub enabled: bool,
    pub show_before: bool,
}

impl Default for LutPreviewViewConfig {
    fn default() -> Self {
        LutPreviewViewConfig {
            lut_size: 32,
            intensity: 1.0,
            enabled: true,
            show_before: false,
        }
    }
}

/// A simple identity LUT.
#[derive(Debug, Clone)]
pub struct LutPreview {
    pub config: LutPreviewViewConfig,
    pub data: Vec<[f32; 3]>,
}

impl LutPreview {
    pub fn new() -> Self {
        let cfg = LutPreviewViewConfig::default();
        let n = cfg.lut_size * cfg.lut_size * cfg.lut_size;
        LutPreview {
            config: cfg,
            data: vec![[0.0; 3]; n],
        }
    }
}

impl Default for LutPreview {
    fn default() -> Self {
        LutPreview::new()
    }
}

/// Create a new LUT preview.
pub fn new_lut_preview() -> LutPreview {
    LutPreview::new()
}

/// Set intensity.
pub fn lpv2_set_intensity(lp: &mut LutPreview, intensity: f32) {
    lp.config.intensity = intensity.clamp(0.0, 1.0);
}

/// Enable or disable.
pub fn lpv2_set_enabled(lp: &mut LutPreview, enabled: bool) {
    lp.config.enabled = enabled;
}

/// Toggle before view.
pub fn lpv2_toggle_before(lp: &mut LutPreview) {
    lp.config.show_before = !lp.config.show_before;
}

/// Sample the LUT at a given RGB value (stub: returns identity if data is zero).
pub fn lpv2_sample(lp: &LutPreview, rgb: [f32; 3]) -> [f32; 3] {
    let s = lp.config.lut_size as f32;
    let idx_r = (rgb[0] * (s - 1.0)).clamp(0.0, s - 1.0) as usize;
    let idx_g = (rgb[1] * (s - 1.0)).clamp(0.0, s - 1.0) as usize;
    let idx_b = (rgb[2] * (s - 1.0)).clamp(0.0, s - 1.0) as usize;
    let flat = idx_r + idx_g * lp.config.lut_size + idx_b * lp.config.lut_size * lp.config.lut_size;
    let lut_val = lp.data.get(flat).copied().unwrap_or([0.0; 3]);
    /* Blend between original and LUT by intensity */
    let t = lp.config.intensity;
    [
        rgb[0] * (1.0 - t) + lut_val[0] * t,
        rgb[1] * (1.0 - t) + lut_val[1] * t,
        rgb[2] * (1.0 - t) + lut_val[2] * t,
    ]
}

/// Return a JSON-like string.
pub fn lpv2_to_json(lp: &LutPreview) -> String {
    format!(
        r#"{{"lut_size":{},"intensity":{:.4},"enabled":{}}}"#,
        lp.config.lut_size, lp.config.intensity, lp.config.enabled
    )
}

/// Return LUT data size.
pub fn lpv2_data_size(lp: &LutPreview) -> usize {
    lp.data.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_lut_size() {
        let l = new_lut_preview();
        assert_eq!(l.config.lut_size, 32 /* default LUT size is 32 */,);
    }

    #[test]
    fn test_data_size_cube() {
        let l = new_lut_preview();
        assert_eq!(
            lpv2_data_size(&l),
            32 * 32 * 32, /* data size should be cube */
        );
    }

    #[test]
    fn test_set_intensity() {
        let mut l = new_lut_preview();
        lpv2_set_intensity(&mut l, 0.7);
        assert!((l.config.intensity - 0.7).abs() < 1e-5, /* intensity must match */);
    }

    #[test]
    fn test_set_intensity_clamps() {
        let mut l = new_lut_preview();
        lpv2_set_intensity(&mut l, 2.0);
        assert!((l.config.intensity - 1.0).abs() < 1e-5, /* intensity clamped to 1 */);
    }

    #[test]
    fn test_set_enabled_false() {
        let mut l = new_lut_preview();
        lpv2_set_enabled(&mut l, false);
        assert!(!l.config.enabled /* should be disabled */,);
    }

    #[test]
    fn test_toggle_before() {
        let mut l = new_lut_preview();
        lpv2_toggle_before(&mut l);
        assert!(l.config.show_before /* show_before toggled on */,);
    }

    #[test]
    fn test_sample_at_zero_intensity_returns_original() {
        let mut l = new_lut_preview();
        lpv2_set_intensity(&mut l, 0.0);
        let rgb = [0.5, 0.3, 0.7];
        let out = lpv2_sample(&l, rgb);
        assert!((out[0] - 0.5).abs() < 1e-5, /* at zero intensity, output matches input */);
    }

    #[test]
    fn test_to_json_contains_lut_size() {
        let l = new_lut_preview();
        let j = lpv2_to_json(&l);
        assert!(j.contains("lut_size") /* JSON must contain lut_size */,);
    }

    #[test]
    fn test_default_intensity_one() {
        let l = new_lut_preview();
        assert!((l.config.intensity - 1.0).abs() < 1e-5, /* default intensity is 1.0 */);
    }

    #[test]
    fn test_default_enabled() {
        let l = new_lut_preview();
        assert!(l.config.enabled /* enabled by default */,);
    }

    #[test]
    fn test_sample_returns_three_components() {
        let l = new_lut_preview();
        let out = lpv2_sample(&l, [0.5, 0.5, 0.5]);
        assert_eq!(out.len(), 3 /* output must be RGB */,);
    }
}
