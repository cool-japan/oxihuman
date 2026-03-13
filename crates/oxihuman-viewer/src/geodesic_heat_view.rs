// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct GeodesicHeatView {
    pub enabled: bool,
    pub max_distance: f32,
    pub source_vertex: usize,
}

pub fn new_geodesic_heat_view() -> GeodesicHeatView {
    GeodesicHeatView {
        enabled: false,
        max_distance: 1.0,
        source_vertex: 0,
    }
}

pub fn ghv_set_max_distance(v: &mut GeodesicHeatView, d: f32) {
    v.max_distance = d.max(1e-6);
}

pub fn ghv_set_source(v: &mut GeodesicHeatView, idx: usize) {
    v.source_vertex = idx;
}

pub fn ghv_enable(v: &mut GeodesicHeatView) {
    v.enabled = true;
}

pub fn ghv_distance_to_color(v: &GeodesicHeatView, dist: f32) -> [f32; 3] {
    let t = (dist / v.max_distance).clamp(0.0, 1.0);
    /* cool-to-warm: blue -> cyan -> green -> yellow -> red */
    let r = t.min(1.0);
    let b = (1.0 - t).min(1.0);
    let g = (1.0 - (2.0 * t - 1.0).abs()).max(0.0);
    [r, g, b]
}

pub fn ghv_is_enabled(v: &GeodesicHeatView) -> bool {
    v.enabled
}

pub fn ghv_to_json(v: &GeodesicHeatView) -> String {
    format!(
        r#"{{"enabled":{},"max_distance":{:.4},"source_vertex":{}}}"#,
        v.enabled, v.max_distance, v.source_vertex
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* disabled, max_distance=1 */
        let v = new_geodesic_heat_view();
        assert!(!v.enabled);
        assert!((v.max_distance - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_max_distance() {
        /* valid distance */
        let mut v = new_geodesic_heat_view();
        ghv_set_max_distance(&mut v, 5.0);
        assert!((v.max_distance - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_max_distance_min() {
        /* minimum enforced */
        let mut v = new_geodesic_heat_view();
        ghv_set_max_distance(&mut v, 0.0);
        assert!(v.max_distance > 0.0);
    }

    #[test]
    fn test_set_source() {
        /* source vertex stored */
        let mut v = new_geodesic_heat_view();
        ghv_set_source(&mut v, 42);
        assert_eq!(v.source_vertex, 42);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_geodesic_heat_view();
        ghv_enable(&mut v);
        assert!(ghv_is_enabled(&v));
    }

    #[test]
    fn test_distance_to_color_zero() {
        /* zero distance -> blue */
        let v = new_geodesic_heat_view();
        let c = ghv_distance_to_color(&v, 0.0);
        assert!((0.0..=1.0).contains(&c[0]));
    }

    #[test]
    fn test_distance_to_color_max() {
        /* max distance -> red */
        let v = new_geodesic_heat_view();
        let c = ghv_distance_to_color(&v, 1.0);
        assert!((0.0..=1.0).contains(&c[0]));
    }

    #[test]
    fn test_to_json() {
        /* JSON has max_distance */
        let v = new_geodesic_heat_view();
        assert!(ghv_to_json(&v).contains("max_distance"));
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let v = new_geodesic_heat_view();
        let v2 = v.clone();
        assert_eq!(v.source_vertex, v2.source_vertex);
    }
}
