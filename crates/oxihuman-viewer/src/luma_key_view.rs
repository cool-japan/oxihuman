// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Luma key matte debug view for compositing workflows.

/// Luma key view configuration.
#[derive(Debug, Clone)]
pub struct LumaKeyView {
    pub key_low: f32,
    pub key_high: f32,
    pub clip_black: f32,
    pub clip_white: f32,
    pub invert: bool,
    pub enabled: bool,
}

impl LumaKeyView {
    pub fn new() -> Self {
        Self {
            key_low: 0.0,
            key_high: 0.5,
            clip_black: 0.0,
            clip_white: 1.0,
            invert: false,
            enabled: false,
        }
    }
}

impl Default for LumaKeyView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new luma key view.
pub fn new_luma_key_view() -> LumaKeyView {
    LumaKeyView::new()
}

/// Set the key range (low and high luminance thresholds).
pub fn lkv_set_key_range(view: &mut LumaKeyView, low: f32, high: f32) {
    view.key_low = low.clamp(0.0, 1.0);
    view.key_high = high.clamp(0.0, 1.0);
}

/// Set clip levels for matte crushing.
pub fn lkv_set_clip_levels(view: &mut LumaKeyView, black: f32, white: f32) {
    view.clip_black = black.clamp(0.0, 1.0);
    view.clip_white = white.clamp(0.0, 1.0);
}

/// Toggle matte inversion.
pub fn lkv_set_invert(view: &mut LumaKeyView, invert: bool) {
    view.invert = invert;
}

/// Toggle luma key overlay.
pub fn lkv_set_enabled(view: &mut LumaKeyView, enabled: bool) {
    view.enabled = enabled;
}

/// Evaluate luma key matte value for a given luminance.
pub fn lkv_evaluate(view: &LumaKeyView, luminance: f32) -> f32 {
    let range = view.key_high - view.key_low;
    let alpha = if range.abs() < 1e-6 {
        if luminance >= view.key_low {
            1.0
        } else {
            0.0
        }
    } else {
        ((luminance - view.key_low) / range).clamp(0.0, 1.0)
    };
    let matte = if view.invert { 1.0 - alpha } else { alpha };
    matte.clamp(view.clip_black, view.clip_white)
}

/// Serialize to JSON-like string.
pub fn luma_key_view_to_json(view: &LumaKeyView) -> String {
    format!(
        r#"{{"key_low":{:.4},"key_high":{:.4},"invert":{},"enabled":{}}}"#,
        view.key_low, view.key_high, view.invert, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_luma_key_view();
        assert_eq!(v.key_low, 0.0);
        assert!(!v.invert);
    }

    #[test]
    fn test_set_key_range() {
        let mut v = new_luma_key_view();
        lkv_set_key_range(&mut v, 0.2, 0.8);
        assert!((v.key_low - 0.2).abs() < 1e-6);
        assert!((v.key_high - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_clip_levels() {
        let mut v = new_luma_key_view();
        lkv_set_clip_levels(&mut v, 0.05, 0.95);
        assert!((v.clip_black - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_invert_toggle() {
        let mut v = new_luma_key_view();
        lkv_set_invert(&mut v, true);
        assert!(v.invert);
    }

    #[test]
    fn test_evaluate_in_range() {
        let mut v = new_luma_key_view();
        lkv_set_key_range(&mut v, 0.0, 1.0);
        let m = lkv_evaluate(&v, 0.5);
        assert!((m - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_inverted() {
        let mut v = new_luma_key_view();
        lkv_set_key_range(&mut v, 0.0, 1.0);
        lkv_set_invert(&mut v, true);
        let m = lkv_evaluate(&v, 1.0);
        assert_eq!(m, 0.0);
    }

    #[test]
    fn test_enabled_toggle() {
        let mut v = new_luma_key_view();
        lkv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_json() {
        let v = new_luma_key_view();
        let s = luma_key_view_to_json(&v);
        assert!(s.contains("key_low"));
    }

    #[test]
    fn test_clone() {
        let v = new_luma_key_view();
        let v2 = v.clone();
        assert!((v2.key_high - v.key_high).abs() < 1e-6);
    }
}
