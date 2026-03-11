// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! FFT-based frequency analysis via direct DFT computation.

use std::f32::consts::TAU;

/// Result of a DFT analysis: frequency bin (Hz) and magnitude.
#[derive(Debug, Clone, PartialEq)]
pub struct FrequencyBin {
    pub frequency_hz: f32,
    pub magnitude: f32,
    pub phase_rad: f32,
}

/// Frequency analyzer that computes the DFT of a signal.
pub struct FrequencyAnalyzer {
    sample_rate: f32,
}

/// Construct a new FrequencyAnalyzer.
pub fn new_frequency_analyzer(sample_rate: f32) -> FrequencyAnalyzer {
    FrequencyAnalyzer {
        sample_rate: sample_rate.max(1.0),
    }
}

impl FrequencyAnalyzer {
    /// Compute the DFT of `samples` and return frequency bins.
    /// Only returns bins up to the Nyquist frequency.
    pub fn analyze(&self, samples: &[f32]) -> Vec<FrequencyBin> {
        let n = samples.len();
        if n == 0 {
            return Vec::new();
        }
        let n_out = n / 2 + 1;
        let mut bins = Vec::with_capacity(n_out);
        let n_f = n as f32;

        for k in 0..n_out {
            let mut re = 0.0f32;
            let mut im = 0.0f32;
            for (j, &s) in samples.iter().enumerate() {
                let angle = TAU * k as f32 * j as f32 / n_f;
                re += s * angle.cos();
                im -= s * angle.sin();
            }
            let magnitude = (re * re + im * im).sqrt() / n_f;
            let phase_rad = im.atan2(re);
            let frequency_hz = k as f32 * self.sample_rate / n_f;
            bins.push(FrequencyBin {
                frequency_hz,
                magnitude,
                phase_rad,
            });
        }
        bins
    }

    /// Return the dominant frequency (highest magnitude bin, excluding DC).
    pub fn dominant_frequency(&self, samples: &[f32]) -> Option<f32> {
        let bins = self.analyze(samples);
        bins.iter()
            .skip(1) /* skip DC */
            .max_by(|a, b| {
                a.magnitude
                    .partial_cmp(&b.magnitude)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|b| b.frequency_hz)
    }

    /// Total spectral power of the signal.
    pub fn total_power(&self, samples: &[f32]) -> f32 {
        self.analyze(samples)
            .iter()
            .map(|b| b.magnitude * b.magnitude)
            .sum()
    }

    /// Sample rate accessor.
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}

/// Compute the magnitude spectrum (convenience wrapper).
pub fn magnitude_spectrum(samples: &[f32], sample_rate: f32) -> Vec<f32> {
    let fa = new_frequency_analyzer(sample_rate);
    fa.analyze(samples)
        .into_iter()
        .map(|b| b.magnitude)
        .collect()
}

/// Compute DC component (mean) of the signal.
pub fn dc_component(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.iter().sum::<f32>() / samples.len() as f32
}

/// Return the index of the peak magnitude bin (excluding DC).
pub fn peak_bin_index(samples: &[f32], sample_rate: f32) -> Option<usize> {
    let fa = new_frequency_analyzer(sample_rate);
    let bins = fa.analyze(samples);
    bins.iter()
        .enumerate()
        .skip(1)
        .max_by(|(_, a), (_, b)| {
            a.magnitude
                .partial_cmp(&b.magnitude)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::TAU;

    fn sine_wave(freq: f32, sample_rate: f32, n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| (TAU * freq * i as f32 / sample_rate).sin())
            .collect()
    }

    #[test]
    fn test_analyzer_creates() {
        /* new_frequency_analyzer returns a valid struct */
        let fa = new_frequency_analyzer(44100.0);
        assert!((fa.sample_rate() - 44100.0).abs() < 1.0);
    }

    #[test]
    fn test_analyze_empty() {
        /* analyze of empty signal is empty */
        let fa = new_frequency_analyzer(1000.0);
        assert!(fa.analyze(&[]).is_empty());
    }

    #[test]
    fn test_analyze_dc() {
        /* constant signal has large DC bin */
        let fa = new_frequency_analyzer(1000.0);
        let samples = vec![1.0f32; 64];
        let bins = fa.analyze(&samples);
        assert!(!bins.is_empty());
        assert!(bins[0].magnitude > 0.5);
    }

    #[test]
    fn test_dominant_frequency_sine() {
        /* dominant frequency of a 10 Hz sine at 1000 Hz SR is ~10 Hz */
        let sr = 1000.0f32;
        let samples = sine_wave(10.0, sr, 256);
        let fa = new_frequency_analyzer(sr);
        let dom = fa.dominant_frequency(&samples).unwrap();
        assert!((dom - 10.0).abs() < 5.0, "dom={dom}");
    }

    #[test]
    fn test_magnitude_spectrum_length() {
        /* magnitude_spectrum length = n/2 + 1 */
        let samples = vec![0.0f32; 64];
        let spec = magnitude_spectrum(&samples, 1000.0);
        assert_eq!(spec.len(), 33);
    }

    #[test]
    fn test_dc_component() {
        /* DC component of constant signal equals the constant */
        let samples = vec![3.0f32; 16];
        assert!((dc_component(&samples) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_total_power_nonzero() {
        /* total_power of a non-zero signal is positive */
        let fa = new_frequency_analyzer(1000.0);
        let samples = sine_wave(10.0, 1000.0, 64);
        assert!(fa.total_power(&samples) > 0.0);
    }

    #[test]
    fn test_peak_bin_index_some() {
        /* peak_bin_index returns Some for non-trivial signal */
        let samples = sine_wave(10.0, 1000.0, 128);
        assert!(peak_bin_index(&samples, 1000.0).is_some());
    }

    #[test]
    fn test_frequency_hz_range() {
        /* all bin frequencies are within [0, sample_rate/2] */
        let fa = new_frequency_analyzer(500.0);
        let samples = vec![1.0f32; 32];
        for b in fa.analyze(&samples) {
            assert!((0.0..=250.0).contains(&b.frequency_hz));
        }
    }
}
