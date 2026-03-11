// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Temperature field visualization — color-maps thermal data across a grid.

/// Temperature field view configuration.
#[derive(Debug, Clone)]
pub struct TemperatureFieldView {
    pub enabled: bool,
    pub temp_min: f32,
    pub temp_max: f32,
    pub show_isotherms: bool,
    pub isotherm_count: u32,
}

impl TemperatureFieldView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            temp_min: 273.15, /* 0°C in Kelvin */
            temp_max: 373.15, /* 100°C in Kelvin */
            show_isotherms: false,
            isotherm_count: 5,
        }
    }
}

impl Default for TemperatureFieldView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new temperature field view.
pub fn new_temperature_field_view() -> TemperatureFieldView {
    TemperatureFieldView::new()
}

/// Enable or disable temperature field display.
pub fn tfv_set_enabled(v: &mut TemperatureFieldView, enabled: bool) {
    v.enabled = enabled;
}

/// Set minimum displayed temperature in Kelvin.
pub fn tfv_set_temp_min(v: &mut TemperatureFieldView, t: f32) {
    v.temp_min = t.max(0.0);
}

/// Set maximum displayed temperature in Kelvin.
pub fn tfv_set_temp_max(v: &mut TemperatureFieldView, t: f32) {
    v.temp_max = t.max(v.temp_min + 1e-3);
}

/// Toggle isotherm line overlay.
pub fn tfv_set_show_isotherms(v: &mut TemperatureFieldView, show: bool) {
    v.show_isotherms = show;
}

/// Set isotherm line count.
pub fn tfv_set_isotherm_count(v: &mut TemperatureFieldView, count: u32) {
    v.isotherm_count = count.clamp(2, 32);
}

/// Normalize temperature to 0-1 for color lookup.
pub fn tfv_normalize(v: &TemperatureFieldView, temp: f32) -> f32 {
    let range = v.temp_max - v.temp_min;
    if range < 1e-9 {
        return 0.0;
    }
    ((temp - v.temp_min) / range).clamp(0.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn temperature_field_view_to_json(v: &TemperatureFieldView) -> String {
    format!(
        r#"{{"enabled":{},"temp_min":{:.2},"temp_max":{:.2},"show_isotherms":{},"isotherm_count":{}}}"#,
        v.enabled, v.temp_min, v.temp_max, v.show_isotherms, v.isotherm_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_temperature_field_view();
        assert!(!v.enabled);
        assert!(!v.show_isotherms);
        assert_eq!(v.isotherm_count, 5);
    }

    #[test]
    fn test_enable() {
        let mut v = new_temperature_field_view();
        tfv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_temp_min_clamp() {
        let mut v = new_temperature_field_view();
        tfv_set_temp_min(&mut v, -100.0);
        assert_eq!(v.temp_min, 0.0);
    }

    #[test]
    fn test_temp_max_set() {
        let mut v = new_temperature_field_view();
        tfv_set_temp_max(&mut v, 1000.0);
        assert!((v.temp_max - 1000.0).abs() < 1e-3);
    }

    #[test]
    fn test_isotherms_toggle() {
        let mut v = new_temperature_field_view();
        tfv_set_show_isotherms(&mut v, true);
        assert!(v.show_isotherms);
    }

    #[test]
    fn test_isotherm_count_clamp() {
        let mut v = new_temperature_field_view();
        tfv_set_isotherm_count(&mut v, 1);
        assert_eq!(v.isotherm_count, 2);
    }

    #[test]
    fn test_normalize_mid() {
        let v = new_temperature_field_view(); /* 273.15..373.15 */
        let mid = (v.temp_min + v.temp_max) / 2.0;
        assert!((tfv_normalize(&v, mid) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_json_keys() {
        let v = new_temperature_field_view();
        let s = temperature_field_view_to_json(&v);
        assert!(s.contains("isotherm_count"));
    }

    #[test]
    fn test_clone() {
        let v = new_temperature_field_view();
        let v2 = v.clone();
        assert!((v2.temp_min - v.temp_min).abs() < 1e-4);
    }
}
