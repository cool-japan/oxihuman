// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Color temperature overlay view stub.

/// Color temperature view config.
#[derive(Debug, Clone)]
pub struct ColorTemperatureViewConfig {
    pub kelvin: f32,
    pub tint: f32,
    pub enabled: bool,
    pub show_gamut: bool,
}

impl Default for ColorTemperatureViewConfig {
    fn default() -> Self {
        ColorTemperatureViewConfig {
            kelvin: 6500.0,
            tint: 0.0,
            enabled: true,
            show_gamut: false,
        }
    }
}

/// Create a new color temperature view config.
pub fn new_color_temperature_view() -> ColorTemperatureViewConfig {
    ColorTemperatureViewConfig::default()
}

/// Set temperature in Kelvin.
pub fn ctv_set_kelvin(cfg: &mut ColorTemperatureViewConfig, kelvin: f32) {
    cfg.kelvin = kelvin.clamp(1000.0, 20000.0);
}

/// Set tint offset.
pub fn ctv_set_tint(cfg: &mut ColorTemperatureViewConfig, tint: f32) {
    cfg.tint = tint.clamp(-1.0, 1.0);
}

/// Enable or disable.
pub fn ctv_set_enabled(cfg: &mut ColorTemperatureViewConfig, enabled: bool) {
    cfg.enabled = enabled;
}

/// Toggle gamut visualization.
pub fn ctv_toggle_gamut(cfg: &mut ColorTemperatureViewConfig) {
    cfg.show_gamut = !cfg.show_gamut;
}

/// Convert Kelvin to approximate RGB (stub — simplified Tanner Helland method).
pub fn ctv_kelvin_to_rgb(kelvin: f32) -> [f32; 3] {
    let t = kelvin / 100.0;
    let r = if t <= 66.0 {
        1.0
    } else {
        (329.698_73 * (t - 60.0).powf(-0.133_205) / 255.0).clamp(0.0, 1.0)
    };
    let g = if t <= 66.0 {
        (99.470_8 * t.ln() - 161.119_57) / 255.0
    } else {
        288.122_17 * (t - 60.0).powf(-0.075_515) / 255.0
    }
    .clamp(0.0, 1.0);
    let b = if t >= 66.0 {
        1.0
    } else if t <= 19.0 {
        0.0
    } else {
        (138.517_73 * (t - 10.0).ln() - 305.044_8) / 255.0
    }
    .clamp(0.0, 1.0);
    [r, g, b]
}

/// Return a JSON-like string.
pub fn ctv_to_json(cfg: &ColorTemperatureViewConfig) -> String {
    format!(
        r#"{{"kelvin":{:.1},"tint":{:.4},"enabled":{}}}"#,
        cfg.kelvin, cfg.tint, cfg.enabled
    )
}

/// Check if the temperature is in the daylight range (5500-6500K).
pub fn ctv_is_daylight(cfg: &ColorTemperatureViewConfig) -> bool {
    (5500.0..=6500.0).contains(&cfg.kelvin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_kelvin() {
        let c = new_color_temperature_view();
        assert!((c.kelvin - 6500.0).abs() < 1e-3 /* default is 6500K */,);
    }

    #[test]
    fn test_set_kelvin() {
        let mut c = new_color_temperature_view();
        ctv_set_kelvin(&mut c, 3200.0);
        assert!((c.kelvin - 3200.0).abs() < 1e-3, /* kelvin must match */);
    }

    #[test]
    fn test_set_kelvin_clamps_low() {
        let mut c = new_color_temperature_view();
        ctv_set_kelvin(&mut c, 0.0);
        assert!(c.kelvin >= 1000.0 /* kelvin clamped to minimum */,);
    }

    #[test]
    fn test_set_tint() {
        let mut c = new_color_temperature_view();
        ctv_set_tint(&mut c, 0.5);
        assert!((c.tint - 0.5).abs() < 1e-5 /* tint must match */,);
    }

    #[test]
    fn test_set_tint_clamps() {
        let mut c = new_color_temperature_view();
        ctv_set_tint(&mut c, 2.0);
        assert!((c.tint - 1.0).abs() < 1e-5 /* tint clamped to 1 */,);
    }

    #[test]
    fn test_set_enabled_false() {
        let mut c = new_color_temperature_view();
        ctv_set_enabled(&mut c, false);
        assert!(!c.enabled /* should be disabled */,);
    }

    #[test]
    fn test_toggle_gamut() {
        let mut c = new_color_temperature_view();
        ctv_toggle_gamut(&mut c);
        assert!(c.show_gamut /* gamut toggled on */,);
    }

    #[test]
    fn test_kelvin_to_rgb_returns_three() {
        let rgb = ctv_kelvin_to_rgb(6500.0);
        assert_eq!(rgb.len(), 3 /* RGB must have 3 components */,);
    }

    #[test]
    fn test_is_daylight_at_6500() {
        let c = new_color_temperature_view();
        assert!(ctv_is_daylight(&c) /* 6500K is daylight */,);
    }

    #[test]
    fn test_to_json_contains_kelvin() {
        let c = new_color_temperature_view();
        let j = ctv_to_json(&c);
        assert!(j.contains("kelvin") /* JSON must contain kelvin */,);
    }
}
