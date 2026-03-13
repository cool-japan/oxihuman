// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Strain gauge sensor model.

/// Strain gauge configuration.
#[derive(Debug, Clone)]
pub struct StrainGaugeConfig {
    /// Gauge factor (sensitivity).
    pub gauge_factor: f32,
    /// Nominal resistance in ohms.
    pub nominal_resistance_ohm: f32,
    /// Maximum strain (µε).
    pub max_strain_ue: f32,
    /// Noise floor in µε RMS.
    pub noise_ue: f32,
}

impl Default for StrainGaugeConfig {
    fn default() -> Self {
        StrainGaugeConfig {
            gauge_factor: 2.0,
            nominal_resistance_ohm: 350.0,
            max_strain_ue: 3000.0,
            noise_ue: 1.0,
        }
    }
}

/// A single strain gauge sample.
#[derive(Debug, Clone, PartialEq)]
pub struct StrainSample {
    pub time: f32,
    /// Measured strain in microstrain (µε).
    pub strain_ue: f32,
}

/// Strain gauge sensor.
#[derive(Debug)]
pub struct StrainGaugeSensor {
    pub config: StrainGaugeConfig,
    samples: Vec<StrainSample>,
}

impl StrainGaugeSensor {
    /// Create a new strain gauge sensor.
    pub fn new(config: StrainGaugeConfig) -> Self {
        StrainGaugeSensor {
            config,
            samples: vec![],
        }
    }

    /// Record a sample.
    pub fn push_sample(&mut self, s: StrainSample) {
        self.samples.push(s);
    }

    /// Return sample count.
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Return the latest sample.
    pub fn latest(&self) -> Option<&StrainSample> {
        self.samples.last()
    }

    /// Clear all samples.
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

/// Compute the change in resistance for a given strain.
pub fn delta_resistance(strain_ue: f32, config: &StrainGaugeConfig) -> f32 {
    let strain = strain_ue * 1e-6;
    config.gauge_factor * config.nominal_resistance_ohm * strain
}

/// Convert a resistance change to strain in µε.
pub fn resistance_to_strain(delta_r: f32, config: &StrainGaugeConfig) -> f32 {
    if config.nominal_resistance_ohm < 1e-9 || config.gauge_factor < 1e-9 {
        return 0.0;
    }
    let strain = delta_r / (config.gauge_factor * config.nominal_resistance_ohm);
    strain * 1e6
}

/// Compute the stress from strain using Young's modulus (Pa).
pub fn strain_to_stress_pa(strain_ue: f32, youngs_modulus_pa: f32) -> f32 {
    strain_ue * 1e-6 * youngs_modulus_pa
}

/// Return `true` if strain exceeds the maximum rated value.
pub fn strain_overrange(strain_ue: f32, config: &StrainGaugeConfig) -> bool {
    strain_ue.abs() > config.max_strain_ue
}

/// Compute the peak absolute strain in a list of samples.
pub fn peak_strain(samples: &[StrainSample]) -> f32 {
    samples
        .iter()
        .map(|s| s.strain_ue.abs())
        .fold(0.0f32, f32::max)
}

/// Compute the mean strain.
pub fn mean_strain(samples: &[StrainSample]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.iter().map(|s| s.strain_ue).sum::<f32>() / samples.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_gauge_factor() {
        /* default gauge factor is 2.0 */
        assert_eq!(StrainGaugeConfig::default().gauge_factor, 2.0);
    }

    #[test]
    fn test_delta_resistance_zero_strain() {
        /* zero strain gives zero ΔR */
        let cfg = StrainGaugeConfig::default();
        assert!(delta_resistance(0.0, &cfg).abs() < 1e-10);
    }

    #[test]
    fn test_roundtrip() {
        /* strain → ΔR → strain roundtrip */
        let cfg = StrainGaugeConfig::default();
        let original = 500.0f32; /* 500 µε */
        let dr = delta_resistance(original, &cfg);
        let back = resistance_to_strain(dr, &cfg);
        assert!((back - original).abs() < 0.01);
    }

    #[test]
    fn test_stress_calculation() {
        /* 1000 µε × 200 GPa = 200 MPa */
        let stress = strain_to_stress_pa(1000.0, 200e9);
        assert!((stress - 200e6).abs() < 1.0);
    }

    #[test]
    fn test_strain_overrange_true() {
        /* exceeding max_strain flags overrange */
        let cfg = StrainGaugeConfig::default();
        assert!(strain_overrange(cfg.max_strain_ue + 1.0, &cfg));
    }

    #[test]
    fn test_peak_strain() {
        /* peak finds max abs */
        let samples = vec![
            StrainSample {
                time: 0.0,
                strain_ue: -200.0,
            },
            StrainSample {
                time: 0.1,
                strain_ue: 100.0,
            },
        ];
        assert!((peak_strain(&samples) - 200.0).abs() < 1e-5);
    }

    #[test]
    fn test_mean_strain_empty() {
        /* mean of empty is 0 */
        assert_eq!(mean_strain(&[]), 0.0);
    }

    #[test]
    fn test_push_and_count() {
        /* push increments count */
        let mut s = StrainGaugeSensor::new(StrainGaugeConfig::default());
        s.push_sample(StrainSample {
            time: 0.0,
            strain_ue: 0.0,
        });
        assert_eq!(s.sample_count(), 1);
    }

    #[test]
    fn test_clear() {
        /* clear removes samples */
        let mut s = StrainGaugeSensor::new(StrainGaugeConfig::default());
        s.push_sample(StrainSample {
            time: 0.0,
            strain_ue: 0.0,
        });
        s.clear();
        assert_eq!(s.sample_count(), 0);
    }

    #[test]
    fn test_latest() {
        /* latest returns last pushed */
        let mut s = StrainGaugeSensor::new(StrainGaugeConfig::default());
        s.push_sample(StrainSample {
            time: 0.7,
            strain_ue: 150.0,
        });
        assert_eq!(s.latest().expect("should succeed").time, 0.7);
    }
}
