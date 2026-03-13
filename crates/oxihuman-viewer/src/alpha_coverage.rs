// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Alpha coverage — MSAA alpha-to-coverage configuration for transparent geometry.

/// Alpha-to-coverage mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaCoverageMode {
    Disabled,
    AlphaToCoverage,
    AlphaToOne,
}

/// Configuration for alpha coverage rendering.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AlphaCoverageConfig {
    pub mode: AlphaCoverageMode,
    pub threshold: f32,
    pub sample_count: u32,
}

/// Per-draw alpha coverage statistics.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct AlphaCoverageStats {
    pub transparent_draws: u32,
    pub coverage_pixels: u64,
}

#[allow(dead_code)]
pub fn default_alpha_coverage_config() -> AlphaCoverageConfig {
    AlphaCoverageConfig {
        mode: AlphaCoverageMode::Disabled,
        threshold: 0.5,
        sample_count: 4,
    }
}

#[allow(dead_code)]
pub fn ac_set_mode(cfg: &mut AlphaCoverageConfig, mode: AlphaCoverageMode) {
    cfg.mode = mode;
}

#[allow(dead_code)]
pub fn ac_set_threshold(cfg: &mut AlphaCoverageConfig, t: f32) {
    cfg.threshold = t.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ac_is_enabled(cfg: &AlphaCoverageConfig) -> bool {
    cfg.mode != AlphaCoverageMode::Disabled
}

#[allow(dead_code)]
pub fn ac_record_draw(stats: &mut AlphaCoverageStats, pixels: u64) {
    stats.transparent_draws += 1;
    stats.coverage_pixels += pixels;
}

#[allow(dead_code)]
pub fn ac_clear_stats(stats: &mut AlphaCoverageStats) {
    *stats = AlphaCoverageStats::default();
}

#[allow(dead_code)]
pub fn ac_average_pixels_per_draw(stats: &AlphaCoverageStats) -> f64 {
    if stats.transparent_draws == 0 {
        0.0
    } else {
        stats.coverage_pixels as f64 / stats.transparent_draws as f64
    }
}

#[allow(dead_code)]
pub fn ac_sample_mask(cfg: &AlphaCoverageConfig) -> u32 {
    (1u32 << cfg.sample_count) - 1
}

#[allow(dead_code)]
pub fn ac_to_json(cfg: &AlphaCoverageConfig) -> String {
    let mode_str = match cfg.mode {
        AlphaCoverageMode::Disabled => "disabled",
        AlphaCoverageMode::AlphaToCoverage => "alpha_to_coverage",
        AlphaCoverageMode::AlphaToOne => "alpha_to_one",
    };
    format!(
        r#"{{"mode":"{}","threshold":{:.4},"sample_count":{}}}"#,
        mode_str, cfg.threshold, cfg.sample_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        let cfg = default_alpha_coverage_config();
        assert!(!ac_is_enabled(&cfg));
    }

    #[test]
    fn set_mode_enables() {
        let mut cfg = default_alpha_coverage_config();
        ac_set_mode(&mut cfg, AlphaCoverageMode::AlphaToCoverage);
        assert!(ac_is_enabled(&cfg));
    }

    #[test]
    fn set_threshold_clamps() {
        let mut cfg = default_alpha_coverage_config();
        ac_set_threshold(&mut cfg, 5.0);
        assert!((cfg.threshold - 1.0).abs() < 1e-6);
        ac_set_threshold(&mut cfg, -1.0);
        assert!((cfg.threshold - 0.0).abs() < 1e-6);
    }

    #[test]
    fn record_and_stats() {
        let mut stats = AlphaCoverageStats::default();
        ac_record_draw(&mut stats, 1024);
        ac_record_draw(&mut stats, 2048);
        assert_eq!(stats.transparent_draws, 2);
        assert_eq!(stats.coverage_pixels, 3072);
    }

    #[test]
    fn average_pixels_zero_draws() {
        let stats = AlphaCoverageStats::default();
        assert!((ac_average_pixels_per_draw(&stats) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn average_pixels() {
        let mut stats = AlphaCoverageStats::default();
        ac_record_draw(&mut stats, 1000);
        ac_record_draw(&mut stats, 3000);
        assert!((ac_average_pixels_per_draw(&stats) - 2000.0).abs() < 1e-6);
    }

    #[test]
    fn sample_mask_4x() {
        let cfg = default_alpha_coverage_config();
        assert_eq!(ac_sample_mask(&cfg), 0b1111);
    }

    #[test]
    fn clear_stats() {
        let mut stats = AlphaCoverageStats::default();
        ac_record_draw(&mut stats, 100);
        ac_clear_stats(&mut stats);
        assert_eq!(stats.transparent_draws, 0);
    }

    #[test]
    fn to_json_contains_mode() {
        let cfg = default_alpha_coverage_config();
        let j = ac_to_json(&cfg);
        assert!(j.contains("disabled"));
        assert!(j.contains("threshold"));
    }
}
