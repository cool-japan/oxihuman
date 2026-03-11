// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Tactile/touch sensor grid model.

/// Tactile sensor configuration.
#[derive(Debug, Clone)]
pub struct TactileConfig {
    /// Grid dimensions [rows, cols].
    pub grid: [usize; 2],
    /// Physical cell size in metres [width, height].
    pub cell_size_m: [f32; 2],
    /// Maximum force per taxel in Newtons.
    pub max_force_per_taxel_n: f32,
    /// Sample rate in Hz.
    pub sample_rate_hz: f32,
}

impl Default for TactileConfig {
    fn default() -> Self {
        TactileConfig {
            grid: [16, 16],
            cell_size_m: [0.002, 0.002],
            max_force_per_taxel_n: 10.0,
            sample_rate_hz: 200.0,
        }
    }
}

/// A tactile sensor frame.
#[derive(Debug, Clone)]
pub struct TactileFrame {
    pub time: f32,
    /// Force per taxel in Newtons, length = rows × cols.
    pub taxels: Vec<f32>,
}

/// Tactile sensor.
#[derive(Debug)]
pub struct TactileSensor {
    pub config: TactileConfig,
    frames: Vec<TactileFrame>,
}

impl TactileSensor {
    /// Create a new tactile sensor.
    pub fn new(config: TactileConfig) -> Self {
        TactileSensor {
            config,
            frames: vec![],
        }
    }

    /// Record a frame.
    pub fn push_frame(&mut self, frame: TactileFrame) {
        self.frames.push(frame);
    }

    /// Return frame count.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Return the latest frame.
    pub fn latest(&self) -> Option<&TactileFrame> {
        self.frames.last()
    }

    /// Clear all frames.
    pub fn clear(&mut self) {
        self.frames.clear();
    }

    /// Return total taxel count.
    pub fn taxel_count(&self) -> usize {
        self.config.grid[0] * self.config.grid[1]
    }
}

/// Compute the total contact force across all taxels.
pub fn total_contact_force(frame: &TactileFrame) -> f32 {
    frame.taxels.iter().sum()
}

/// Count active (non-zero) taxels above a threshold.
pub fn active_taxels(frame: &TactileFrame, threshold_n: f32) -> usize {
    frame.taxels.iter().filter(|&&f| f > threshold_n).count()
}

/// Compute the centre of force as a taxel index [row, col].
pub fn centre_of_force(frame: &TactileFrame, config: &TactileConfig) -> Option<[f32; 2]> {
    let total: f32 = frame.taxels.iter().sum();
    if total < 1e-9 {
        return None;
    }
    let cols = config.grid[1];
    let rows = config.grid[0];
    let mut wr = 0.0f32;
    let mut wc = 0.0f32;
    for r in 0..rows {
        for c in 0..cols {
            let idx = r * cols + c;
            if idx >= frame.taxels.len() {
                break;
            }
            let f = frame.taxels[idx];
            wr += f * r as f32;
            wc += f * c as f32;
        }
    }
    Some([wr / total, wc / total])
}

/// Return the peak taxel force in a frame.
pub fn peak_taxel_force(frame: &TactileFrame) -> f32 {
    frame.taxels.iter().cloned().fold(0.0f32, f32::max)
}

/// Return `true` if any taxel exceeds the rated maximum.
pub fn any_taxel_overrange(frame: &TactileFrame, config: &TactileConfig) -> bool {
    frame
        .taxels
        .iter()
        .any(|&f| f > config.max_force_per_taxel_n)
}

/// Compute the contact area in m².
pub fn contact_area_m2(frame: &TactileFrame, config: &TactileConfig, threshold_n: f32) -> f32 {
    let n = active_taxels(frame, threshold_n) as f32;
    n * config.cell_size_m[0] * config.cell_size_m[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_taxel_count() {
        /* taxel count is rows × cols */
        let s = TactileSensor::new(TactileConfig::default());
        assert_eq!(s.taxel_count(), 16 * 16);
    }

    #[test]
    fn test_total_force() {
        /* sum of all taxels */
        let frame = TactileFrame {
            time: 0.0,
            taxels: vec![1.0; 4],
        };
        assert!((total_contact_force(&frame) - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_active_taxels() {
        /* count above threshold */
        let frame = TactileFrame {
            time: 0.0,
            taxels: vec![0.0, 1.0, 2.0],
        };
        assert_eq!(active_taxels(&frame, 0.5), 2);
    }

    #[test]
    fn test_centre_of_force_uniform() {
        /* uniform grid: CoF at geometric centre */
        let cfg = TactileConfig {
            grid: [2, 2],
            cell_size_m: [0.01, 0.01],
            max_force_per_taxel_n: 10.0,
            sample_rate_hz: 100.0,
        };
        let frame = TactileFrame {
            time: 0.0,
            taxels: vec![1.0; 4],
        };
        let cof = centre_of_force(&frame, &cfg).unwrap();
        assert!((cof[0] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_centre_of_force_zero() {
        /* zero force returns None */
        let cfg = TactileConfig::default();
        let frame = TactileFrame {
            time: 0.0,
            taxels: vec![0.0; 16],
        };
        assert!(centre_of_force(&frame, &cfg).is_none());
    }

    #[test]
    fn test_peak_taxel_force() {
        /* peak finds max */
        let frame = TactileFrame {
            time: 0.0,
            taxels: vec![0.0, 5.0, 3.0],
        };
        assert!((peak_taxel_force(&frame) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_overrange_true() {
        /* force above max flags overrange */
        let cfg = TactileConfig::default();
        let frame = TactileFrame {
            time: 0.0,
            taxels: vec![cfg.max_force_per_taxel_n + 1.0],
        };
        assert!(any_taxel_overrange(&frame, &cfg));
    }

    #[test]
    fn test_push_and_count() {
        /* push increments count */
        let mut s = TactileSensor::new(TactileConfig::default());
        s.push_frame(TactileFrame {
            time: 0.0,
            taxels: vec![0.0; 256],
        });
        assert_eq!(s.frame_count(), 1);
    }

    #[test]
    fn test_contact_area() {
        /* contact area for 4 active taxels at 1 cm each */
        let cfg = TactileConfig {
            grid: [2, 2],
            cell_size_m: [0.01, 0.01],
            max_force_per_taxel_n: 10.0,
            sample_rate_hz: 100.0,
        };
        let frame = TactileFrame {
            time: 0.0,
            taxels: vec![1.0; 4],
        };
        let area = contact_area_m2(&frame, &cfg, 0.5);
        assert!((area - 4.0 * 0.0001).abs() < 1e-9);
    }

    #[test]
    fn test_clear() {
        /* clear removes frames */
        let mut s = TactileSensor::new(TactileConfig::default());
        s.push_frame(TactileFrame {
            time: 0.0,
            taxels: vec![],
        });
        s.clear();
        assert_eq!(s.frame_count(), 0);
    }
}
