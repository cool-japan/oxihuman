// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Flex/bend sensor model.

/// Configuration for a flex sensor.
#[derive(Debug, Clone)]
pub struct FlexConfig {
    /// Resistance at 0° bend in ohms.
    pub resistance_flat_ohm: f32,
    /// Resistance at maximum bend in ohms.
    pub resistance_max_ohm: f32,
    /// Maximum bend angle in degrees.
    pub max_angle_deg: f32,
    /// Noise standard deviation in ohms.
    pub noise_ohm: f32,
}

impl Default for FlexConfig {
    fn default() -> Self {
        FlexConfig {
            resistance_flat_ohm: 25_000.0,
            resistance_max_ohm: 125_000.0,
            max_angle_deg: 90.0,
            noise_ohm: 500.0,
        }
    }
}

/// A single flex sensor reading.
#[derive(Debug, Clone, PartialEq)]
pub struct FlexSample {
    pub time: f32,
    /// Measured resistance in ohms.
    pub resistance_ohm: f32,
}

/// Flex sensor model.
#[derive(Debug)]
pub struct FlexSensor {
    pub config: FlexConfig,
    samples: Vec<FlexSample>,
}

impl FlexSensor {
    /// Create a new flex sensor.
    pub fn new(config: FlexConfig) -> Self {
        FlexSensor {
            config,
            samples: vec![],
        }
    }

    /// Record a sample.
    pub fn push_sample(&mut self, s: FlexSample) {
        self.samples.push(s);
    }

    /// Return sample count.
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Return the latest sample.
    pub fn latest(&self) -> Option<&FlexSample> {
        self.samples.last()
    }

    /// Clear all samples.
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

/// Convert a resistance reading to an estimated bend angle in degrees.
pub fn resistance_to_angle(r_ohm: f32, config: &FlexConfig) -> f32 {
    let r_flat = config.resistance_flat_ohm;
    let r_max = config.resistance_max_ohm;
    if (r_max - r_flat).abs() < 1e-6 {
        return 0.0;
    }
    let t = (r_ohm - r_flat) / (r_max - r_flat);
    t.clamp(0.0, 1.0) * config.max_angle_deg
}

/// Convert a target angle to the expected sensor resistance.
pub fn angle_to_resistance(angle_deg: f32, config: &FlexConfig) -> f32 {
    let t = (angle_deg / config.max_angle_deg).clamp(0.0, 1.0);
    config.resistance_flat_ohm + t * (config.resistance_max_ohm - config.resistance_flat_ohm)
}

/// Return `true` if the resistance is within the valid sensor range.
pub fn resistance_in_range(r_ohm: f32, config: &FlexConfig) -> bool {
    let lo = config.resistance_flat_ohm.min(config.resistance_max_ohm);
    let hi = config.resistance_flat_ohm.max(config.resistance_max_ohm);
    (lo..=hi).contains(&r_ohm)
}

/// Compute the mean angle over a list of samples.
pub fn mean_angle(samples: &[FlexSample], config: &FlexConfig) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f32 = samples
        .iter()
        .map(|s| resistance_to_angle(s.resistance_ohm, config))
        .sum();
    sum / samples.len() as f32
}

/// Return the sample with the maximum bend angle.
pub fn max_bend_sample<'a>(
    samples: &'a [FlexSample],
    config: &FlexConfig,
) -> Option<&'a FlexSample> {
    samples.iter().max_by(|a, b| {
        resistance_to_angle(a.resistance_ohm, config)
            .partial_cmp(&resistance_to_angle(b.resistance_ohm, config))
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resistance_to_angle_flat() {
        /* flat resistance gives 0° */
        let cfg = FlexConfig::default();
        let angle = resistance_to_angle(cfg.resistance_flat_ohm, &cfg);
        assert!(angle.abs() < 1e-5);
    }

    #[test]
    fn test_resistance_to_angle_max() {
        /* max resistance gives max angle */
        let cfg = FlexConfig::default();
        let angle = resistance_to_angle(cfg.resistance_max_ohm, &cfg);
        assert!((angle - cfg.max_angle_deg).abs() < 1e-4);
    }

    #[test]
    fn test_angle_to_resistance_roundtrip() {
        /* angle → resistance → angle roundtrip */
        let cfg = FlexConfig::default();
        let r = angle_to_resistance(45.0, &cfg);
        let a = resistance_to_angle(r, &cfg);
        assert!((a - 45.0).abs() < 1e-3);
    }

    #[test]
    fn test_resistance_in_range() {
        /* mid-range resistance is valid */
        let cfg = FlexConfig::default();
        let mid = (cfg.resistance_flat_ohm + cfg.resistance_max_ohm) / 2.0;
        assert!(resistance_in_range(mid, &cfg));
    }

    #[test]
    fn test_resistance_out_of_range() {
        /* resistance above max is invalid */
        let cfg = FlexConfig::default();
        assert!(!resistance_in_range(cfg.resistance_max_ohm + 1.0, &cfg));
    }

    #[test]
    fn test_push_and_count() {
        /* push increments count */
        let mut s = FlexSensor::new(FlexConfig::default());
        s.push_sample(FlexSample {
            time: 0.0,
            resistance_ohm: 25_000.0,
        });
        assert_eq!(s.sample_count(), 1);
    }

    #[test]
    fn test_mean_angle_empty() {
        /* mean angle of empty slice is 0 */
        assert_eq!(mean_angle(&[], &FlexConfig::default()), 0.0);
    }

    #[test]
    fn test_max_bend_sample() {
        /* max_bend_sample finds highest angle */
        let cfg = FlexConfig::default();
        let samples = vec![
            FlexSample {
                time: 0.0,
                resistance_ohm: cfg.resistance_flat_ohm,
            },
            FlexSample {
                time: 0.1,
                resistance_ohm: cfg.resistance_max_ohm,
            },
        ];
        let m = max_bend_sample(&samples, &cfg).expect("should succeed");
        assert_eq!(m.time, 0.1);
    }

    #[test]
    fn test_clear() {
        /* clear removes all samples */
        let mut s = FlexSensor::new(FlexConfig::default());
        s.push_sample(FlexSample {
            time: 0.0,
            resistance_ohm: 25_000.0,
        });
        s.clear();
        assert_eq!(s.sample_count(), 0);
    }

    #[test]
    fn test_latest() {
        /* latest returns last pushed */
        let mut s = FlexSensor::new(FlexConfig::default());
        s.push_sample(FlexSample {
            time: 0.5,
            resistance_ohm: 50_000.0,
        });
        assert_eq!(s.latest().expect("should succeed").time, 0.5);
    }
}
