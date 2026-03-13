// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Tone-mapped histogram display stub.

/// Histogram tone view config.
#[derive(Debug, Clone)]
pub struct HistogramToneViewConfig {
    pub bin_count: usize,
    pub exposure: f32,
    pub enabled: bool,
    pub show_log_scale: bool,
}

impl Default for HistogramToneViewConfig {
    fn default() -> Self {
        HistogramToneViewConfig {
            bin_count: 256,
            exposure: 1.0,
            enabled: true,
            show_log_scale: false,
        }
    }
}

/// Histogram tone data.
#[derive(Debug, Clone)]
pub struct HistogramTone {
    pub config: HistogramToneViewConfig,
    pub bins: Vec<u32>,
}

impl HistogramTone {
    pub fn new() -> Self {
        let cfg = HistogramToneViewConfig::default();
        let n = cfg.bin_count;
        HistogramTone {
            config: cfg,
            bins: vec![0; n],
        }
    }
}

impl Default for HistogramTone {
    fn default() -> Self {
        HistogramTone::new()
    }
}

/// Create a new histogram tone.
pub fn new_histogram_tone() -> HistogramTone {
    HistogramTone::new()
}

/// Accumulate a luminance value into the histogram.
pub fn htv_accumulate(hist: &mut HistogramTone, luminance: f32) {
    let n = hist.config.bin_count;
    let idx = (luminance * hist.config.exposure * n as f32) as usize;
    let idx = idx.min(n.saturating_sub(1));
    hist.bins[idx] = hist.bins[idx].saturating_add(1);
}

/// Clear the histogram.
pub fn htv_clear(hist: &mut HistogramTone) {
    hist.bins.fill(0);
}

/// Return the peak bin count.
pub fn htv_peak(hist: &HistogramTone) -> u32 {
    hist.bins.iter().copied().max().unwrap_or(0)
}

/// Return total sample count.
pub fn htv_total_samples(hist: &HistogramTone) -> u64 {
    hist.bins.iter().map(|&b| b as u64).sum()
}

/// Return a JSON-like string.
pub fn htv_to_json(hist: &HistogramTone) -> String {
    format!(
        r#"{{"bins":{},"peak":{},"exposure":{:.4},"enabled":{}}}"#,
        hist.config.bin_count,
        htv_peak(hist),
        hist.config.exposure,
        hist.config.enabled
    )
}

/// Toggle log scale.
pub fn htv_toggle_log(hist: &mut HistogramTone) {
    hist.config.show_log_scale = !hist.config.show_log_scale;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bin_count() {
        let h = new_histogram_tone();
        assert_eq!(h.config.bin_count, 256 /* default bin count is 256 */,);
    }

    #[test]
    fn test_initial_peak_zero() {
        let h = new_histogram_tone();
        assert_eq!(htv_peak(&h), 0 /* initial peak is 0 */,);
    }

    #[test]
    fn test_accumulate_increases_bin() {
        let mut h = new_histogram_tone();
        htv_accumulate(&mut h, 0.5);
        assert!(htv_peak(&h) > 0 /* accumulate should increase a bin */,);
    }

    #[test]
    fn test_clear_zeroes_bins() {
        let mut h = new_histogram_tone();
        htv_accumulate(&mut h, 0.5);
        htv_clear(&mut h);
        assert_eq!(htv_peak(&h), 0 /* clear should zero all bins */,);
    }

    #[test]
    fn test_total_samples_after_accumulate() {
        let mut h = new_histogram_tone();
        htv_accumulate(&mut h, 0.2);
        htv_accumulate(&mut h, 0.8);
        assert_eq!(
            htv_total_samples(&h),
            2, /* two samples should be counted */
        );
    }

    #[test]
    fn test_total_samples_initially_zero() {
        let h = new_histogram_tone();
        assert_eq!(htv_total_samples(&h), 0 /* no samples initially */,);
    }

    #[test]
    fn test_toggle_log() {
        let mut h = new_histogram_tone();
        htv_toggle_log(&mut h);
        assert!(h.config.show_log_scale /* log scale toggled on */,);
    }

    #[test]
    fn test_to_json_contains_bins() {
        let h = new_histogram_tone();
        let j = htv_to_json(&h);
        assert!(j.contains("bins") /* JSON must contain bins */,);
    }

    #[test]
    fn test_accumulate_clamps_index() {
        let mut h = new_histogram_tone();
        htv_accumulate(&mut h, 10.0);
        assert_eq!(
            htv_total_samples(&h),
            1, /* extreme value should still accumulate */
        );
    }

    #[test]
    fn test_multiple_accumulate() {
        let mut h = new_histogram_tone();
        for _ in 0..5 {
            htv_accumulate(&mut h, 0.5);
        }
        assert!(htv_peak(&h) >= 5 /* peak should be at least 5 */,);
    }
}
