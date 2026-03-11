// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fire simulation debug view — heat temperature and fuel density display.

/// Fire debug view configuration.
#[derive(Debug, Clone)]
pub struct FireDebugView {
    pub enabled: bool,
    pub show_temperature: bool,
    pub show_fuel: bool,
    pub temp_min: f32,
    pub temp_max: f32,
}

impl FireDebugView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            show_temperature: true,
            show_fuel: false,
            temp_min: 300.0,
            temp_max: 2000.0,
        }
    }
}

impl Default for FireDebugView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new fire debug view.
pub fn new_fire_debug_view() -> FireDebugView {
    FireDebugView::new()
}

/// Enable or disable fire debug display.
pub fn frdv_set_enabled(v: &mut FireDebugView, enabled: bool) {
    v.enabled = enabled;
}

/// Toggle temperature visualization.
pub fn frdv_set_show_temperature(v: &mut FireDebugView, show: bool) {
    v.show_temperature = show;
}

/// Toggle fuel density visualization.
pub fn frdv_set_show_fuel(v: &mut FireDebugView, show: bool) {
    v.show_fuel = show;
}

/// Set minimum displayed temperature (Kelvin).
pub fn frdv_set_temp_min(v: &mut FireDebugView, t: f32) {
    v.temp_min = t.max(0.0);
}

/// Set maximum displayed temperature (Kelvin).
pub fn frdv_set_temp_max(v: &mut FireDebugView, t: f32) {
    v.temp_max = t.max(v.temp_min + 1.0);
}

/// Normalize a temperature value to 0-1 range for display.
pub fn frdv_normalize_temp(v: &FireDebugView, temp: f32) -> f32 {
    let range = v.temp_max - v.temp_min;
    if range < 1e-6 {
        return 0.0;
    }
    ((temp - v.temp_min) / range).clamp(0.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn fire_debug_view_to_json(v: &FireDebugView) -> String {
    format!(
        r#"{{"enabled":{},"show_temperature":{},"show_fuel":{},"temp_min":{:.2},"temp_max":{:.2}}}"#,
        v.enabled, v.show_temperature, v.show_fuel, v.temp_min, v.temp_max
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_fire_debug_view();
        assert!(!v.enabled);
        assert!(v.show_temperature);
        assert!(!v.show_fuel);
    }

    #[test]
    fn test_enable() {
        let mut v = new_fire_debug_view();
        frdv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_toggle_fuel() {
        let mut v = new_fire_debug_view();
        frdv_set_show_fuel(&mut v, true);
        assert!(v.show_fuel);
    }

    #[test]
    fn test_temp_min_clamped() {
        let mut v = new_fire_debug_view();
        frdv_set_temp_min(&mut v, -100.0);
        assert_eq!(v.temp_min, 0.0);
    }

    #[test]
    fn test_temp_max_set() {
        let mut v = new_fire_debug_view();
        frdv_set_temp_max(&mut v, 3000.0);
        assert!((v.temp_max - 3000.0).abs() < 1e-4);
    }

    #[test]
    fn test_normalize_mid() {
        let v = new_fire_debug_view(); /* range 300-2000 */
        let n = frdv_normalize_temp(&v, 1150.0);
        assert!((n - 0.5).abs() < 0.01); /* midpoint */
    }

    #[test]
    fn test_normalize_clamp_below() {
        let v = new_fire_debug_view();
        assert_eq!(frdv_normalize_temp(&v, 0.0), 0.0);
    }

    #[test]
    fn test_json_keys() {
        let v = new_fire_debug_view();
        let s = fire_debug_view_to_json(&v);
        assert!(s.contains("temp_max"));
    }

    #[test]
    fn test_clone() {
        let v = new_fire_debug_view();
        let v2 = v.clone();
        assert!((v2.temp_min - v.temp_min).abs() < 1e-4);
    }
}
