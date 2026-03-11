// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! NLA track strip preview view stub.

/// NLA strip blend mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NlaBlendMode {
    Replace,
    Add,
    Subtract,
    Multiply,
}

/// An NLA strip entry.
#[derive(Debug, Clone)]
pub struct NlaStrip {
    pub name: String,
    pub blend_mode: NlaBlendMode,
    pub start_frame: f32,
    pub end_frame: f32,
    pub influence: f32,
}

/// NLA track view configuration.
#[derive(Debug, Clone)]
pub struct NlaTrackView {
    pub strips: Vec<NlaStrip>,
    pub track_height: f32,
    pub show_influence: bool,
    pub enabled: bool,
}

impl NlaTrackView {
    pub fn new() -> Self {
        NlaTrackView {
            strips: Vec::new(),
            track_height: 20.0,
            show_influence: true,
            enabled: true,
        }
    }
}

impl Default for NlaTrackView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new NLA track view.
pub fn new_nla_track_view() -> NlaTrackView {
    NlaTrackView::new()
}

/// Add a strip.
pub fn ntv_add_strip(view: &mut NlaTrackView, strip: NlaStrip) {
    view.strips.push(strip);
}

/// Clear all strips.
pub fn ntv_clear(view: &mut NlaTrackView) {
    view.strips.clear();
}

/// Set track display height in pixels.
pub fn ntv_set_track_height(view: &mut NlaTrackView, height: f32) {
    view.track_height = height.max(8.0);
}

/// Toggle influence curve display.
pub fn ntv_show_influence(view: &mut NlaTrackView, show: bool) {
    view.show_influence = show;
}

/// Enable or disable.
pub fn ntv_set_enabled(view: &mut NlaTrackView, enabled: bool) {
    view.enabled = enabled;
}

/// Return strip count.
pub fn ntv_strip_count(view: &NlaTrackView) -> usize {
    view.strips.len()
}

/// Serialize to JSON-like string.
pub fn ntv_to_json(view: &NlaTrackView) -> String {
    format!(
        r#"{{"strip_count":{},"track_height":{},"show_influence":{},"enabled":{}}}"#,
        view.strips.len(),
        view.track_height,
        view.show_influence,
        view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_strip() -> NlaStrip {
        NlaStrip {
            name: "idle".to_string(),
            blend_mode: NlaBlendMode::Replace,
            start_frame: 0.0,
            end_frame: 60.0,
            influence: 1.0,
        }
    }

    #[test]
    fn test_initial_empty() {
        let v = new_nla_track_view();
        assert_eq!(ntv_strip_count(&v), 0 /* no strips initially */);
    }

    #[test]
    fn test_add_strip() {
        let mut v = new_nla_track_view();
        ntv_add_strip(&mut v, make_strip());
        assert_eq!(ntv_strip_count(&v), 1 /* one strip after add */);
    }

    #[test]
    fn test_clear() {
        let mut v = new_nla_track_view();
        ntv_add_strip(&mut v, make_strip());
        ntv_clear(&mut v);
        assert_eq!(ntv_strip_count(&v), 0 /* cleared */);
    }

    #[test]
    fn test_track_height_min() {
        let mut v = new_nla_track_view();
        ntv_set_track_height(&mut v, 1.0);
        assert!((v.track_height - 8.0).abs() < 1e-6 /* minimum track height must be 8.0 */);
    }

    #[test]
    fn test_set_track_height() {
        let mut v = new_nla_track_view();
        ntv_set_track_height(&mut v, 30.0);
        assert!((v.track_height - 30.0).abs() < 1e-6 /* track height must be set */);
    }

    #[test]
    fn test_show_influence() {
        let mut v = new_nla_track_view();
        ntv_show_influence(&mut v, false);
        assert!(!v.show_influence /* influence must be hidden */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_nla_track_view();
        ntv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_strip_count() {
        let v = new_nla_track_view();
        let j = ntv_to_json(&v);
        assert!(j.contains("\"strip_count\"") /* JSON must have strip_count */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_nla_track_view();
        assert!(v.enabled /* must be enabled by default */);
    }
}
