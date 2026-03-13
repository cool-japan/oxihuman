// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Alpha threshold — configurable alpha cutoff for transparency masking.

/// Configuration for alpha threshold.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AlphaThresholdConfig {
    pub threshold: f32,
    pub soft_range: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn default_alpha_threshold_config() -> AlphaThresholdConfig {
    AlphaThresholdConfig {
        threshold: 0.5,
        soft_range: 0.1,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn at_set_threshold(cfg: &mut AlphaThresholdConfig, v: f32) {
    cfg.threshold = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn at_set_soft_range(cfg: &mut AlphaThresholdConfig, v: f32) {
    cfg.soft_range = v.clamp(0.0, 0.5);
}

#[allow(dead_code)]
pub fn at_set_enabled(cfg: &mut AlphaThresholdConfig, enabled: bool) {
    cfg.enabled = enabled;
}

#[allow(dead_code)]
pub fn at_is_opaque(cfg: &AlphaThresholdConfig, alpha: f32) -> bool {
    if !cfg.enabled {
        return true;
    }
    alpha >= cfg.threshold
}

#[allow(dead_code)]
pub fn at_soft_alpha(cfg: &AlphaThresholdConfig, alpha: f32) -> f32 {
    if !cfg.enabled {
        return alpha;
    }
    if cfg.soft_range < 1e-6 {
        return if alpha >= cfg.threshold { 1.0 } else { 0.0 };
    }
    let lo = cfg.threshold - cfg.soft_range;
    let hi = cfg.threshold + cfg.soft_range;
    ((alpha - lo) / (hi - lo)).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn at_coverage_estimate(cfg: &AlphaThresholdConfig) -> f32 {
    1.0 - cfg.threshold
}

#[allow(dead_code)]
pub fn at_blend(
    a: &AlphaThresholdConfig,
    b: &AlphaThresholdConfig,
    t: f32,
) -> AlphaThresholdConfig {
    let t = t.clamp(0.0, 1.0);
    AlphaThresholdConfig {
        threshold: a.threshold + (b.threshold - a.threshold) * t,
        soft_range: a.soft_range + (b.soft_range - a.soft_range) * t,
        enabled: a.enabled,
    }
}

#[allow(dead_code)]
pub fn at_to_json(cfg: &AlphaThresholdConfig) -> String {
    format!(
        r#"{{"threshold":{:.4},"soft_range":{:.4},"enabled":{}}}"#,
        cfg.threshold, cfg.soft_range, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_threshold_half() {
        let cfg = default_alpha_threshold_config();
        assert!((cfg.threshold - 0.5).abs() < 1e-6);
    }

    #[test]
    fn set_threshold_clamps() {
        let mut cfg = default_alpha_threshold_config();
        at_set_threshold(&mut cfg, 2.0);
        assert!((cfg.threshold - 1.0).abs() < 1e-6);
    }

    #[test]
    fn is_opaque_above_threshold() {
        let cfg = default_alpha_threshold_config();
        assert!(at_is_opaque(&cfg, 0.6));
    }

    #[test]
    fn is_transparent_below_threshold() {
        let cfg = default_alpha_threshold_config();
        assert!(!at_is_opaque(&cfg, 0.3));
    }

    #[test]
    fn disabled_always_opaque() {
        let mut cfg = default_alpha_threshold_config();
        at_set_enabled(&mut cfg, false);
        assert!(at_is_opaque(&cfg, 0.0));
    }

    #[test]
    fn soft_alpha_midpoint() {
        let mut cfg = default_alpha_threshold_config();
        at_set_threshold(&mut cfg, 0.5);
        at_set_soft_range(&mut cfg, 0.1);
        let a = at_soft_alpha(&cfg, 0.5);
        assert!((a - 0.5).abs() < 0.01);
    }

    #[test]
    fn coverage_estimate() {
        let cfg = default_alpha_threshold_config();
        let cov = at_coverage_estimate(&cfg);
        assert!((0.0..=1.0).contains(&cov));
    }

    #[test]
    fn blend_midpoint() {
        let a = default_alpha_threshold_config();
        let mut b = default_alpha_threshold_config();
        at_set_threshold(&mut b, 1.0);
        let m = at_blend(&a, &b, 0.5);
        assert!((m.threshold - 0.75).abs() < 1e-6);
    }

    #[test]
    fn to_json_fields() {
        let cfg = default_alpha_threshold_config();
        let j = at_to_json(&cfg);
        assert!(j.contains("threshold"));
        assert!(j.contains("enabled"));
    }
}
