// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Radial / pie menu overlay view.

use std::f32::consts::TAU;

/// A single slice of the pie menu.
#[derive(Debug, Clone)]
pub struct PieSlice {
    pub id: u32,
    pub label: String,
    pub angle_deg: f32,
    pub enabled: bool,
}

/// State for the pie menu view.
#[derive(Debug, Clone)]
pub struct PieMenuView {
    pub slices: Vec<PieSlice>,
    pub center: [f32; 2],
    pub radius: f32,
    pub visible: bool,
    pub enabled: bool,
}

/// Create a new pie menu view.
pub fn new_pie_menu_view() -> PieMenuView {
    PieMenuView {
        slices: Vec::new(),
        center: [0.0, 0.0],
        radius: 80.0,
        visible: false,
        enabled: true,
    }
}

/// Add a pie slice at the given angle (degrees, 0 = right).
pub fn pmv_add_slice(v: &mut PieMenuView, id: u32, label: &str, angle_deg: f32) {
    v.slices.push(PieSlice {
        id,
        label: label.to_string(),
        angle_deg,
        enabled: true,
    });
}

/// Show the pie menu at a screen position.
pub fn pmv_show(v: &mut PieMenuView, x: f32, y: f32) {
    v.center = [x, y];
    v.visible = true;
}

/// Hide the pie menu.
pub fn pmv_hide(v: &mut PieMenuView) {
    v.visible = false;
}

/// Find the closest slice to a given angle in degrees.
pub fn pmv_slice_at_angle(v: &PieMenuView, angle_deg: f32) -> Option<u32> {
    v.slices
        .iter()
        .filter(|s| s.enabled)
        .min_by(|a, b| {
            let da = angle_diff(a.angle_deg, angle_deg);
            let db = angle_diff(b.angle_deg, angle_deg);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|s| s.id)
}

fn angle_diff(a: f32, b: f32) -> f32 {
    ((a - b).rem_euclid(360.0)).min((b - a).rem_euclid(360.0))
}

/// Return the world position of a slice centre.
pub fn pmv_slice_position(v: &PieMenuView, angle_deg: f32) -> [f32; 2] {
    let rad = angle_deg * TAU / 360.0;
    [
        v.center[0] + v.radius * rad.cos(),
        v.center[1] + v.radius * rad.sin(),
    ]
}

/// Serialise to JSON.
pub fn pmv_to_json(v: &PieMenuView) -> String {
    format!(
        r#"{{"slice_count":{},"visible":{},"radius":{:.1},"enabled":{}}}"#,
        v.slices.len(),
        v.visible,
        v.radius,
        v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_hidden() {
        let v = new_pie_menu_view();
        assert!(!v.visible /* hidden by default */);
    }

    #[test]
    fn show_sets_visible() {
        let mut v = new_pie_menu_view();
        pmv_show(&mut v, 100.0, 200.0);
        assert!(v.visible /* visible after show */);
        assert!((v.center[0] - 100.0).abs() < 1e-6 /* center x */);
    }

    #[test]
    fn hide_clears_visible() {
        let mut v = new_pie_menu_view();
        pmv_show(&mut v, 0.0, 0.0);
        pmv_hide(&mut v);
        assert!(!v.visible /* hidden after hide */);
    }

    #[test]
    fn add_slice() {
        let mut v = new_pie_menu_view();
        pmv_add_slice(&mut v, 1, "Select", 0.0);
        assert_eq!(v.slices.len(), 1 /* one slice */);
    }

    #[test]
    fn slice_at_angle_finds_closest() {
        let mut v = new_pie_menu_view();
        pmv_add_slice(&mut v, 1, "Right", 0.0);
        pmv_add_slice(&mut v, 2, "Top", 90.0);
        let id = pmv_slice_at_angle(&v, 10.0);
        assert_eq!(id, Some(1) /* closest to 0 degrees */);
    }

    #[test]
    fn slice_position_correct_at_zero() {
        let mut v = new_pie_menu_view();
        v.center = [0.0, 0.0];
        v.radius = 100.0;
        let pos = pmv_slice_position(&v, 0.0);
        assert!((pos[0] - 100.0).abs() < 1e-4 /* at right */);
    }

    #[test]
    fn json_has_slice_count() {
        let v = new_pie_menu_view();
        assert!(pmv_to_json(&v).contains("slice_count") /* json field */);
    }

    #[test]
    fn enabled_default() {
        let v = new_pie_menu_view();
        assert!(v.enabled /* enabled */);
    }
}
