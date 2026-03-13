// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Load cell force sensor model.

/// Load cell configuration.
#[derive(Debug, Clone)]
pub struct LoadCellConfig {
    /// Maximum rated force in Newtons.
    pub max_force_n: f32,
    /// Sensitivity in mV/V/N.
    pub sensitivity_mv_per_v_per_n: f32,
    /// Excitation voltage in Volts.
    pub excitation_v: f32,
    /// Non-linearity error as fraction of full scale.
    pub nonlinearity_fraction: f32,
    /// Sample rate in Hz.
    pub sample_rate_hz: f32,
}

impl Default for LoadCellConfig {
    fn default() -> Self {
        LoadCellConfig {
            max_force_n: 500.0,
            sensitivity_mv_per_v_per_n: 2.0,
            excitation_v: 5.0,
            nonlinearity_fraction: 0.001,
            sample_rate_hz: 1000.0,
        }
    }
}

/// A single load cell sample.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadCellSample {
    pub time: f32,
    /// Measured force in Newtons.
    pub force_n: f32,
}

/// Load cell sensor.
#[derive(Debug)]
pub struct LoadCellSensor {
    pub config: LoadCellConfig,
    samples: Vec<LoadCellSample>,
}

impl LoadCellSensor {
    /// Create a new load cell sensor.
    pub fn new(config: LoadCellConfig) -> Self {
        LoadCellSensor {
            config,
            samples: vec![],
        }
    }

    /// Record a sample.
    pub fn push_sample(&mut self, s: LoadCellSample) {
        self.samples.push(s);
    }

    /// Return sample count.
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Return the latest sample.
    pub fn latest(&self) -> Option<&LoadCellSample> {
        self.samples.last()
    }

    /// Clear all samples.
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

/// Convert a bridge output voltage (mV) to force (N).
pub fn voltage_to_force(bridge_mv: f32, config: &LoadCellConfig) -> f32 {
    let sensitivity = config.sensitivity_mv_per_v_per_n * config.excitation_v;
    if sensitivity < 1e-12 {
        return 0.0;
    }
    bridge_mv / sensitivity
}

/// Convert a force to the expected bridge output voltage (mV).
pub fn force_to_voltage(force_n: f32, config: &LoadCellConfig) -> f32 {
    force_n * config.sensitivity_mv_per_v_per_n * config.excitation_v
}

/// Return `true` if the force exceeds the rated capacity.
pub fn force_overrange(force_n: f32, config: &LoadCellConfig) -> bool {
    force_n.abs() > config.max_force_n
}

/// Compute the maximum non-linearity error in Newtons.
pub fn nonlinearity_error_n(config: &LoadCellConfig) -> f32 {
    config.max_force_n * config.nonlinearity_fraction
}

/// Compute the peak force in a list of samples.
pub fn peak_force(samples: &[LoadCellSample]) -> f32 {
    samples
        .iter()
        .map(|s| s.force_n.abs())
        .fold(0.0f32, f32::max)
}

/// Compute the mean force.
pub fn mean_force(samples: &[LoadCellSample]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.iter().map(|s| s.force_n).sum::<f32>() / samples.len() as f32
}

/// Compute impulse (force × dt) from adjacent samples.
pub fn compute_impulse(samples: &[LoadCellSample]) -> f32 {
    if samples.len() < 2 {
        return 0.0;
    }
    let mut imp = 0.0f32;
    for i in 1..samples.len() {
        let dt = (samples[i].time - samples[i - 1].time).abs();
        imp += samples[i].force_n * dt;
    }
    imp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_force_to_voltage_zero() {
        /* zero force gives zero voltage */
        let cfg = LoadCellConfig::default();
        assert!(force_to_voltage(0.0, &cfg).abs() < 1e-10);
    }

    #[test]
    fn test_voltage_force_roundtrip() {
        /* force → voltage → force roundtrip */
        let cfg = LoadCellConfig::default();
        let f = 100.0f32;
        let v = force_to_voltage(f, &cfg);
        let back = voltage_to_force(v, &cfg);
        assert!((back - f).abs() < 0.001);
    }

    #[test]
    fn test_force_overrange_true() {
        /* force above max is overrange */
        let cfg = LoadCellConfig::default();
        assert!(force_overrange(cfg.max_force_n + 1.0, &cfg));
    }

    #[test]
    fn test_force_overrange_false() {
        /* force within range is not overrange */
        let cfg = LoadCellConfig::default();
        assert!(!force_overrange(cfg.max_force_n / 2.0, &cfg));
    }

    #[test]
    fn test_peak_force() {
        /* peak_force finds max abs */
        let samples = vec![
            LoadCellSample {
                time: 0.0,
                force_n: -300.0,
            },
            LoadCellSample {
                time: 0.1,
                force_n: 100.0,
            },
        ];
        assert!((peak_force(&samples) - 300.0).abs() < 1e-5);
    }

    #[test]
    fn test_mean_force_empty() {
        /* mean of empty is 0 */
        assert_eq!(mean_force(&[]), 0.0);
    }

    #[test]
    fn test_impulse_two_samples() {
        /* impulse over 0.1 s at 100 N */
        let samples = vec![
            LoadCellSample {
                time: 0.0,
                force_n: 100.0,
            },
            LoadCellSample {
                time: 0.1,
                force_n: 100.0,
            },
        ];
        let imp = compute_impulse(&samples);
        assert!((imp - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_nonlinearity_error() {
        /* nonlinearity error is fraction of max force */
        let cfg = LoadCellConfig::default();
        let err = nonlinearity_error_n(&cfg);
        assert!((err - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_push_and_count() {
        /* push increments count */
        let mut s = LoadCellSensor::new(LoadCellConfig::default());
        s.push_sample(LoadCellSample {
            time: 0.0,
            force_n: 0.0,
        });
        assert_eq!(s.sample_count(), 1);
    }

    #[test]
    fn test_clear() {
        /* clear removes samples */
        let mut s = LoadCellSensor::new(LoadCellConfig::default());
        s.push_sample(LoadCellSample {
            time: 0.0,
            force_n: 0.0,
        });
        s.clear();
        assert_eq!(s.sample_count(), 0);
    }
}
