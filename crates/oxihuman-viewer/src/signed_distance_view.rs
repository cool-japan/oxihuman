// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Signed distance field (SDF) value visualization — color-maps distance values.

/// Signed distance view configuration.
#[derive(Debug, Clone)]
pub struct SignedDistanceView {
    pub enabled: bool,
    pub band_width: f32,
    pub show_zero_crossing: bool,
    pub interior_color: [f32; 3],
    pub exterior_color: [f32; 3],
}

impl SignedDistanceView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            band_width: 5.0,
            show_zero_crossing: true,
            interior_color: [0.2, 0.4, 1.0],
            exterior_color: [1.0, 0.6, 0.2],
        }
    }
}

impl Default for SignedDistanceView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new signed distance view.
pub fn new_signed_distance_view() -> SignedDistanceView {
    SignedDistanceView::new()
}

/// Enable or disable SDF display.
pub fn sdv2_set_enabled(v: &mut SignedDistanceView, enabled: bool) {
    v.enabled = enabled;
}

/// Set the narrow-band width to display.
pub fn sdv2_set_band_width(v: &mut SignedDistanceView, w: f32) {
    v.band_width = w.max(0.001);
}

/// Toggle zero-crossing line highlight.
pub fn sdv2_set_show_zero_crossing(v: &mut SignedDistanceView, show: bool) {
    v.show_zero_crossing = show;
}

/// Set interior color (inside the surface, SDF < 0).
pub fn sdv2_set_interior_color(v: &mut SignedDistanceView, r: f32, g: f32, b: f32) {
    v.interior_color = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
}

/// Set exterior color (outside the surface, SDF > 0).
pub fn sdv2_set_exterior_color(v: &mut SignedDistanceView, r: f32, g: f32, b: f32) {
    v.exterior_color = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
}

/// Blend color for a given signed distance value.
pub fn sdv2_color_for_dist(v: &SignedDistanceView, dist: f32) -> [f32; 3] {
    let t = (dist / v.band_width).clamp(-1.0, 1.0) * 0.5 + 0.5;
    let ic = v.interior_color;
    let ec = v.exterior_color;
    [
        ic[0] * (1.0 - t) + ec[0] * t,
        ic[1] * (1.0 - t) + ec[1] * t,
        ic[2] * (1.0 - t) + ec[2] * t,
    ]
}

/// Serialize to JSON-like string.
pub fn signed_distance_view_to_json(v: &SignedDistanceView) -> String {
    format!(
        r#"{{"enabled":{},"band_width":{:.4},"show_zero_crossing":{}}}"#,
        v.enabled, v.band_width, v.show_zero_crossing
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_signed_distance_view();
        assert!(!v.enabled);
        assert!(v.show_zero_crossing);
    }

    #[test]
    fn test_enable() {
        let mut v = new_signed_distance_view();
        sdv2_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_band_width_clamp() {
        let mut v = new_signed_distance_view();
        sdv2_set_band_width(&mut v, 0.0);
        assert!(v.band_width > 0.0);
    }

    #[test]
    fn test_zero_crossing_toggle() {
        let mut v = new_signed_distance_view();
        sdv2_set_show_zero_crossing(&mut v, false);
        assert!(!v.show_zero_crossing);
    }

    #[test]
    fn test_interior_color_set() {
        let mut v = new_signed_distance_view();
        sdv2_set_interior_color(&mut v, 1.0, 0.0, 0.0);
        assert!((v.interior_color[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_exterior_color_clamp() {
        let mut v = new_signed_distance_view();
        sdv2_set_exterior_color(&mut v, 2.0, -1.0, 0.5);
        assert_eq!(v.exterior_color[0], 1.0);
        assert_eq!(v.exterior_color[1], 0.0);
    }

    #[test]
    fn test_color_at_zero() {
        let v = new_signed_distance_view();
        let c = sdv2_color_for_dist(&v, 0.0); /* midpoint: blend 50/50 */
        assert!((c[0] - (v.interior_color[0] * 0.5 + v.exterior_color[0] * 0.5)).abs() < 1e-5);
    }

    #[test]
    fn test_json_keys() {
        let v = new_signed_distance_view();
        let s = signed_distance_view_to_json(&v);
        assert!(s.contains("band_width"));
    }

    #[test]
    fn test_clone() {
        let v = new_signed_distance_view();
        let v2 = v.clone();
        assert!((v2.band_width - v.band_width).abs() < 1e-6);
    }
}
