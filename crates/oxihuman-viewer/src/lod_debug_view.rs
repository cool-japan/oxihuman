// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! LOD level debug overlay — visualizes which LOD level is active per object.

/// LOD debug view configuration.
#[derive(Debug, Clone)]
pub struct LodDebugView {
    pub enabled: bool,
    pub show_labels: bool,
    pub lod_count: u32,
    pub color_bands: bool,
    pub distance_threshold: f32,
}

impl LodDebugView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            show_labels: true,
            lod_count: 4,
            color_bands: true,
            distance_threshold: 100.0,
        }
    }
}

impl Default for LodDebugView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new LOD debug view.
pub fn new_lod_debug_view() -> LodDebugView {
    LodDebugView::new()
}

/// Enable or disable the LOD debug overlay.
pub fn lddv_set_enabled(v: &mut LodDebugView, enabled: bool) {
    v.enabled = enabled;
}

/// Set whether LOD level labels are shown on objects.
pub fn lddv_set_show_labels(v: &mut LodDebugView, show: bool) {
    v.show_labels = show;
}

/// Set number of LOD levels to visualize.
pub fn lddv_set_lod_count(v: &mut LodDebugView, count: u32) {
    v.lod_count = count.clamp(1, 8);
}

/// Set the distance threshold for LOD transition visibility.
pub fn lddv_set_distance_threshold(v: &mut LodDebugView, dist: f32) {
    v.distance_threshold = dist.clamp(1.0, 10_000.0);
}

/// Compute per-LOD color intensity factor for a given level.
pub fn lddv_color_intensity(v: &LodDebugView, level: u32) -> f32 {
    if v.lod_count == 0 {
        return 0.0;
    }
    let max = v.lod_count.saturating_sub(1).max(1);
    1.0 - (level.min(max) as f32 / max as f32) * 0.7
}

/// Serialize to JSON-like string.
pub fn lod_debug_view_to_json(v: &LodDebugView) -> String {
    format!(
        r#"{{"enabled":{},"show_labels":{},"lod_count":{},"color_bands":{},"distance_threshold":{:.2}}}"#,
        v.enabled, v.show_labels, v.lod_count, v.color_bands, v.distance_threshold
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_lod_debug_view();
        assert!(!v.enabled);
        assert_eq!(v.lod_count, 4);
    }

    #[test]
    fn test_enable() {
        let mut v = new_lod_debug_view();
        lddv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_show_labels_false() {
        let mut v = new_lod_debug_view();
        lddv_set_show_labels(&mut v, false);
        assert!(!v.show_labels);
    }

    #[test]
    fn test_lod_count_clamp_high() {
        let mut v = new_lod_debug_view();
        lddv_set_lod_count(&mut v, 100);
        assert_eq!(v.lod_count, 8);
    }

    #[test]
    fn test_lod_count_clamp_low() {
        let mut v = new_lod_debug_view();
        lddv_set_lod_count(&mut v, 0);
        assert_eq!(v.lod_count, 1);
    }

    #[test]
    fn test_distance_threshold_clamp() {
        let mut v = new_lod_debug_view();
        lddv_set_distance_threshold(&mut v, 0.0);
        assert!((v.distance_threshold - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_color_intensity_level_zero() {
        let v = new_lod_debug_view();
        let ci = lddv_color_intensity(&v, 0);
        assert!((ci - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_color_intensity_decreases() {
        let v = new_lod_debug_view();
        let ci0 = lddv_color_intensity(&v, 0);
        let ci3 = lddv_color_intensity(&v, 3);
        assert!(ci0 > ci3);
    }

    #[test]
    fn test_json_keys() {
        let v = new_lod_debug_view();
        let s = lod_debug_view_to_json(&v);
        assert!(s.contains("distance_threshold"));
    }

    #[test]
    fn test_clone() {
        let v = new_lod_debug_view();
        let v2 = v.clone();
        assert_eq!(v2.lod_count, v.lod_count);
    }
}
