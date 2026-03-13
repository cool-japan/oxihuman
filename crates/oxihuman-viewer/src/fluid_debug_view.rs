// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Fluid simulation debug view — visualizes fluid cells, pressure, and velocity.

/// Fluid debug view configuration.
#[derive(Debug, Clone)]
pub struct FluidDebugView {
    pub enabled: bool,
    pub show_pressure: bool,
    pub show_velocity: bool,
    pub velocity_scale: f32,
    pub opacity: f32,
}

impl FluidDebugView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            show_pressure: true,
            show_velocity: false,
            velocity_scale: 0.1,
            opacity: 0.7,
        }
    }
}

impl Default for FluidDebugView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new fluid debug view.
pub fn new_fluid_debug_view() -> FluidDebugView {
    FluidDebugView::new()
}

/// Enable or disable the fluid debug view.
pub fn fdv_set_enabled(v: &mut FluidDebugView, enabled: bool) {
    v.enabled = enabled;
}

/// Toggle pressure visualization.
pub fn fdv_set_show_pressure(v: &mut FluidDebugView, show: bool) {
    v.show_pressure = show;
}

/// Toggle velocity arrow display.
pub fn fdv_set_show_velocity(v: &mut FluidDebugView, show: bool) {
    v.show_velocity = show;
}

/// Set velocity arrow scale.
pub fn fdv_set_velocity_scale(v: &mut FluidDebugView, scale: f32) {
    v.velocity_scale = scale.clamp(0.001, 10.0);
}

/// Set overlay opacity.
pub fn fdv_set_opacity(v: &mut FluidDebugView, opacity: f32) {
    v.opacity = opacity.clamp(0.0, 1.0);
}

/// Serialize to JSON-like string.
pub fn fluid_debug_view_to_json(v: &FluidDebugView) -> String {
    format!(
        r#"{{"enabled":{},"show_pressure":{},"show_velocity":{},"velocity_scale":{:.4},"opacity":{:.4}}}"#,
        v.enabled, v.show_pressure, v.show_velocity, v.velocity_scale, v.opacity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_fluid_debug_view();
        assert!(!v.enabled);
        assert!(v.show_pressure);
    }

    #[test]
    fn test_enable() {
        let mut v = new_fluid_debug_view();
        fdv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_pressure_toggle() {
        let mut v = new_fluid_debug_view();
        fdv_set_show_pressure(&mut v, false);
        assert!(!v.show_pressure);
    }

    #[test]
    fn test_velocity_toggle() {
        let mut v = new_fluid_debug_view();
        fdv_set_show_velocity(&mut v, true);
        assert!(v.show_velocity);
    }

    #[test]
    fn test_velocity_scale_clamp_low() {
        let mut v = new_fluid_debug_view();
        fdv_set_velocity_scale(&mut v, 0.0);
        assert_eq!(v.velocity_scale, 0.001);
    }

    #[test]
    fn test_velocity_scale_set() {
        let mut v = new_fluid_debug_view();
        fdv_set_velocity_scale(&mut v, 0.5);
        assert!((v.velocity_scale - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_opacity_clamp() {
        let mut v = new_fluid_debug_view();
        fdv_set_opacity(&mut v, 2.0);
        assert_eq!(v.opacity, 1.0);
    }

    #[test]
    fn test_json_keys() {
        let v = new_fluid_debug_view();
        let s = fluid_debug_view_to_json(&v);
        assert!(s.contains("velocity_scale"));
    }

    #[test]
    fn test_clone() {
        let v = new_fluid_debug_view();
        let v2 = v.clone();
        assert!((v2.opacity - v.opacity).abs() < 1e-6);
    }
}
