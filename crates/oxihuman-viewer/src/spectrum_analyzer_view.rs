// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Frequency spectrum view stub.

/// Frequency scale type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FreqScale {
    Linear,
    Logarithmic,
}

/// Spectrum analyzer view configuration.
#[derive(Debug, Clone)]
pub struct SpectrumAnalyzerView {
    pub bands: Vec<f32>,
    pub peak_hold: Vec<f32>,
    pub freq_scale: FreqScale,
    pub decay_rate: f32,
    pub enabled: bool,
}

impl SpectrumAnalyzerView {
    pub fn new(band_count: usize) -> Self {
        SpectrumAnalyzerView {
            bands: vec![0.0; band_count],
            peak_hold: vec![0.0; band_count],
            freq_scale: FreqScale::Logarithmic,
            decay_rate: 0.85,
            enabled: true,
        }
    }
}

/// Create a new spectrum analyzer view.
pub fn new_spectrum_analyzer_view(band_count: usize) -> SpectrumAnalyzerView {
    SpectrumAnalyzerView::new(band_count)
}

/// Feed magnitude values from an FFT frame.
pub fn sav_feed(sav: &mut SpectrumAnalyzerView, magnitudes: &[f32]) {
    /* Stub: copies magnitudes, updates peak hold */
    for (i, (&m, band)) in magnitudes.iter().zip(sav.bands.iter_mut()).enumerate() {
        *band = m.max(0.0);
        if i < sav.peak_hold.len() && m > sav.peak_hold[i] {
            sav.peak_hold[i] = m;
        }
    }
}

/// Tick decay on all bands.
pub fn sav_tick_decay(sav: &mut SpectrumAnalyzerView) {
    /* Stub: multiplies all bands and peaks by decay_rate */
    for v in &mut sav.bands {
        *v *= sav.decay_rate;
    }
    for v in &mut sav.peak_hold {
        *v *= sav.decay_rate;
    }
}

/// Return band count.
pub fn sav_band_count(sav: &SpectrumAnalyzerView) -> usize {
    sav.bands.len()
}

/// Set frequency scale.
pub fn sav_set_freq_scale(sav: &mut SpectrumAnalyzerView, scale: FreqScale) {
    sav.freq_scale = scale;
}

/// Enable or disable.
pub fn sav_set_enabled(sav: &mut SpectrumAnalyzerView, enabled: bool) {
    sav.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn sav_to_json(sav: &SpectrumAnalyzerView) -> String {
    let scale = match sav.freq_scale {
        FreqScale::Linear => "linear",
        FreqScale::Logarithmic => "logarithmic",
    };
    format!(
        r#"{{"band_count":{},"freq_scale":"{}","decay_rate":{},"enabled":{}}}"#,
        sav.bands.len(),
        scale,
        sav.decay_rate,
        sav.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_bands_zero() {
        let s = new_spectrum_analyzer_view(32);
        assert!(s.bands.iter().all(|&v| v.abs() < 1e-6), /* initial bands must be zero */);
    }

    #[test]
    fn test_band_count() {
        let s = new_spectrum_analyzer_view(16);
        assert_eq!(sav_band_count(&s), 16 /* band count must match */,);
    }

    #[test]
    fn test_feed_sets_bands() {
        let mut s = new_spectrum_analyzer_view(4);
        sav_feed(&mut s, &[0.5, 0.3, 0.1, 0.8]);
        assert!((s.bands[0] - 0.5).abs() < 1e-5, /* band[0] must match input */);
    }

    #[test]
    fn test_feed_updates_peak_hold() {
        let mut s = new_spectrum_analyzer_view(4);
        sav_feed(&mut s, &[0.9, 0.0, 0.0, 0.0]);
        assert!((s.peak_hold[0] - 0.9).abs() < 1e-5, /* peak hold must track max */);
    }

    #[test]
    fn test_tick_decay() {
        let mut s = new_spectrum_analyzer_view(4);
        sav_feed(&mut s, &[1.0; 4]);
        sav_tick_decay(&mut s);
        assert!(s.bands[0] < 1.0 /* bands must decay after tick */,);
    }

    #[test]
    fn test_set_freq_scale() {
        let mut s = new_spectrum_analyzer_view(8);
        sav_set_freq_scale(&mut s, FreqScale::Linear);
        assert_eq!(
            s.freq_scale,
            FreqScale::Linear, /* freq scale must be set */
        );
    }

    #[test]
    fn test_set_enabled() {
        let mut s = new_spectrum_analyzer_view(8);
        sav_set_enabled(&mut s, false);
        assert!(!s.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_band_count() {
        let s = new_spectrum_analyzer_view(8);
        let j = sav_to_json(&s);
        assert!(j.contains("\"band_count\""), /* json must contain band_count */);
    }

    #[test]
    fn test_enabled_default() {
        let s = new_spectrum_analyzer_view(8);
        assert!(s.enabled /* must be enabled by default */,);
    }

    #[test]
    fn test_default_log_scale() {
        let s = new_spectrum_analyzer_view(8);
        assert_eq!(
            s.freq_scale,
            FreqScale::Logarithmic, /* default scale must be logarithmic */
        );
    }
}
