// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Temporal AA ghost/jitter visualization stub.

/// Temporal AA debug view config.
#[derive(Debug, Clone)]
pub struct TemporalAaViewConfig {
    pub show_jitter: bool,
    pub show_ghosting: bool,
    pub ghosting_threshold: f32,
    pub jitter_scale: f32,
    pub enabled: bool,
}

impl Default for TemporalAaViewConfig {
    fn default() -> Self {
        TemporalAaViewConfig {
            show_jitter: true,
            show_ghosting: false,
            ghosting_threshold: 0.1,
            jitter_scale: 1.0,
            enabled: true,
        }
    }
}

/// Create a new TAA view config.
pub fn new_temporal_aa_view() -> TemporalAaViewConfig {
    TemporalAaViewConfig::default()
}

/// Set jitter scale.
pub fn tav_set_jitter_scale(cfg: &mut TemporalAaViewConfig, scale: f32) {
    cfg.jitter_scale = scale.max(0.0);
}

/// Set ghosting threshold.
pub fn tav_set_ghosting_threshold(cfg: &mut TemporalAaViewConfig, threshold: f32) {
    cfg.ghosting_threshold = threshold.clamp(0.0, 1.0);
}

/// Toggle jitter display.
pub fn tav_toggle_jitter(cfg: &mut TemporalAaViewConfig) {
    cfg.show_jitter = !cfg.show_jitter;
}

/// Toggle ghosting display.
pub fn tav_toggle_ghosting(cfg: &mut TemporalAaViewConfig) {
    cfg.show_ghosting = !cfg.show_ghosting;
}

/// Enable or disable.
pub fn tav_set_enabled(cfg: &mut TemporalAaViewConfig, enabled: bool) {
    cfg.enabled = enabled;
}

/// Check if a pixel has ghosting given temporal difference.
pub fn tav_is_ghosting(cfg: &TemporalAaViewConfig, temporal_diff: f32) -> bool {
    temporal_diff.abs() > cfg.ghosting_threshold
}

/// Return a JSON-like string.
pub fn tav_to_json(cfg: &TemporalAaViewConfig) -> String {
    format!(
        r#"{{"jitter_scale":{:.4},"ghosting_threshold":{:.4},"enabled":{}}}"#,
        cfg.jitter_scale, cfg.ghosting_threshold, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_show_jitter_true() {
        let c = new_temporal_aa_view();
        assert!(c.show_jitter /* show jitter by default */,);
    }

    #[test]
    fn test_set_jitter_scale() {
        let mut c = new_temporal_aa_view();
        tav_set_jitter_scale(&mut c, 2.0);
        assert!((c.jitter_scale - 2.0).abs() < 1e-5, /* jitter scale must match */);
    }

    #[test]
    fn test_set_jitter_scale_negative_clamps() {
        let mut c = new_temporal_aa_view();
        tav_set_jitter_scale(&mut c, -1.0);
        assert!((c.jitter_scale).abs() < 1e-6, /* negative scale clamped to 0 */);
    }

    #[test]
    fn test_set_ghosting_threshold() {
        let mut c = new_temporal_aa_view();
        tav_set_ghosting_threshold(&mut c, 0.3);
        assert!((c.ghosting_threshold - 0.3).abs() < 1e-5, /* threshold must match */);
    }

    #[test]
    fn test_toggle_jitter() {
        let mut c = new_temporal_aa_view();
        tav_toggle_jitter(&mut c);
        assert!(!c.show_jitter /* jitter should be toggled off */,);
    }

    #[test]
    fn test_toggle_ghosting() {
        let mut c = new_temporal_aa_view();
        tav_toggle_ghosting(&mut c);
        assert!(c.show_ghosting /* ghosting should be toggled on */,);
    }

    #[test]
    fn test_is_ghosting_above_threshold() {
        let c = new_temporal_aa_view();
        assert!(tav_is_ghosting(&c, 0.5), /* 0.5 > 0.1 threshold is ghosting */);
    }

    #[test]
    fn test_is_ghosting_below_threshold() {
        let c = new_temporal_aa_view();
        assert!(!tav_is_ghosting(&c, 0.05), /* 0.05 < 0.1 threshold is not ghosting */);
    }

    #[test]
    fn test_set_enabled_false() {
        let mut c = new_temporal_aa_view();
        tav_set_enabled(&mut c, false);
        assert!(!c.enabled /* should be disabled */,);
    }

    #[test]
    fn test_to_json_contains_jitter() {
        let c = new_temporal_aa_view();
        let j = tav_to_json(&c);
        assert!(j.contains("jitter_scale"), /* JSON must contain jitter_scale */);
    }
}
