// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Joint angle encoder sensor model.

/// Encoder configuration.
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    /// Pulses per revolution.
    pub ppr: u32,
    /// Whether the encoder is absolute (true) or incremental (false).
    pub absolute: bool,
    /// Maximum rotational speed in RPM.
    pub max_rpm: f32,
    /// Sample rate in Hz.
    pub sample_rate_hz: f32,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        EncoderConfig {
            ppr: 4096,
            absolute: true,
            max_rpm: 3000.0,
            sample_rate_hz: 1000.0,
        }
    }
}

/// A single encoder sample.
#[derive(Debug, Clone, PartialEq)]
pub struct EncoderSample {
    pub time: f32,
    /// Raw pulse count.
    pub count: i64,
}

/// Encoder sensor.
#[derive(Debug)]
pub struct EncoderSensor {
    pub config: EncoderConfig,
    samples: Vec<EncoderSample>,
}

impl EncoderSensor {
    /// Create a new encoder sensor.
    pub fn new(config: EncoderConfig) -> Self {
        EncoderSensor {
            config,
            samples: vec![],
        }
    }

    /// Record a sample.
    pub fn push_sample(&mut self, s: EncoderSample) {
        self.samples.push(s);
    }

    /// Return sample count.
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Return the latest sample.
    pub fn latest(&self) -> Option<&EncoderSample> {
        self.samples.last()
    }

    /// Clear all samples.
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

/// Convert pulse count to angle in degrees.
pub fn count_to_degrees(count: i64, ppr: u32) -> f64 {
    360.0 * count as f64 / ppr as f64
}

/// Convert pulse count to angle in radians.
pub fn count_to_radians(count: i64, ppr: u32) -> f64 {
    std::f64::consts::TAU * count as f64 / ppr as f64
}

/// Compute angular velocity in rad/s from two consecutive samples.
pub fn angular_velocity_rad_s(a: &EncoderSample, b: &EncoderSample, ppr: u32) -> f64 {
    let dt = (b.time - a.time) as f64;
    if dt.abs() < 1e-12 {
        return 0.0;
    }
    let delta_rad = count_to_radians(b.count - a.count, ppr);
    delta_rad / dt
}

/// Compute angular velocity in RPM.
pub fn angular_velocity_rpm(a: &EncoderSample, b: &EncoderSample, ppr: u32) -> f64 {
    angular_velocity_rad_s(a, b, ppr) * 60.0 / std::f64::consts::TAU
}

/// Return the latest angle in degrees, or 0 if no samples.
pub fn current_angle_deg(sensor: &EncoderSensor) -> f64 {
    sensor
        .latest()
        .map(|s| count_to_degrees(s.count, sensor.config.ppr))
        .unwrap_or(0.0)
}

/// Return `true` if the computed RPM exceeds the rated maximum.
pub fn rpm_overrange(rpm: f64, config: &EncoderConfig) -> bool {
    rpm.abs() > config.max_rpm as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_to_degrees_full() {
        /* full revolution (ppr counts) = 360° */
        let ppr = 4096u32;
        let angle = count_to_degrees(ppr as i64, ppr);
        assert!((angle - 360.0).abs() < 1e-9);
    }

    #[test]
    fn test_count_to_degrees_zero() {
        /* zero count = 0° */
        assert_eq!(count_to_degrees(0, 1024), 0.0);
    }

    #[test]
    fn test_count_to_radians_full() {
        /* full revolution = 2π rad */
        let ppr = 4096u32;
        let rad = count_to_radians(ppr as i64, ppr);
        assert!((rad - std::f64::consts::TAU).abs() < 1e-9);
    }

    #[test]
    fn test_angular_velocity_zero() {
        /* no position change → zero velocity */
        let a = EncoderSample {
            time: 0.0,
            count: 100,
        };
        let b = EncoderSample {
            time: 0.1,
            count: 100,
        };
        assert_eq!(angular_velocity_rad_s(&a, &b, 4096), 0.0);
    }

    #[test]
    fn test_angular_velocity_positive() {
        /* positive count change → positive velocity */
        let a = EncoderSample {
            time: 0.0,
            count: 0,
        };
        let b = EncoderSample {
            time: 1.0,
            count: 4096,
        };
        let rpm = angular_velocity_rpm(&a, &b, 4096);
        assert!((rpm - 60.0).abs() < 1e-6);
    }

    #[test]
    fn test_push_and_count() {
        /* push increments count */
        let mut s = EncoderSensor::new(EncoderConfig::default());
        s.push_sample(EncoderSample {
            time: 0.0,
            count: 0,
        });
        assert_eq!(s.sample_count(), 1);
    }

    #[test]
    fn test_current_angle_empty() {
        /* empty sensor returns 0 degrees */
        let s = EncoderSensor::new(EncoderConfig::default());
        assert_eq!(current_angle_deg(&s), 0.0);
    }

    #[test]
    fn test_rpm_overrange() {
        /* rpm above max flags overrange */
        let cfg = EncoderConfig::default();
        assert!(rpm_overrange(cfg.max_rpm as f64 + 1.0, &cfg));
    }

    #[test]
    fn test_clear() {
        /* clear removes samples */
        let mut s = EncoderSensor::new(EncoderConfig::default());
        s.push_sample(EncoderSample {
            time: 0.0,
            count: 0,
        });
        s.clear();
        assert_eq!(s.sample_count(), 0);
    }

    #[test]
    fn test_latest() {
        /* latest returns last pushed */
        let mut s = EncoderSensor::new(EncoderConfig::default());
        s.push_sample(EncoderSample {
            time: 1.0,
            count: 512,
        });
        assert_eq!(s.latest().expect("should succeed").count, 512);
    }
}
