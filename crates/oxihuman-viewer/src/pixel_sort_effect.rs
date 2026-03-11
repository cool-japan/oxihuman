// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Pixel sort effect — pixel sort glitch effect parameters.

/// Sort direction.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Horizontal,
    Vertical,
    Diagonal,
}

/// Sort key channel.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortKey {
    #[default]
    Luminance,
    Hue,
    Saturation,
    Red,
    Green,
    Blue,
}

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct PixelSortEffectConfig {
    pub direction: SortDirection,
    pub sort_key: SortKey,
    /// Luminance threshold below which pixels are sorted 0..=1.
    pub threshold_low: f32,
    /// Luminance threshold above which pixels are sorted 0..=1.
    pub threshold_high: f32,
    /// Intensity of the effect 0..=1.
    pub intensity: f32,
    /// Seed for row/column selection.
    pub seed: u64,
    pub enabled: bool,
}

impl Default for PixelSortEffectConfig {
    fn default() -> Self {
        Self {
            direction: SortDirection::Horizontal,
            sort_key: SortKey::Luminance,
            threshold_low: 0.2,
            threshold_high: 0.8,
            intensity: 0.7,
            seed: 12345,
            enabled: true,
        }
    }
}

#[allow(dead_code)]
pub fn new_pixel_sort_effect_config() -> PixelSortEffectConfig {
    PixelSortEffectConfig::default()
}

#[allow(dead_code)]
pub fn ps_set_threshold_low(cfg: &mut PixelSortEffectConfig, v: f32) {
    cfg.threshold_low = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ps_set_threshold_high(cfg: &mut PixelSortEffectConfig, v: f32) {
    cfg.threshold_high = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ps_set_intensity(cfg: &mut PixelSortEffectConfig, v: f32) {
    cfg.intensity = v.clamp(0.0, 1.0);
}

/// Effective sort range (high - low).
#[allow(dead_code)]
pub fn ps_sort_range(cfg: &PixelSortEffectConfig) -> f32 {
    (cfg.threshold_high - cfg.threshold_low).max(0.0)
}

/// Returns true if a pixel with given luminance is in the sort range.
#[allow(dead_code)]
pub fn ps_should_sort(luminance: f32, cfg: &PixelSortEffectConfig) -> bool {
    luminance >= cfg.threshold_low && luminance <= cfg.threshold_high
}

/// Sort a row of luminance values in-place (ascending).  Simulates pixel sorting on that row.
#[allow(dead_code)]
pub fn ps_sort_row(row: &mut [f32], cfg: &PixelSortEffectConfig) {
    if !cfg.enabled {
        return;
    }
    // Find sortable segments and sort them
    let n = row.len();
    let mut i = 0;
    while i < n {
        if ps_should_sort(row[i], cfg) {
            let start = i;
            while i < n && ps_should_sort(row[i], cfg) {
                i += 1;
            }
            row[start..i].sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        } else {
            i += 1;
        }
    }
}

#[allow(dead_code)]
pub fn ps_to_json(cfg: &PixelSortEffectConfig) -> String {
    format!(
        "{{\"threshold_low\":{:.3},\"threshold_high\":{:.3},\"intensity\":{:.3},\"enabled\":{}}}",
        cfg.threshold_low, cfg.threshold_high, cfg.intensity, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_sort_in_range() {
        let cfg = new_pixel_sort_effect_config();
        assert!(ps_should_sort(0.5, &cfg));
    }

    #[test]
    fn should_not_sort_below() {
        let cfg = new_pixel_sort_effect_config();
        assert!(!ps_should_sort(0.1, &cfg));
    }

    #[test]
    fn should_not_sort_above() {
        let cfg = new_pixel_sort_effect_config();
        assert!(!ps_should_sort(0.9, &cfg));
    }

    #[test]
    fn sort_range_positive() {
        let cfg = new_pixel_sort_effect_config();
        assert!(ps_sort_range(&cfg) > 0.0);
    }

    #[test]
    fn sort_row_ascending() {
        let cfg = new_pixel_sort_effect_config();
        let mut row = vec![0.7f32, 0.3, 0.5, 0.6, 0.4];
        ps_sort_row(&mut row, &cfg);
        for i in 0..row.len() - 1 {
            assert!(row[i] <= row[i + 1] || !ps_should_sort(row[i], &cfg));
        }
    }

    #[test]
    fn disabled_does_not_sort() {
        let mut cfg = new_pixel_sort_effect_config();
        cfg.enabled = false;
        let mut row = vec![0.7f32, 0.3, 0.5];
        let orig = row.clone();
        ps_sort_row(&mut row, &cfg);
        assert_eq!(row, orig);
    }

    #[test]
    fn threshold_low_clamps() {
        let mut cfg = new_pixel_sort_effect_config();
        ps_set_threshold_low(&mut cfg, -1.0);
        assert!(cfg.threshold_low < 1e-6);
    }

    #[test]
    fn threshold_high_clamps() {
        let mut cfg = new_pixel_sort_effect_config();
        ps_set_threshold_high(&mut cfg, 5.0);
        assert!((cfg.threshold_high - 1.0).abs() < 1e-6);
    }

    #[test]
    fn intensity_clamps() {
        let mut cfg = new_pixel_sort_effect_config();
        ps_set_intensity(&mut cfg, 2.0);
        assert!((cfg.intensity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn json_has_keys() {
        let j = ps_to_json(&new_pixel_sort_effect_config());
        assert!(j.contains("threshold_low") && j.contains("enabled"));
    }
}
