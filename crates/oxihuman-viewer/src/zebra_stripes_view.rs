// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Zebra stripe overexposure overlay for exposure monitoring.

/// Zebra stripes view configuration.
#[derive(Debug, Clone)]
pub struct ZebraStripesView {
    pub lower_threshold: f32,
    pub upper_threshold: f32,
    pub stripe_angle: f32,
    pub stripe_frequency: f32,
    pub stripe_color: [f32; 4],
    pub enabled: bool,
}

impl ZebraStripesView {
    pub fn new() -> Self {
        Self {
            lower_threshold: 0.9,
            upper_threshold: 1.0,
            stripe_angle: 45.0,
            stripe_frequency: 8.0,
            stripe_color: [1.0, 0.8, 0.0, 1.0],
            enabled: false,
        }
    }
}

impl Default for ZebraStripesView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new zebra stripes view.
pub fn new_zebra_stripes_view() -> ZebraStripesView {
    ZebraStripesView::new()
}

/// Set lower luminance threshold for zebra display.
pub fn zsv_set_lower_threshold(view: &mut ZebraStripesView, threshold: f32) {
    view.lower_threshold = threshold.clamp(0.0, 1.0);
}

/// Set upper luminance threshold.
pub fn zsv_set_upper_threshold(view: &mut ZebraStripesView, threshold: f32) {
    view.upper_threshold = threshold.clamp(0.0, 1.0);
}

/// Set zebra stripe frequency (stripes per viewport unit).
pub fn zsv_set_stripe_frequency(view: &mut ZebraStripesView, frequency: f32) {
    view.stripe_frequency = frequency.clamp(1.0, 64.0);
}

/// Toggle zebra overlay.
pub fn zsv_set_enabled(view: &mut ZebraStripesView, enabled: bool) {
    view.enabled = enabled;
}

/// Check if a given luminance value should show zebra stripes.
pub fn zsv_is_overexposed(view: &ZebraStripesView, luminance: f32) -> bool {
    luminance >= view.lower_threshold && luminance <= view.upper_threshold
}

/// Serialize to JSON-like string.
pub fn zebra_stripes_view_to_json(view: &ZebraStripesView) -> String {
    format!(
        r#"{{"lower_threshold":{:.4},"upper_threshold":{:.4},"enabled":{}}}"#,
        view.lower_threshold, view.upper_threshold, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_zebra_stripes_view();
        assert!((v.lower_threshold - 0.9).abs() < 1e-6);
        assert!(!v.enabled);
    }

    #[test]
    fn test_lower_threshold_clamp() {
        let mut v = new_zebra_stripes_view();
        zsv_set_lower_threshold(&mut v, 2.0);
        assert_eq!(v.lower_threshold, 1.0);
    }

    #[test]
    fn test_upper_threshold() {
        let mut v = new_zebra_stripes_view();
        zsv_set_upper_threshold(&mut v, 0.95);
        assert!((v.upper_threshold - 0.95).abs() < 1e-6);
    }

    #[test]
    fn test_stripe_frequency_clamp() {
        let mut v = new_zebra_stripes_view();
        zsv_set_stripe_frequency(&mut v, 0.0);
        assert!((v.stripe_frequency - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_enabled_toggle() {
        let mut v = new_zebra_stripes_view();
        zsv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_overexposed_true() {
        let v = new_zebra_stripes_view();
        assert!(zsv_is_overexposed(&v, 0.95));
    }

    #[test]
    fn test_overexposed_false() {
        let v = new_zebra_stripes_view();
        assert!(!zsv_is_overexposed(&v, 0.5));
    }

    #[test]
    fn test_json() {
        let v = new_zebra_stripes_view();
        let s = zebra_stripes_view_to_json(&v);
        assert!(s.contains("lower_threshold"));
    }

    #[test]
    fn test_clone() {
        let v = new_zebra_stripes_view();
        let v2 = v.clone();
        assert!((v2.stripe_frequency - v.stripe_frequency).abs() < 1e-6);
    }
}
