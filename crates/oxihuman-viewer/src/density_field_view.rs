// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Density field visualization — renders mass density distribution in a simulation grid.

/// Density field view configuration.
#[derive(Debug, Clone)]
pub struct DensityFieldView {
    pub enabled: bool,
    pub density_min: f32,
    pub density_max: f32,
    pub log_scale: bool,
    pub opacity: f32,
}

impl DensityFieldView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            density_min: 0.0,
            density_max: 1000.0,
            log_scale: false,
            opacity: 0.7,
        }
    }
}

impl Default for DensityFieldView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new density field view.
pub fn new_density_field_view() -> DensityFieldView {
    DensityFieldView::new()
}

/// Enable or disable density display.
pub fn dfv_set_enabled(v: &mut DensityFieldView, enabled: bool) {
    v.enabled = enabled;
}

/// Set minimum density for color mapping.
pub fn dfv_set_density_min(v: &mut DensityFieldView, d: f32) {
    v.density_min = d.max(0.0);
}

/// Set maximum density for color mapping.
pub fn dfv_set_density_max(v: &mut DensityFieldView, d: f32) {
    v.density_max = d.max(v.density_min + 1e-6);
}

/// Toggle logarithmic density scale.
pub fn dfv_set_log_scale(v: &mut DensityFieldView, log: bool) {
    v.log_scale = log;
}

/// Set overlay opacity.
pub fn dfv_set_opacity(v: &mut DensityFieldView, o: f32) {
    v.opacity = o.clamp(0.0, 1.0);
}

/// Normalize density to 0-1.
pub fn dfv_normalize(v: &DensityFieldView, density: f32) -> f32 {
    let d = density.max(0.0);
    if v.log_scale {
        let lo = (v.density_min + 1.0).ln();
        let hi = (v.density_max + 1.0).ln();
        if (hi - lo).abs() < 1e-9 {
            return 0.0;
        }
        (((d + 1.0).ln() - lo) / (hi - lo)).clamp(0.0, 1.0)
    } else {
        let range = v.density_max - v.density_min;
        if range < 1e-9 {
            return 0.0;
        }
        ((d - v.density_min) / range).clamp(0.0, 1.0)
    }
}

/// Serialize to JSON-like string.
pub fn density_field_view_to_json(v: &DensityFieldView) -> String {
    format!(
        r#"{{"enabled":{},"density_min":{:.4},"density_max":{:.4},"log_scale":{},"opacity":{:.4}}}"#,
        v.enabled, v.density_min, v.density_max, v.log_scale, v.opacity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_density_field_view();
        assert!(!v.enabled);
        assert!(!v.log_scale);
    }

    #[test]
    fn test_enable() {
        let mut v = new_density_field_view();
        dfv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_density_min_clamp() {
        let mut v = new_density_field_view();
        dfv_set_density_min(&mut v, -10.0);
        assert_eq!(v.density_min, 0.0);
    }

    #[test]
    fn test_density_max_set() {
        let mut v = new_density_field_view();
        dfv_set_density_max(&mut v, 2000.0);
        assert!((v.density_max - 2000.0).abs() < 1e-3);
    }

    #[test]
    fn test_log_scale_toggle() {
        let mut v = new_density_field_view();
        dfv_set_log_scale(&mut v, true);
        assert!(v.log_scale);
    }

    #[test]
    fn test_opacity_clamp() {
        let mut v = new_density_field_view();
        dfv_set_opacity(&mut v, 3.0);
        assert_eq!(v.opacity, 1.0);
    }

    #[test]
    fn test_normalize_linear_zero() {
        let v = new_density_field_view();
        assert_eq!(dfv_normalize(&v, 0.0), 0.0);
    }

    #[test]
    fn test_normalize_linear_max() {
        let v = new_density_field_view();
        assert!((dfv_normalize(&v, 1000.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_json_keys() {
        let v = new_density_field_view();
        let s = density_field_view_to_json(&v);
        assert!(s.contains("log_scale"));
    }

    #[test]
    fn test_clone() {
        let v = new_density_field_view();
        let v2 = v.clone();
        assert!((v2.opacity - v.opacity).abs() < 1e-6);
    }
}
