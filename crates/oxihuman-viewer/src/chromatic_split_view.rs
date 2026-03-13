// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct ChromaticSplitView {
    pub red_offset: f32,
    pub green_offset: f32,
    pub blue_offset: f32,
    pub enabled: bool,
}

pub fn new_chromatic_split_view() -> ChromaticSplitView {
    ChromaticSplitView {
        red_offset: 0.0,
        green_offset: 0.0,
        blue_offset: 0.0,
        enabled: false,
    }
}

pub fn csv_set_offset(v: &mut ChromaticSplitView, r: f32, g: f32, b: f32) {
    v.red_offset = r.clamp(0.0, 1.0);
    v.green_offset = g.clamp(0.0, 1.0);
    v.blue_offset = b.clamp(0.0, 1.0);
}

pub fn csv_enable(v: &mut ChromaticSplitView) {
    v.enabled = true;
}

pub fn csv_disable(v: &mut ChromaticSplitView) {
    v.enabled = false;
}

pub fn csv_is_enabled(v: &ChromaticSplitView) -> bool {
    v.enabled
}

pub fn csv_blend(a: &ChromaticSplitView, b: &ChromaticSplitView, t: f32) -> ChromaticSplitView {
    let t = t.clamp(0.0, 1.0);
    ChromaticSplitView {
        red_offset: a.red_offset + (b.red_offset - a.red_offset) * t,
        green_offset: a.green_offset + (b.green_offset - a.green_offset) * t,
        blue_offset: a.blue_offset + (b.blue_offset - a.blue_offset) * t,
        enabled: a.enabled,
    }
}

pub fn csv_to_json(v: &ChromaticSplitView) -> String {
    format!(
        r#"{{"red_offset":{:.4},"green_offset":{:.4},"blue_offset":{:.4},"enabled":{}}}"#,
        v.red_offset, v.green_offset, v.blue_offset, v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_is_disabled() {
        /* newly created view is disabled */
        let v = new_chromatic_split_view();
        assert!(!csv_is_enabled(&v));
    }

    #[test]
    fn test_new_offsets_zero() {
        /* offsets start at zero */
        let v = new_chromatic_split_view();
        assert_eq!(v.red_offset, 0.0);
        assert_eq!(v.green_offset, 0.0);
        assert_eq!(v.blue_offset, 0.0);
    }

    #[test]
    fn test_set_offset_clamps_high() {
        /* values above 1 are clamped */
        let mut v = new_chromatic_split_view();
        csv_set_offset(&mut v, 3.0, 4.0, 5.0);
        assert_eq!(v.red_offset, 1.0);
        assert_eq!(v.green_offset, 1.0);
        assert_eq!(v.blue_offset, 1.0);
    }

    #[test]
    fn test_set_offset_clamps_low() {
        /* values below 0 are clamped */
        let mut v = new_chromatic_split_view();
        csv_set_offset(&mut v, -1.0, -2.0, -0.5);
        assert_eq!(v.red_offset, 0.0);
    }

    #[test]
    fn test_enable_disable() {
        /* enable then disable */
        let mut v = new_chromatic_split_view();
        csv_enable(&mut v);
        assert!(csv_is_enabled(&v));
        csv_disable(&mut v);
        assert!(!csv_is_enabled(&v));
    }

    #[test]
    fn test_blend_midpoint() {
        /* t=0.5 averages offsets */
        let mut a = new_chromatic_split_view();
        let mut b = new_chromatic_split_view();
        csv_set_offset(&mut a, 0.0, 0.0, 0.0);
        csv_set_offset(&mut b, 1.0, 1.0, 1.0);
        let c = csv_blend(&a, &b, 0.5);
        assert!((c.red_offset - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_json_contains_enabled() {
        /* JSON output includes enabled field */
        let v = new_chromatic_split_view();
        let s = csv_to_json(&v);
        assert!(s.contains("enabled"));
    }

    #[test]
    fn test_to_json_disabled_false() {
        /* disabled shows false in JSON */
        let v = new_chromatic_split_view();
        let s = csv_to_json(&v);
        assert!(s.contains("false"));
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let v = new_chromatic_split_view();
        let v2 = v.clone();
        assert_eq!(v.enabled, v2.enabled);
    }
}
