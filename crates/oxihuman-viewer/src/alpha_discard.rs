// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Alpha discard — threshold-based alpha clipping for transparent surfaces.

use std::f32::consts::FRAC_PI_4;

/// Mode for alpha discard.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaDiscardMode {
    /// Hard threshold: pixels with alpha < threshold are discarded.
    Hard,
    /// Dithered threshold using a screen-space pattern.
    Dithered,
    /// No discard — pass all pixels.
    None,
}

/// Configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AlphaDiscardConfig {
    /// Alpha threshold in `[0.0, 1.0]`.
    pub threshold: f32,
    /// Soft range around threshold (for anti-aliased edges).
    pub soft_range: f32,
    pub mode: AlphaDiscardMode,
    /// Pattern scale for dithered mode (informational).
    pub dither_scale: f32,
}

impl Default for AlphaDiscardConfig {
    fn default() -> Self {
        AlphaDiscardConfig {
            threshold: 0.5,
            soft_range: 0.05,
            mode: AlphaDiscardMode::Hard,
            dither_scale: FRAC_PI_4,
        }
    }
}

/// Return the default config.
pub fn default_alpha_discard_config() -> AlphaDiscardConfig {
    AlphaDiscardConfig::default()
}

/// Create a new config with a specific threshold.
pub fn new_alpha_discard(threshold: f32, mode: AlphaDiscardMode) -> AlphaDiscardConfig {
    AlphaDiscardConfig {
        threshold: threshold.clamp(0.0, 1.0),
        mode,
        ..AlphaDiscardConfig::default()
    }
}

/// Set the discard threshold.
pub fn ad_set_threshold(cfg: &mut AlphaDiscardConfig, v: f32) {
    cfg.threshold = v.clamp(0.0, 1.0);
}

/// Set the soft range.
pub fn ad_set_soft_range(cfg: &mut AlphaDiscardConfig, v: f32) {
    cfg.soft_range = v.clamp(0.0, 0.5);
}

/// Set the mode.
pub fn ad_set_mode(cfg: &mut AlphaDiscardConfig, mode: AlphaDiscardMode) {
    cfg.mode = mode;
}

/// Return the mode name.
pub fn ad_mode_name(cfg: &AlphaDiscardConfig) -> &'static str {
    match cfg.mode {
        AlphaDiscardMode::Hard => "hard",
        AlphaDiscardMode::Dithered => "dithered",
        AlphaDiscardMode::None => "none",
    }
}

/// Evaluate whether a pixel with the given alpha should be discarded.
///
/// Returns `true` if the pixel is **kept** (not discarded).
pub fn ad_should_keep(cfg: &AlphaDiscardConfig, alpha: f32) -> bool {
    match cfg.mode {
        AlphaDiscardMode::None => true,
        AlphaDiscardMode::Hard => alpha >= cfg.threshold,
        AlphaDiscardMode::Dithered => {
            // simple smooth-step approximation
            let lo = cfg.threshold - cfg.soft_range;
            let hi = cfg.threshold + cfg.soft_range;
            if alpha >= hi {
                return true;
            }
            if alpha < lo {
                return false;
            }
            let t = (alpha - lo) / (hi - lo);
            t * t * (3.0 - 2.0 * t) >= 0.5
        }
    }
}

/// Estimate the fraction of pixels kept for a uniform alpha distribution.
///
/// Integrates `ad_should_keep` over a grid of 10 alpha values in `[0.0, 1.0]`.
pub fn ad_coverage_estimate(cfg: &AlphaDiscardConfig) -> f32 {
    let n = 10usize;
    let mut kept = 0usize;
    #[allow(clippy::needless_range_loop)]
    for i in 0..n {
        let alpha = i as f32 / (n - 1) as f32;
        if ad_should_keep(cfg, alpha) {
            kept += 1;
        }
    }
    kept as f32 / n as f32
}

/// Serialise to JSON-like string.
pub fn ad_to_json(cfg: &AlphaDiscardConfig) -> String {
    format!(
        r#"{{"threshold":{:.4},"soft_range":{:.4},"mode":"{}"}}"#,
        cfg.threshold,
        cfg.soft_range,
        ad_mode_name(cfg)
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_threshold_half() {
        let cfg = default_alpha_discard_config();
        assert!((cfg.threshold - 0.5).abs() < 1e-5);
    }

    #[test]
    fn hard_keep_above_threshold() {
        let cfg = new_alpha_discard(0.5, AlphaDiscardMode::Hard);
        assert!(ad_should_keep(&cfg, 0.6));
    }

    #[test]
    fn hard_discard_below_threshold() {
        let cfg = new_alpha_discard(0.5, AlphaDiscardMode::Hard);
        assert!(!ad_should_keep(&cfg, 0.4));
    }

    #[test]
    fn none_mode_always_keeps() {
        let cfg = new_alpha_discard(0.5, AlphaDiscardMode::None);
        assert!(ad_should_keep(&cfg, 0.0));
    }

    #[test]
    fn set_threshold_clamps() {
        let mut cfg = default_alpha_discard_config();
        ad_set_threshold(&mut cfg, 5.0);
        assert!((cfg.threshold - 1.0).abs() < 1e-5);
    }

    #[test]
    fn mode_name_correct() {
        let cfg = new_alpha_discard(0.5, AlphaDiscardMode::Dithered);
        assert_eq!(ad_mode_name(&cfg), "dithered");
    }

    #[test]
    fn coverage_in_range() {
        let cfg = default_alpha_discard_config();
        assert!((0.0..=1.0).contains(&ad_coverage_estimate(&cfg)));
    }

    #[test]
    fn json_has_threshold() {
        assert!(ad_to_json(&default_alpha_discard_config()).contains("threshold"));
    }

    #[test]
    fn dithered_keeps_full_alpha() {
        let cfg = new_alpha_discard(0.5, AlphaDiscardMode::Dithered);
        assert!(ad_should_keep(&cfg, 1.0));
    }
}
