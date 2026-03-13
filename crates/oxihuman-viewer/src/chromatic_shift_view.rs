// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Chromatic aberration debug view stub.

/// Chromatic shift view config.
#[derive(Debug, Clone)]
pub struct ChromaticShiftViewConfig {
    pub red_offset: [f32; 2],
    pub blue_offset: [f32; 2],
    pub intensity: f32,
    pub enabled: bool,
    pub show_channels: bool,
}

impl Default for ChromaticShiftViewConfig {
    fn default() -> Self {
        ChromaticShiftViewConfig {
            red_offset: [-0.002, 0.0],
            blue_offset: [0.002, 0.0],
            intensity: 1.0,
            enabled: true,
            show_channels: false,
        }
    }
}

/// Create a new chromatic shift view config.
pub fn new_chromatic_shift_view() -> ChromaticShiftViewConfig {
    ChromaticShiftViewConfig::default()
}

/// Set red channel offset.
pub fn csv_set_red_offset(cfg: &mut ChromaticShiftViewConfig, offset: [f32; 2]) {
    cfg.red_offset = offset;
}

/// Set blue channel offset.
pub fn csv_set_blue_offset(cfg: &mut ChromaticShiftViewConfig, offset: [f32; 2]) {
    cfg.blue_offset = offset;
}

/// Set intensity.
pub fn csv_set_intensity(cfg: &mut ChromaticShiftViewConfig, intensity: f32) {
    cfg.intensity = intensity.clamp(0.0, 1.0);
}

/// Enable or disable.
pub fn csv_set_enabled(cfg: &mut ChromaticShiftViewConfig, enabled: bool) {
    cfg.enabled = enabled;
}

/// Toggle channel visualization.
pub fn csv_toggle_channels(cfg: &mut ChromaticShiftViewConfig) {
    cfg.show_channels = !cfg.show_channels;
}

/// Compute shifted UV for a given channel.
pub fn csv_shifted_uv(cfg: &ChromaticShiftViewConfig, uv: [f32; 2], is_red: bool) -> [f32; 2] {
    let offset = if is_red {
        cfg.red_offset
    } else {
        cfg.blue_offset
    };
    [
        uv[0] + offset[0] * cfg.intensity,
        uv[1] + offset[1] * cfg.intensity,
    ]
}

/// Return the shift magnitude for a channel.
pub fn csv_shift_magnitude(cfg: &ChromaticShiftViewConfig, is_red: bool) -> f32 {
    let o = if is_red {
        cfg.red_offset
    } else {
        cfg.blue_offset
    };
    (o[0] * o[0] + o[1] * o[1]).sqrt() * cfg.intensity
}

/// Return a JSON-like string.
pub fn csv_to_json(cfg: &ChromaticShiftViewConfig) -> String {
    format!(
        r#"{{"intensity":{:.4},"enabled":{}}}"#,
        cfg.intensity, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_red_offset() {
        let c = new_chromatic_shift_view();
        assert!((c.red_offset[0] - (-0.002)).abs() < 1e-5, /* default red x offset */);
    }

    #[test]
    fn test_set_red_offset() {
        let mut c = new_chromatic_shift_view();
        csv_set_red_offset(&mut c, [0.01, 0.005]);
        assert!((c.red_offset[0] - 0.01).abs() < 1e-5, /* red offset x must match */);
    }

    #[test]
    fn test_set_blue_offset() {
        let mut c = new_chromatic_shift_view();
        csv_set_blue_offset(&mut c, [-0.01, 0.0]);
        assert!((c.blue_offset[0] - (-0.01)).abs() < 1e-5, /* blue offset x must match */);
    }

    #[test]
    fn test_set_intensity() {
        let mut c = new_chromatic_shift_view();
        csv_set_intensity(&mut c, 0.6);
        assert!((c.intensity - 0.6).abs() < 1e-5, /* intensity must match */);
    }

    #[test]
    fn test_set_intensity_clamps() {
        let mut c = new_chromatic_shift_view();
        csv_set_intensity(&mut c, 3.0);
        assert!((c.intensity - 1.0).abs() < 1e-5, /* intensity clamped to 1 */);
    }

    #[test]
    fn test_set_enabled_false() {
        let mut c = new_chromatic_shift_view();
        csv_set_enabled(&mut c, false);
        assert!(!c.enabled /* should be disabled */,);
    }

    #[test]
    fn test_toggle_channels() {
        let mut c = new_chromatic_shift_view();
        csv_toggle_channels(&mut c);
        assert!(c.show_channels /* channels toggled on */,);
    }

    #[test]
    fn test_shifted_uv_zero_intensity() {
        let mut c = new_chromatic_shift_view();
        csv_set_intensity(&mut c, 0.0);
        let uv = csv_shifted_uv(&c, [0.5, 0.5], true);
        assert!((uv[0] - 0.5).abs() < 1e-5, /* zero intensity means no shift */);
    }

    #[test]
    fn test_shift_magnitude_positive() {
        let c = new_chromatic_shift_view();
        let m = csv_shift_magnitude(&c, true);
        assert!(m > 0.0, /* shift magnitude should be positive with non-zero offset */);
    }

    #[test]
    fn test_to_json_contains_intensity() {
        let c = new_chromatic_shift_view();
        let j = csv_to_json(&c);
        assert!(j.contains("intensity"), /* JSON must contain intensity */);
    }
}
