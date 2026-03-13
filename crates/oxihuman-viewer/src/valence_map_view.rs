// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Vertex valence heat-map visualization.
#[derive(Debug, Clone)]
pub struct ValenceMapView {
    pub enabled: bool,
    /// Regular valence target (typically 4 for quad mesh).
    pub target_valence: u32,
    /// Heat-map opacity (0.0 … 1.0).
    pub opacity: f32,
}

pub fn new_valence_map_view() -> ValenceMapView {
    ValenceMapView {
        enabled: false,
        target_valence: 4,
        opacity: 0.7,
    }
}

pub fn vmv_enable(v: &mut ValenceMapView) {
    v.enabled = true;
}

pub fn vmv_set_opacity(v: &mut ValenceMapView, o: f32) {
    v.opacity = o.clamp(0.0, 1.0);
}

pub fn vmv_set_target_valence(v: &mut ValenceMapView, t: u32) {
    v.target_valence = t.max(1);
}

/// Returns a heat-map colour for the valence deviation from target.
pub fn vmv_valence_color(v: &ValenceMapView, valence: u32) -> [f32; 3] {
    let dev = (valence as i32 - v.target_valence as i32).unsigned_abs();
    let t = (dev as f32 / 3.0).clamp(0.0, 1.0);
    [t, 1.0 - t, 0.0]
}

pub fn vmv_is_regular(v: &ValenceMapView, valence: u32) -> bool {
    valence == v.target_valence
}

pub fn vmv_to_json(v: &ValenceMapView) -> String {
    format!(
        r#"{{"enabled":{},"target_valence":{},"opacity":{:.4}}}"#,
        v.enabled, v.target_valence, v.opacity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* target 4, opacity 0.7 */
        let v = new_valence_map_view();
        assert_eq!(v.target_valence, 4);
        assert!((v.opacity - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_enable() {
        /* enable works */
        let mut v = new_valence_map_view();
        vmv_enable(&mut v);
        assert!(v.enabled);
    }

    #[test]
    fn test_set_opacity() {
        /* valid opacity */
        let mut v = new_valence_map_view();
        vmv_set_opacity(&mut v, 0.5);
        assert!((v.opacity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_opacity_clamp() {
        /* clamped above 1 */
        let mut v = new_valence_map_view();
        vmv_set_opacity(&mut v, 2.0);
        assert_eq!(v.opacity, 1.0);
    }

    #[test]
    fn test_is_regular_true() {
        /* valence 4 is regular */
        let v = new_valence_map_view();
        assert!(vmv_is_regular(&v, 4));
    }

    #[test]
    fn test_is_regular_false() {
        /* valence 3 is irregular */
        let v = new_valence_map_view();
        assert!(!vmv_is_regular(&v, 3));
    }

    #[test]
    fn test_color_regular_green() {
        /* regular valence -> green component dominant */
        let v = new_valence_map_view();
        let c = vmv_valence_color(&v, 4);
        assert!(c[1] > c[0]);
    }

    #[test]
    fn test_to_json() {
        /* JSON has opacity */
        assert!(vmv_to_json(&new_valence_map_view()).contains("opacity"));
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let v = new_valence_map_view();
        let v2 = v.clone();
        assert_eq!(v.target_valence, v2.target_valence);
    }
}
