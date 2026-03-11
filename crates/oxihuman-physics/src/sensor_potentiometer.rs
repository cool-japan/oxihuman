// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Rotary potentiometer sensor model.

/// Potentiometer configuration.
#[derive(Debug, Clone)]
pub struct PotentiometerConfig {
    /// Total resistance in ohms.
    pub total_resistance_ohm: f32,
    /// Angular range in degrees (0 to this value).
    pub range_deg: f32,
    /// Supply voltage in Volts.
    pub supply_voltage_v: f32,
    /// Linearity error as fraction of full scale.
    pub linearity_error: f32,
}

impl Default for PotentiometerConfig {
    fn default() -> Self {
        PotentiometerConfig {
            total_resistance_ohm: 10_000.0,
            range_deg: 300.0,
            supply_voltage_v: 5.0,
            linearity_error: 0.005,
        }
    }
}

/// A single potentiometer sample.
#[derive(Debug, Clone, PartialEq)]
pub struct PotSample {
    pub time: f32,
    /// Output voltage in Volts.
    pub voltage_v: f32,
}

/// Potentiometer sensor.
#[derive(Debug)]
pub struct PotentiometerSensor {
    pub config: PotentiometerConfig,
    samples: Vec<PotSample>,
}

impl PotentiometerSensor {
    /// Create a new potentiometer sensor.
    pub fn new(config: PotentiometerConfig) -> Self {
        PotentiometerSensor {
            config,
            samples: vec![],
        }
    }

    /// Record a sample.
    pub fn push_sample(&mut self, s: PotSample) {
        self.samples.push(s);
    }

    /// Return sample count.
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Return the latest sample.
    pub fn latest(&self) -> Option<&PotSample> {
        self.samples.last()
    }

    /// Clear all samples.
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

/// Convert an output voltage to an angle in degrees.
pub fn voltage_to_angle(voltage_v: f32, config: &PotentiometerConfig) -> f32 {
    let fraction = (voltage_v / config.supply_voltage_v).clamp(0.0, 1.0);
    fraction * config.range_deg
}

/// Convert an angle to the expected output voltage.
pub fn angle_to_voltage(angle_deg: f32, config: &PotentiometerConfig) -> f32 {
    let fraction = (angle_deg / config.range_deg).clamp(0.0, 1.0);
    fraction * config.supply_voltage_v
}

/// Return `true` if the voltage is within [0, supply_voltage].
pub fn voltage_in_range(voltage_v: f32, config: &PotentiometerConfig) -> bool {
    (0.0..=config.supply_voltage_v).contains(&voltage_v)
}

/// Compute the angular velocity in deg/s from two consecutive samples.
pub fn angular_velocity_deg_s(a: &PotSample, b: &PotSample, config: &PotentiometerConfig) -> f32 {
    let dt = (b.time - a.time).abs();
    if dt < 1e-9 {
        return 0.0;
    }
    let angle_a = voltage_to_angle(a.voltage_v, config);
    let angle_b = voltage_to_angle(b.voltage_v, config);
    (angle_b - angle_a) / dt
}

/// Compute the maximum linearity error in degrees.
pub fn max_linearity_error_deg(config: &PotentiometerConfig) -> f32 {
    config.range_deg * config.linearity_error
}

/// Return the current angle from the latest sample, or 0.0.
pub fn current_angle_deg(sensor: &PotentiometerSensor) -> f32 {
    sensor
        .latest()
        .map(|s| voltage_to_angle(s.voltage_v, &sensor.config))
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voltage_to_angle_min() {
        /* zero voltage → 0° */
        let cfg = PotentiometerConfig::default();
        assert_eq!(voltage_to_angle(0.0, &cfg), 0.0);
    }

    #[test]
    fn test_voltage_to_angle_max() {
        /* supply voltage → range_deg */
        let cfg = PotentiometerConfig::default();
        assert!((voltage_to_angle(cfg.supply_voltage_v, &cfg) - cfg.range_deg).abs() < 1e-4);
    }

    #[test]
    fn test_angle_to_voltage_roundtrip() {
        /* angle → voltage → angle roundtrip */
        let cfg = PotentiometerConfig::default();
        let v = angle_to_voltage(150.0, &cfg);
        let a = voltage_to_angle(v, &cfg);
        assert!((a - 150.0).abs() < 0.001);
    }

    #[test]
    fn test_voltage_in_range() {
        /* mid-supply voltage is in range */
        let cfg = PotentiometerConfig::default();
        assert!(voltage_in_range(2.5, &cfg));
    }

    #[test]
    fn test_voltage_out_of_range() {
        /* negative voltage is out of range */
        let cfg = PotentiometerConfig::default();
        assert!(!voltage_in_range(-0.1, &cfg));
    }

    #[test]
    fn test_angular_velocity() {
        /* 150° change over 1 s = 150 deg/s */
        let cfg = PotentiometerConfig::default();
        let a = PotSample {
            time: 0.0,
            voltage_v: 0.0,
        };
        let b = PotSample {
            time: 1.0,
            voltage_v: angle_to_voltage(150.0, &cfg),
        };
        let av = angular_velocity_deg_s(&a, &b, &cfg);
        assert!((av - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_max_linearity_error() {
        /* linearity error in degrees */
        let cfg = PotentiometerConfig::default();
        let err = max_linearity_error_deg(&cfg);
        assert!((err - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_push_and_count() {
        /* push increments count */
        let mut s = PotentiometerSensor::new(PotentiometerConfig::default());
        s.push_sample(PotSample {
            time: 0.0,
            voltage_v: 0.0,
        });
        assert_eq!(s.sample_count(), 1);
    }

    #[test]
    fn test_clear() {
        /* clear removes samples */
        let mut s = PotentiometerSensor::new(PotentiometerConfig::default());
        s.push_sample(PotSample {
            time: 0.0,
            voltage_v: 0.0,
        });
        s.clear();
        assert_eq!(s.sample_count(), 0);
    }

    #[test]
    fn test_current_angle_empty() {
        /* empty sensor returns 0° */
        let s = PotentiometerSensor::new(PotentiometerConfig::default());
        assert_eq!(current_angle_deg(&s), 0.0);
    }
}
