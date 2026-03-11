// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Draw call / batch count heat map overlay.

/// Configuration for batch count view.
#[derive(Debug, Clone)]
pub struct BatchCountViewConfig {
    pub low_threshold: u32,
    pub high_threshold: u32,
    pub opacity: f32,
}

impl Default for BatchCountViewConfig {
    fn default() -> Self {
        Self { low_threshold: 4, high_threshold: 32, opacity: 0.6 }
    }
}

/// State for batch count visualization.
#[derive(Debug, Clone)]
pub struct BatchCountView {
    pub config: BatchCountViewConfig,
    pub enabled: bool,
}

impl Default for BatchCountView {
    fn default() -> Self {
        Self { config: BatchCountViewConfig::default(), enabled: false }
    }
}

/// Enable the batch count overlay.
pub fn bcv_enable(view: &mut BatchCountView) {
    view.enabled = true;
}

/// Disable the batch count overlay.
pub fn bcv_disable(view: &mut BatchCountView) {
    view.enabled = false;
}

/// Set the thresholds for low/high batch counts.
pub fn bcv_set_thresholds(view: &mut BatchCountView, low: u32, high: u32) {
    view.config.low_threshold = low;
    view.config.high_threshold = high.max(low + 1);
}

/// Map a batch count to a heat colour RGBA.
pub fn bcv_count_to_color(count: u32, config: &BatchCountViewConfig) -> [f32; 4] {
    let lo = config.low_threshold as f32;
    let hi = config.high_threshold as f32;
    let t = ((count as f32 - lo) / (hi - lo)).clamp(0.0, 1.0);
    /* green → yellow → red gradient */
    let r = t.min(0.5) * 2.0;
    let g = (1.0 - t).min(0.5) * 2.0;
    [r, g, 0.0, config.opacity]
}

/// Return whether a batch count exceeds the high threshold.
pub fn bcv_is_hot(count: u32, config: &BatchCountViewConfig) -> bool {
    count >= config.high_threshold
}

/// Return the batch efficiency ratio (lower is more efficient).
pub fn bcv_efficiency(count: u32, config: &BatchCountViewConfig) -> f32 {
    let lo = config.low_threshold.max(1) as f32;
    (count as f32 / lo).min(1.0)
}

/// Export config to JSON string (stub).
pub fn bcv_to_json(view: &BatchCountView) -> String {
    format!(
        r#"{{"low_threshold":{},"high_threshold":{},"enabled":{}}}"#,
        view.config.low_threshold, view.config.high_threshold, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled() {
        /* default should be disabled */
        let v = BatchCountView::default();
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable_disable() {
        /* enable/disable should toggle */
        let mut v = BatchCountView::default();
        bcv_enable(&mut v);
        assert!(v.enabled);
        bcv_disable(&mut v);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_thresholds() {
        /* thresholds should be updated */
        let mut v = BatchCountView::default();
        bcv_set_thresholds(&mut v, 2, 20);
        assert_eq!(v.config.low_threshold, 2);
        assert_eq!(v.config.high_threshold, 20);
    }

    #[test]
    fn test_high_threshold_minimum() {
        /* high must be at least low + 1 */
        let mut v = BatchCountView::default();
        bcv_set_thresholds(&mut v, 10, 5);
        assert!(v.config.high_threshold > v.config.low_threshold);
    }

    #[test]
    fn test_count_to_color_low() {
        /* low count should produce green-ish color */
        let cfg = BatchCountViewConfig::default();
        let c = bcv_count_to_color(cfg.low_threshold, &cfg);
        assert!(c[1] >= c[0]);
    }

    #[test]
    fn test_count_to_color_high() {
        /* high count should produce red-ish color */
        let cfg = BatchCountViewConfig::default();
        let c = bcv_count_to_color(cfg.high_threshold, &cfg);
        assert!(c[0] >= c[1]);
    }

    #[test]
    fn test_is_hot() {
        /* count at or above high threshold should be hot */
        let cfg = BatchCountViewConfig::default();
        assert!(bcv_is_hot(cfg.high_threshold, &cfg));
    }

    #[test]
    fn test_is_not_hot() {
        /* count below high threshold should not be hot */
        let cfg = BatchCountViewConfig::default();
        assert!(!bcv_is_hot(cfg.low_threshold, &cfg));
    }

    #[test]
    fn test_to_json_contains_thresholds() {
        /* JSON should contain threshold values */
        let v = BatchCountView::default();
        let json = bcv_to_json(&v);
        assert!(json.contains("low_threshold"));
    }
}
