// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Hair system settings view.

/// Hair system type.
#[derive(Debug, Clone, PartialEq)]
pub enum HairSystemType {
    Legacy,
    Curves,
}

/// Hair system settings state.
#[derive(Debug, Clone)]
pub struct HairSystemView {
    pub system_type: HairSystemType,
    pub strand_count: u32,
    pub segment_count: u32,
    pub length: f32,
    pub visible: bool,
}

impl Default for HairSystemView {
    fn default() -> Self {
        Self {
            system_type: HairSystemType::Curves,
            strand_count: 1000,
            segment_count: 6,
            length: 0.1,
            visible: true,
        }
    }
}

/// Create a new HairSystemView.
pub fn new_hair_system_view() -> HairSystemView {
    HairSystemView::default()
}

/// Set strand count.
pub fn hair_system_set_strand_count(view: &mut HairSystemView, n: u32) {
    view.strand_count = n.clamp(1, 10_000_000);
}

/// Set segment count per strand.
pub fn hair_system_set_segments(view: &mut HairSystemView, n: u32) {
    view.segment_count = n.clamp(1, 256);
}

/// Set average strand length.
pub fn hair_system_set_length(view: &mut HairSystemView, l: f32) {
    view.length = l.clamp(0.0, 10.0);
}

/// Toggle hair system visibility.
pub fn hair_system_set_visible(view: &mut HairSystemView, v: bool) {
    view.visible = v;
}

/// Compute total control point count.
pub fn hair_system_control_points(view: &HairSystemView) -> u32 {
    view.strand_count * (view.segment_count + 1)
}

/// Serialize to JSON.
pub fn hair_system_to_json(view: &HairSystemView) -> String {
    format!(
        r#"{{"strand_count":{},"segments":{},"length":{},"visible":{}}}"#,
        view.strand_count, view.segment_count, view.length, view.visible,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let v = new_hair_system_view();
        assert_eq!(v.strand_count, 1000 /* default */);
    }

    #[test]
    fn test_strand_count_clamp() {
        let mut v = new_hair_system_view();
        hair_system_set_strand_count(&mut v, 0);
        assert_eq!(v.strand_count, 1 /* min 1 */);
    }

    #[test]
    fn test_segments_clamp() {
        let mut v = new_hair_system_view();
        hair_system_set_segments(&mut v, 1000);
        assert_eq!(v.segment_count, 256 /* max */);
    }

    #[test]
    fn test_length_clamp() {
        let mut v = new_hair_system_view();
        hair_system_set_length(&mut v, 100.0);
        assert!((v.length - 10.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_visible_toggle() {
        let mut v = new_hair_system_view();
        hair_system_set_visible(&mut v, false);
        assert!(!v.visible /* hidden */);
    }

    #[test]
    fn test_control_points() {
        let mut v = new_hair_system_view();
        hair_system_set_strand_count(&mut v, 10);
        hair_system_set_segments(&mut v, 4);
        assert_eq!(hair_system_control_points(&v), 50 /* 10 * (4+1) */);
    }

    #[test]
    fn test_type_default() {
        let v = new_hair_system_view();
        assert_eq!(v.system_type, HairSystemType::Curves /* default */);
    }

    #[test]
    fn test_json_keys() {
        let v = new_hair_system_view();
        let j = hair_system_to_json(&v);
        assert!(j.contains("strand_count") /* key */);
    }

    #[test]
    fn test_clone() {
        let v = new_hair_system_view();
        let c = v.clone();
        assert_eq!(c.strand_count, v.strand_count /* equal */);
    }
}
