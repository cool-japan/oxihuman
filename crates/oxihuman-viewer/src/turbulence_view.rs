// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Turbulence magnitude visualization — shows turbulent kinetic energy in fluid fields.

/// Turbulence view configuration.
#[derive(Debug, Clone)]
pub struct TurbulenceView {
    pub enabled: bool,
    pub tke_min: f32,
    pub tke_max: f32,
    pub show_eddies: bool,
    pub eddy_scale: f32,
}

impl TurbulenceView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            tke_min: 0.0,
            tke_max: 1.0,
            show_eddies: false,
            eddy_scale: 0.2,
        }
    }
}

impl Default for TurbulenceView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new turbulence view.
pub fn new_turbulence_view() -> TurbulenceView {
    TurbulenceView::new()
}

/// Enable or disable turbulence display.
pub fn turb_set_enabled(v: &mut TurbulenceView, enabled: bool) {
    v.enabled = enabled;
}

/// Set minimum TKE for color mapping.
pub fn turb_set_tke_min(v: &mut TurbulenceView, tke: f32) {
    v.tke_min = tke.max(0.0);
}

/// Set maximum TKE for color mapping.
pub fn turb_set_tke_max(v: &mut TurbulenceView, tke: f32) {
    v.tke_max = tke.max(v.tke_min + 1e-6);
}

/// Toggle eddy glyph display.
pub fn turb_set_show_eddies(v: &mut TurbulenceView, show: bool) {
    v.show_eddies = show;
}

/// Set eddy glyph scale.
pub fn turb_set_eddy_scale(v: &mut TurbulenceView, scale: f32) {
    v.eddy_scale = scale.clamp(0.01, 5.0);
}

/// Normalize TKE to 0-1 range.
pub fn turb_normalize_tke(v: &TurbulenceView, tke: f32) -> f32 {
    let range = v.tke_max - v.tke_min;
    if range < 1e-9 {
        return 0.0;
    }
    ((tke - v.tke_min) / range).clamp(0.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn turbulence_view_to_json(v: &TurbulenceView) -> String {
    format!(
        r#"{{"enabled":{},"tke_min":{:.4},"tke_max":{:.4},"show_eddies":{},"eddy_scale":{:.4}}}"#,
        v.enabled, v.tke_min, v.tke_max, v.show_eddies, v.eddy_scale
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_turbulence_view();
        assert!(!v.enabled);
        assert!(!v.show_eddies);
    }

    #[test]
    fn test_enable() {
        let mut v = new_turbulence_view();
        turb_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_tke_min_clamp() {
        let mut v = new_turbulence_view();
        turb_set_tke_min(&mut v, -1.0);
        assert_eq!(v.tke_min, 0.0);
    }

    #[test]
    fn test_tke_max_set() {
        let mut v = new_turbulence_view();
        turb_set_tke_max(&mut v, 10.0);
        assert!((v.tke_max - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_eddies_toggle() {
        let mut v = new_turbulence_view();
        turb_set_show_eddies(&mut v, true);
        assert!(v.show_eddies);
    }

    #[test]
    fn test_eddy_scale_set() {
        let mut v = new_turbulence_view();
        turb_set_eddy_scale(&mut v, 0.5);
        assert!((v.eddy_scale - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_mid() {
        let v = new_turbulence_view(); /* range 0-1 */
        assert!((turb_normalize_tke(&v, 0.5) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_clamp() {
        let v = new_turbulence_view();
        assert_eq!(turb_normalize_tke(&v, 2.0), 1.0); /* clamped */
    }

    #[test]
    fn test_json_keys() {
        let v = new_turbulence_view();
        let s = turbulence_view_to_json(&v);
        assert!(s.contains("eddy_scale"));
    }

    #[test]
    fn test_clone() {
        let v = new_turbulence_view();
        let v2 = v.clone();
        assert!((v2.eddy_scale - v.eddy_scale).abs() < 1e-6);
    }
}
