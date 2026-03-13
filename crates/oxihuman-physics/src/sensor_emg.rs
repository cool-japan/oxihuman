// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! EMG (electromyography) muscle signal sensor stub.

/// EMG channel configuration.
#[derive(Debug, Clone)]
pub struct EmgConfig {
    /// Number of EMG channels.
    pub channel_count: usize,
    /// Sample rate in Hz.
    pub sample_rate_hz: f32,
    /// Noise floor in µV RMS.
    pub noise_uv: f32,
    /// High-pass cutoff frequency in Hz.
    pub hp_cutoff_hz: f32,
    /// Low-pass cutoff frequency in Hz.
    pub lp_cutoff_hz: f32,
}

impl Default for EmgConfig {
    fn default() -> Self {
        EmgConfig {
            channel_count: 8,
            sample_rate_hz: 2000.0,
            noise_uv: 1.0,
            hp_cutoff_hz: 20.0,
            lp_cutoff_hz: 500.0,
        }
    }
}

/// A single EMG sample frame (one value per channel).
#[derive(Debug, Clone)]
pub struct EmgFrame {
    pub time: f32,
    /// Signal amplitude per channel in µV.
    pub channels: Vec<f32>,
}

/// EMG sensor model.
#[derive(Debug)]
pub struct EmgSensor {
    pub config: EmgConfig,
    frames: Vec<EmgFrame>,
}

impl EmgSensor {
    /// Create a new EMG sensor.
    pub fn new(config: EmgConfig) -> Self {
        EmgSensor {
            config,
            frames: vec![],
        }
    }

    /// Record a frame.
    pub fn push_frame(&mut self, frame: EmgFrame) {
        self.frames.push(frame);
    }

    /// Return the number of recorded frames.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Return the latest frame, if any.
    pub fn latest(&self) -> Option<&EmgFrame> {
        self.frames.last()
    }

    /// Clear all frames.
    pub fn clear(&mut self) {
        self.frames.clear();
    }
}

/// Compute the root-mean-square of an EMG channel signal.
pub fn rms(signal: &[f32]) -> f32 {
    if signal.is_empty() {
        return 0.0;
    }
    let mean_sq = signal.iter().map(|&x| x * x).sum::<f32>() / signal.len() as f32;
    mean_sq.sqrt()
}

/// Apply a simple moving average envelope (window in samples).
pub fn moving_average_envelope(signal: &[f32], window: usize) -> Vec<f32> {
    if signal.is_empty() || window == 0 {
        return vec![];
    }
    let w = window.min(signal.len());
    let mut out = Vec::with_capacity(signal.len());
    for i in 0..signal.len() {
        let start = i.saturating_sub(w / 2);
        let end = (i + w / 2 + 1).min(signal.len());
        let sum: f32 = signal[start..end].iter().map(|&x| x.abs()).sum();
        out.push(sum / (end - start) as f32);
    }
    out
}

/// Return the channel index with the highest RMS across a frame.
pub fn dominant_channel(frame: &EmgFrame) -> Option<usize> {
    frame
        .channels
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            a.abs()
                .partial_cmp(&b.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
}

/// Normalise a signal to the range [0, 1] by its maximum absolute value.
pub fn normalise(signal: &[f32]) -> Vec<f32> {
    let max = signal.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    if max < 1e-12 {
        return vec![0.0; signal.len()];
    }
    signal.iter().map(|&x| x / max).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_channel_count() {
        /* default has 8 channels */
        assert_eq!(EmgConfig::default().channel_count, 8);
    }

    #[test]
    fn test_push_frame() {
        /* push increments frame count */
        let mut emg = EmgSensor::new(EmgConfig::default());
        emg.push_frame(EmgFrame {
            time: 0.0,
            channels: vec![0.0; 8],
        });
        assert_eq!(emg.frame_count(), 1);
    }

    #[test]
    fn test_clear() {
        /* clear resets count */
        let mut emg = EmgSensor::new(EmgConfig::default());
        emg.push_frame(EmgFrame {
            time: 0.0,
            channels: vec![0.0; 8],
        });
        emg.clear();
        assert_eq!(emg.frame_count(), 0);
    }

    #[test]
    fn test_rms_constant_signal() {
        /* RMS of constant [2,2,2] is 2 */
        let sig = vec![2.0f32; 10];
        assert!((rms(&sig) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_rms_empty() {
        /* empty signal RMS is 0 */
        assert_eq!(rms(&[]), 0.0);
    }

    #[test]
    fn test_dominant_channel() {
        /* dominant channel is the one with highest absolute value */
        let frame = EmgFrame {
            time: 0.0,
            channels: vec![1.0, 5.0, 3.0],
        };
        assert_eq!(dominant_channel(&frame), Some(1));
    }

    #[test]
    fn test_normalise_max_one() {
        /* normalised signal has max 1 */
        let sig = vec![2.0f32, 4.0, 1.0];
        let n = normalise(&sig);
        assert!((n.iter().cloned().fold(0.0f32, f32::max) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalise_zero_signal() {
        /* zero signal normalises to zeros */
        let n = normalise(&[0.0, 0.0]);
        assert!(n.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_moving_average_length() {
        /* envelope length matches input */
        let sig = vec![1.0f32; 20];
        let env = moving_average_envelope(&sig, 5);
        assert_eq!(env.len(), 20);
    }

    #[test]
    fn test_latest_frame() {
        /* latest returns last pushed */
        let mut emg = EmgSensor::new(EmgConfig::default());
        emg.push_frame(EmgFrame {
            time: 0.5,
            channels: vec![1.0],
        });
        assert_eq!(emg.latest().expect("should succeed").time, 0.5);
    }
}
