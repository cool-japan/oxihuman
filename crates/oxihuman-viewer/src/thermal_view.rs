// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Thermal imaging overlay stub.

/// Thermal color palette.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThermalPalette {
    Rainbow,
    IronBow,
    Arctic,
    Lava,
}

/// Thermal view configuration.
#[derive(Debug, Clone)]
pub struct ThermalView {
    pub palette: ThermalPalette,
    pub min_temp_c: f32,
    pub max_temp_c: f32,
    pub noise_level: f32,
    pub enabled: bool,
}

impl ThermalView {
    pub fn new() -> Self {
        ThermalView {
            palette: ThermalPalette::IronBow,
            min_temp_c: 20.0,
            max_temp_c: 40.0,
            noise_level: 0.02,
            enabled: true,
        }
    }
}

impl Default for ThermalView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new thermal view.
pub fn new_thermal_view() -> ThermalView {
    ThermalView::new()
}

/// Map a normalized temperature (0..1) to a color using the palette.
pub fn thv_map_temp(thv: &ThermalView, t: f32) -> [f32; 3] {
    /* Stub: returns a simple gradient based on t */
    let t = t.clamp(0.0, 1.0);
    match thv.palette {
        ThermalPalette::Rainbow => [t, 1.0 - t, 0.5 - (t - 0.5).abs()],
        ThermalPalette::IronBow => [t, t * 0.5, 0.0],
        ThermalPalette::Arctic => [t * 0.5, t * 0.8, 1.0],
        ThermalPalette::Lava => [1.0, t * 0.5, 0.0],
    }
}

/// Set temperature range.
pub fn thv_set_range(thv: &mut ThermalView, min_c: f32, max_c: f32) {
    thv.min_temp_c = min_c;
    thv.max_temp_c = max_c.max(min_c + 0.1);
}

/// Set palette.
pub fn thv_set_palette(thv: &mut ThermalView, palette: ThermalPalette) {
    thv.palette = palette;
}

/// Enable or disable.
pub fn thv_set_enabled(thv: &mut ThermalView, enabled: bool) {
    thv.enabled = enabled;
}

/// Normalize a Celsius temperature to `[0,1]` within range.
pub fn thv_normalize_temp(thv: &ThermalView, temp_c: f32) -> f32 {
    let range = thv.max_temp_c - thv.min_temp_c;
    if range < 1e-6 {
        return 0.5;
    }
    ((temp_c - thv.min_temp_c) / range).clamp(0.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn thv_to_json(thv: &ThermalView) -> String {
    let p = match thv.palette {
        ThermalPalette::Rainbow => "rainbow",
        ThermalPalette::IronBow => "iron_bow",
        ThermalPalette::Arctic => "arctic",
        ThermalPalette::Lava => "lava",
    };
    format!(
        r#"{{"palette":"{}","min_temp":{},"max_temp":{},"enabled":{}}}"#,
        p, thv.min_temp_c, thv.max_temp_c, thv.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_palette_iron_bow() {
        let t = new_thermal_view();
        assert_eq!(
            t.palette,
            ThermalPalette::IronBow, /* default palette must be iron bow */
        );
    }

    #[test]
    fn test_normalize_min_gives_zero() {
        let t = new_thermal_view();
        let n = thv_normalize_temp(&t, t.min_temp_c);
        assert!((n).abs() < 1e-5 /* min temp must normalize to 0 */,);
    }

    #[test]
    fn test_normalize_max_gives_one() {
        let t = new_thermal_view();
        let n = thv_normalize_temp(&t, t.max_temp_c);
        assert!((n - 1.0).abs() < 1e-5, /* max temp must normalize to 1 */);
    }

    #[test]
    fn test_map_temp_clamps_input() {
        let t = new_thermal_view();
        let _ = thv_map_temp(&t, 2.0);
        let _ = thv_map_temp(&t, -1.0);
    }

    #[test]
    fn test_set_palette() {
        let mut t = new_thermal_view();
        thv_set_palette(&mut t, ThermalPalette::Rainbow);
        assert_eq!(
            t.palette,
            ThermalPalette::Rainbow, /* palette must be set */
        );
    }

    #[test]
    fn test_set_range() {
        let mut t = new_thermal_view();
        thv_set_range(&mut t, 30.0, 60.0);
        assert!((t.min_temp_c - 30.0).abs() < 1e-5, /* min temp must be set */);
        assert!((t.max_temp_c - 60.0).abs() < 1e-5, /* max temp must be set */);
    }

    #[test]
    fn test_set_enabled() {
        let mut t = new_thermal_view();
        thv_set_enabled(&mut t, false);
        assert!(!t.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_palette() {
        let t = new_thermal_view();
        let j = thv_to_json(&t);
        assert!(j.contains("\"palette\""), /* json must contain palette */);
    }

    #[test]
    fn test_enabled_default() {
        let t = new_thermal_view();
        assert!(t.enabled /* must be enabled by default */,);
    }

    #[test]
    fn test_noise_level_default() {
        let t = new_thermal_view();
        assert!(t.noise_level >= 0.0, /* noise level must be non-negative */);
    }
}
