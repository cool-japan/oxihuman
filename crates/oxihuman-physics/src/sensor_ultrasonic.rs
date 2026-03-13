// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ultrasonic distance sensor model.

/// Ultrasonic sensor configuration.
#[derive(Debug, Clone)]
pub struct UltrasonicConfig {
    /// Speed of sound in m/s.
    pub speed_of_sound_m_s: f32,
    /// Minimum measurable distance in metres.
    pub min_range_m: f32,
    /// Maximum measurable distance in metres.
    pub max_range_m: f32,
    /// Beam angle (half-angle) in degrees.
    pub beam_angle_deg: f32,
    /// Sample rate in Hz.
    pub sample_rate_hz: f32,
}

impl Default for UltrasonicConfig {
    fn default() -> Self {
        UltrasonicConfig {
            speed_of_sound_m_s: 343.0,
            min_range_m: 0.02,
            max_range_m: 4.0,
            beam_angle_deg: 15.0,
            sample_rate_hz: 40.0,
        }
    }
}

/// A single ultrasonic measurement.
#[derive(Debug, Clone, PartialEq)]
pub struct UltrasonicSample {
    pub time: f32,
    /// Measured distance in metres, or `None` if out of range.
    pub distance_m: Option<f32>,
}

/// Ultrasonic distance sensor.
#[derive(Debug)]
pub struct UltrasonicSensor {
    pub config: UltrasonicConfig,
    samples: Vec<UltrasonicSample>,
}

impl UltrasonicSensor {
    /// Create a new ultrasonic sensor.
    pub fn new(config: UltrasonicConfig) -> Self {
        UltrasonicSensor {
            config,
            samples: vec![],
        }
    }

    /// Record a sample.
    pub fn push_sample(&mut self, s: UltrasonicSample) {
        self.samples.push(s);
    }

    /// Return sample count.
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Return the latest sample.
    pub fn latest(&self) -> Option<&UltrasonicSample> {
        self.samples.last()
    }

    /// Clear all samples.
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

/// Convert a round-trip time-of-flight (seconds) to distance (metres).
pub fn tof_to_distance(tof_s: f32, speed_m_s: f32) -> f32 {
    tof_s * speed_m_s / 2.0
}

/// Convert a distance (metres) to expected round-trip ToF (seconds).
pub fn distance_to_tof(distance_m: f32, speed_m_s: f32) -> f32 {
    2.0 * distance_m / speed_m_s
}

/// Return `true` if a distance reading is within the sensor's valid range.
pub fn distance_in_range(distance_m: f32, config: &UltrasonicConfig) -> bool {
    (config.min_range_m..=config.max_range_m).contains(&distance_m)
}

/// Compute the approximate beam footprint diameter at a given distance.
pub fn beam_footprint_m(distance_m: f32, config: &UltrasonicConfig) -> f32 {
    let half_angle_rad = config.beam_angle_deg.to_radians();
    2.0 * distance_m * half_angle_rad.tan()
}

/// Compute the median distance from a set of samples (ignores None values).
pub fn median_distance(samples: &[UltrasonicSample]) -> Option<f32> {
    let mut valid: Vec<f32> = samples.iter().filter_map(|s| s.distance_m).collect();
    if valid.is_empty() {
        return None;
    }
    valid.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = valid.len() / 2;
    Some(if valid.len().is_multiple_of(2) {
        (valid[mid - 1] + valid[mid]) / 2.0
    } else {
        valid[mid]
    })
}

/// Return the fraction of samples that have valid readings.
pub fn valid_fraction(samples: &[UltrasonicSample]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let valid = samples.iter().filter(|s| s.distance_m.is_some()).count();
    valid as f32 / samples.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tof_to_distance() {
        /* 1 ms ToF at 343 m/s = 0.1715 m */
        let d = tof_to_distance(0.001, 343.0);
        assert!((d - 0.1715).abs() < 0.001);
    }

    #[test]
    fn test_distance_to_tof_roundtrip() {
        /* distance → ToF → distance roundtrip */
        let d = 1.5f32;
        let tof = distance_to_tof(d, 343.0);
        let back = tof_to_distance(tof, 343.0);
        assert!((back - d).abs() < 1e-5);
    }

    #[test]
    fn test_distance_in_range_true() {
        /* 1 m is within default range */
        assert!(distance_in_range(1.0, &UltrasonicConfig::default()));
    }

    #[test]
    fn test_distance_in_range_false() {
        /* 10 m is out of range */
        assert!(!distance_in_range(10.0, &UltrasonicConfig::default()));
    }

    #[test]
    fn test_beam_footprint_positive() {
        /* footprint at 1 m is positive */
        let fp = beam_footprint_m(1.0, &UltrasonicConfig::default());
        assert!(fp > 0.0);
    }

    #[test]
    fn test_median_distance_odd() {
        /* median of [1, 2, 3] is 2 */
        let samples = vec![
            UltrasonicSample {
                time: 0.0,
                distance_m: Some(3.0),
            },
            UltrasonicSample {
                time: 0.1,
                distance_m: Some(1.0),
            },
            UltrasonicSample {
                time: 0.2,
                distance_m: Some(2.0),
            },
        ];
        assert_eq!(median_distance(&samples), Some(2.0));
    }

    #[test]
    fn test_median_distance_none() {
        /* all None → median is None */
        let samples = vec![UltrasonicSample {
            time: 0.0,
            distance_m: None,
        }];
        assert!(median_distance(&samples).is_none());
    }

    #[test]
    fn test_valid_fraction() {
        /* 2 valid out of 3 = 2/3 */
        let samples = vec![
            UltrasonicSample {
                time: 0.0,
                distance_m: Some(1.0),
            },
            UltrasonicSample {
                time: 0.1,
                distance_m: Some(1.5),
            },
            UltrasonicSample {
                time: 0.2,
                distance_m: None,
            },
        ];
        let frac = valid_fraction(&samples);
        assert!((frac - 2.0 / 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_push_and_count() {
        /* push increments count */
        let mut s = UltrasonicSensor::new(UltrasonicConfig::default());
        s.push_sample(UltrasonicSample {
            time: 0.0,
            distance_m: Some(1.0),
        });
        assert_eq!(s.sample_count(), 1);
    }

    #[test]
    fn test_clear() {
        /* clear removes samples */
        let mut s = UltrasonicSensor::new(UltrasonicConfig::default());
        s.push_sample(UltrasonicSample {
            time: 0.0,
            distance_m: None,
        });
        s.clear();
        assert_eq!(s.sample_count(), 0);
    }

    #[test]
    fn test_valid_fraction_empty() {
        /* empty slice has zero valid fraction */
        assert_eq!(valid_fraction(&[]), 0.0);
    }
}
