// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Temporal filtering for anti-aliasing and noise reduction.

/// Temporal filter configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TemporalFilterConfig {
    pub blend_factor: f32,
    pub motion_rejection: f32,
    pub variance_clamp: f32,
    pub enabled: bool,
    pub history_weight: f32,
}

#[allow(dead_code)]
pub fn default_temporal_filter() -> TemporalFilterConfig {
    TemporalFilterConfig {
        blend_factor: 0.1,
        motion_rejection: 0.5,
        variance_clamp: 1.5,
        enabled: true,
        history_weight: 0.9,
    }
}

#[allow(dead_code)]
pub fn set_blend_factor(cfg: &mut TemporalFilterConfig, value: f32) {
    cfg.blend_factor = value.clamp(0.01, 1.0);
}

#[allow(dead_code)]
pub fn set_motion_rejection(cfg: &mut TemporalFilterConfig, value: f32) {
    cfg.motion_rejection = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_variance_clamp(cfg: &mut TemporalFilterConfig, value: f32) {
    cfg.variance_clamp = value.clamp(0.5, 5.0);
}

#[allow(dead_code)]
pub fn set_history_weight(cfg: &mut TemporalFilterConfig, value: f32) {
    cfg.history_weight = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn enable_temporal_filter(cfg: &mut TemporalFilterConfig) {
    cfg.enabled = true;
}

#[allow(dead_code)]
pub fn disable_temporal_filter(cfg: &mut TemporalFilterConfig) {
    cfg.enabled = false;
}

#[allow(dead_code)]
pub fn temporal_blend(current: f32, history: f32, factor: f32) -> f32 {
    let f = factor.clamp(0.0, 1.0);
    current * f + history * (1.0 - f)
}

#[allow(dead_code)]
pub fn clamp_to_variance(value: f32, mean: f32, variance: f32, clamp_scale: f32) -> f32 {
    let extent = variance * clamp_scale;
    value.clamp(mean - extent, mean + extent)
}

#[allow(dead_code)]
pub fn reject_motion(velocity_sq: f32, threshold: f32) -> f32 {
    if velocity_sq > threshold * threshold { 1.0 } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_temporal_filter();
        assert!(cfg.enabled);
        assert!((cfg.blend_factor - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_set_blend_factor_clamp() {
        let mut cfg = default_temporal_filter();
        set_blend_factor(&mut cfg, 0.0);
        assert!((cfg.blend_factor - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_set_motion_rejection() {
        let mut cfg = default_temporal_filter();
        set_motion_rejection(&mut cfg, 0.8);
        assert!((cfg.motion_rejection - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_variance_clamp() {
        let mut cfg = default_temporal_filter();
        set_variance_clamp(&mut cfg, 2.0);
        assert!((cfg.variance_clamp - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_history_weight() {
        let mut cfg = default_temporal_filter();
        set_history_weight(&mut cfg, 0.5);
        assert!((cfg.history_weight - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_enable_disable() {
        let mut cfg = default_temporal_filter();
        disable_temporal_filter(&mut cfg);
        assert!(!cfg.enabled);
        enable_temporal_filter(&mut cfg);
        assert!(cfg.enabled);
    }

    #[test]
    fn test_temporal_blend() {
        let result = temporal_blend(1.0, 0.0, 0.5);
        assert!((result - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_to_variance() {
        let result = clamp_to_variance(10.0, 5.0, 1.0, 2.0);
        assert!((result - 7.0).abs() < 1e-6);
    }

    #[test]
    fn test_reject_motion_above() {
        let r = reject_motion(4.0, 1.0);
        assert!((r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_reject_motion_below() {
        let r = reject_motion(0.1, 1.0);
        assert!(r.abs() < 1e-6);
    }
}
