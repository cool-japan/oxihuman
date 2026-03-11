// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Level set isosurface view — shows the zero-level surface of a scalar field.

/// Level set view configuration.
#[derive(Debug, Clone)]
pub struct LevelSetView {
    pub enabled: bool,
    pub iso_value: f32,
    pub show_gradient: bool,
    pub surface_opacity: f32,
    pub wireframe: bool,
}

impl LevelSetView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            iso_value: 0.0,
            show_gradient: false,
            surface_opacity: 0.8,
            wireframe: false,
        }
    }
}

impl Default for LevelSetView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new level set view.
pub fn new_level_set_view() -> LevelSetView {
    LevelSetView::new()
}

/// Enable or disable level set display.
pub fn lsv_set_enabled(v: &mut LevelSetView, enabled: bool) {
    v.enabled = enabled;
}

/// Set the iso-value to extract.
pub fn lsv_set_iso_value(v: &mut LevelSetView, iso: f32) {
    v.iso_value = iso;
}

/// Toggle gradient arrow display.
pub fn lsv_set_show_gradient(v: &mut LevelSetView, show: bool) {
    v.show_gradient = show;
}

/// Set surface opacity.
pub fn lsv_set_surface_opacity(v: &mut LevelSetView, o: f32) {
    v.surface_opacity = o.clamp(0.0, 1.0);
}

/// Toggle wireframe overlay.
pub fn lsv_set_wireframe(v: &mut LevelSetView, wf: bool) {
    v.wireframe = wf;
}

/// Serialize to JSON-like string.
pub fn level_set_view_to_json(v: &LevelSetView) -> String {
    format!(
        r#"{{"enabled":{},"iso_value":{:.4},"show_gradient":{},"surface_opacity":{:.4},"wireframe":{}}}"#,
        v.enabled, v.iso_value, v.show_gradient, v.surface_opacity, v.wireframe
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_level_set_view();
        assert!(!v.enabled);
        assert_eq!(v.iso_value, 0.0);
    }

    #[test]
    fn test_enable() {
        let mut v = new_level_set_view();
        lsv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_iso_value_set() {
        let mut v = new_level_set_view();
        lsv_set_iso_value(&mut v, 0.5);
        assert!((v.iso_value - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_iso_value_negative() {
        let mut v = new_level_set_view();
        lsv_set_iso_value(&mut v, -1.5);
        assert!((v.iso_value - (-1.5)).abs() < 1e-6);
    }

    #[test]
    fn test_gradient_toggle() {
        let mut v = new_level_set_view();
        lsv_set_show_gradient(&mut v, true);
        assert!(v.show_gradient);
    }

    #[test]
    fn test_opacity_clamp() {
        let mut v = new_level_set_view();
        lsv_set_surface_opacity(&mut v, 3.0);
        assert_eq!(v.surface_opacity, 1.0);
    }

    #[test]
    fn test_wireframe_toggle() {
        let mut v = new_level_set_view();
        lsv_set_wireframe(&mut v, true);
        assert!(v.wireframe);
    }

    #[test]
    fn test_json_keys() {
        let v = new_level_set_view();
        let s = level_set_view_to_json(&v);
        assert!(s.contains("iso_value"));
    }

    #[test]
    fn test_clone() {
        let v = new_level_set_view();
        let v2 = v.clone();
        assert!((v2.surface_opacity - v.surface_opacity).abs() < 1e-6);
    }
}
