// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Aspect ratio mask overlay for viewport composition.

/// Aspect ratio view configuration.
#[derive(Debug, Clone)]
pub struct AspectRatioView {
    pub ratio_w: f32,
    pub ratio_h: f32,
    pub mask_alpha: f32,
    pub mask_color: [f32; 4],
    pub enabled: bool,
}

impl AspectRatioView {
    pub fn new() -> Self {
        Self {
            ratio_w: 16.0,
            ratio_h: 9.0,
            mask_alpha: 0.5,
            mask_color: [0.0, 0.0, 0.0, 1.0],
            enabled: false,
        }
    }
}

impl Default for AspectRatioView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new aspect ratio view.
pub fn new_aspect_ratio_view() -> AspectRatioView {
    AspectRatioView::new()
}

/// Set aspect ratio as width:height values.
pub fn arv_set_ratio(view: &mut AspectRatioView, width: f32, height: f32) {
    view.ratio_w = width.max(0.01);
    view.ratio_h = height.max(0.01);
}

/// Set mask transparency (0 = transparent, 1 = opaque).
pub fn arv_set_mask_alpha(view: &mut AspectRatioView, alpha: f32) {
    view.mask_alpha = alpha.clamp(0.0, 1.0);
}

/// Enable or disable the mask overlay.
pub fn arv_set_enabled(view: &mut AspectRatioView, enabled: bool) {
    view.enabled = enabled;
}

/// Compute the numeric aspect ratio value.
pub fn arv_aspect_value(view: &AspectRatioView) -> f32 {
    if view.ratio_h.abs() < 1e-6 {
        1.0
    } else {
        view.ratio_w / view.ratio_h
    }
}

/// Serialize to JSON-like string.
pub fn aspect_ratio_view_to_json(view: &AspectRatioView) -> String {
    format!(
        r#"{{"ratio_w":{:.4},"ratio_h":{:.4},"mask_alpha":{:.4},"enabled":{}}}"#,
        view.ratio_w, view.ratio_h, view.mask_alpha, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_aspect_ratio_view();
        assert!((v.ratio_w - 16.0).abs() < 1e-6);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_ratio() {
        let mut v = new_aspect_ratio_view();
        arv_set_ratio(&mut v, 4.0, 3.0);
        assert!((arv_aspect_value(&v) - 4.0 / 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_mask_alpha_clamp() {
        let mut v = new_aspect_ratio_view();
        arv_set_mask_alpha(&mut v, 2.0);
        assert_eq!(v.mask_alpha, 1.0);
    }

    #[test]
    fn test_enabled_toggle() {
        let mut v = new_aspect_ratio_view();
        arv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_aspect_value_16_9() {
        let v = new_aspect_ratio_view();
        let ratio = arv_aspect_value(&v);
        assert!((ratio - 16.0 / 9.0).abs() < 1e-5);
    }

    #[test]
    fn test_json() {
        let v = new_aspect_ratio_view();
        let s = aspect_ratio_view_to_json(&v);
        assert!(s.contains("ratio_w"));
        assert!(s.contains("enabled"));
    }

    #[test]
    fn test_clone() {
        let v = new_aspect_ratio_view();
        let v2 = v.clone();
        assert!((v2.ratio_w - v.ratio_w).abs() < 1e-6);
    }

    #[test]
    fn test_mask_alpha_zero() {
        let mut v = new_aspect_ratio_view();
        arv_set_mask_alpha(&mut v, 0.0);
        assert_eq!(v.mask_alpha, 0.0);
    }

    #[test]
    fn test_default_trait() {
        let v: AspectRatioView = Default::default();
        assert!((v.ratio_h - 9.0).abs() < 1e-6);
    }
}
