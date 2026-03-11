// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Broad-phase AABB debug view — visualises sweep-and-prune or grid cells.

/// Broad phase view configuration.
#[derive(Debug, Clone)]
pub struct BroadPhaseView {
    pub enabled: bool,
    pub color_aabb: [f32; 4],
    pub color_overlap: [f32; 4],
    pub show_pairs: bool,
}

impl BroadPhaseView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            color_aabb: [0.4, 0.8, 0.4, 0.4],
            color_overlap: [1.0, 0.2, 0.2, 0.7],
            show_pairs: true,
        }
    }
}

impl Default for BroadPhaseView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new broad phase view.
pub fn new_broad_phase_view() -> BroadPhaseView {
    BroadPhaseView::new()
}

/// Enable or disable broad-phase overlay.
pub fn bpv_set_enabled(v: &mut BroadPhaseView, enabled: bool) {
    v.enabled = enabled;
}

/// Set AABB box colour.
pub fn bpv_set_color_aabb(v: &mut BroadPhaseView, color: [f32; 4]) {
    v.color_aabb = color;
}

/// Set overlapping-pair colour.
pub fn bpv_set_color_overlap(v: &mut BroadPhaseView, color: [f32; 4]) {
    v.color_overlap = color;
}

/// Toggle candidate pair display.
pub fn bpv_set_show_pairs(v: &mut BroadPhaseView, show: bool) {
    v.show_pairs = show;
}

/// Serialize to JSON-like string.
pub fn broad_phase_view_to_json(v: &BroadPhaseView) -> String {
    format!(
        r#"{{"enabled":{},"show_pairs":{}}}"#,
        v.enabled, v.show_pairs
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_broad_phase_view();
        assert!(!v.enabled);
        assert!(v.show_pairs);
    }

    #[test]
    fn test_enable() {
        let mut v = new_broad_phase_view();
        bpv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_color_aabb() {
        let mut v = new_broad_phase_view();
        bpv_set_color_aabb(&mut v, [1.0, 0.0, 0.0, 1.0]);
        assert!((v.color_aabb[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_color_overlap() {
        let mut v = new_broad_phase_view();
        bpv_set_color_overlap(&mut v, [0.0, 1.0, 0.0, 1.0]);
        assert!((v.color_overlap[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_pairs_toggle() {
        let mut v = new_broad_phase_view();
        bpv_set_show_pairs(&mut v, false);
        assert!(!v.show_pairs);
    }

    #[test]
    fn test_json_keys() {
        let v = new_broad_phase_view();
        let s = broad_phase_view_to_json(&v);
        assert!(s.contains("show_pairs"));
    }

    #[test]
    fn test_clone() {
        let v = new_broad_phase_view();
        let v2 = v.clone();
        assert_eq!(v2.enabled, v.enabled);
    }

    #[test]
    fn test_default_show_pairs_true() {
        let v = BroadPhaseView::default();
        assert!(v.show_pairs);
    }

    #[test]
    fn test_aabb_alpha_component() {
        let v = new_broad_phase_view();
        assert!((v.color_aabb[3] - 0.4).abs() < 1e-5);
    }
}
