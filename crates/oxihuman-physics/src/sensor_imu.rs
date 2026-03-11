// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! IMU (accelerometer + gyroscope) sensor model.

/// IMU sensor configuration.
#[derive(Debug, Clone)]
pub struct ImuConfig {
    /// Accelerometer noise density (m/s²/√Hz).
    pub accel_noise_density: f32,
    /// Gyroscope noise density (rad/s/√Hz).
    pub gyro_noise_density: f32,
    /// Accelerometer bias instability (m/s²).
    pub accel_bias: f32,
    /// Gyroscope bias instability (rad/s).
    pub gyro_bias: f32,
    /// Sample rate in Hz.
    pub sample_rate_hz: f32,
}

impl Default for ImuConfig {
    fn default() -> Self {
        ImuConfig {
            accel_noise_density: 1e-3,
            gyro_noise_density: 1.7e-4,
            accel_bias: 5e-4,
            gyro_bias: 1e-5,
            sample_rate_hz: 200.0,
        }
    }
}

/// A single IMU measurement.
#[derive(Debug, Clone, PartialEq)]
pub struct ImuSample {
    /// Timestamp in seconds.
    pub time: f32,
    /// Specific force in body frame (m/s²).
    pub accel: [f32; 3],
    /// Angular velocity in body frame (rad/s).
    pub gyro: [f32; 3],
}

/// IMU sensor model.
#[derive(Debug)]
pub struct ImuSensor {
    pub config: ImuConfig,
    samples: Vec<ImuSample>,
}

impl ImuSensor {
    /// Create a new IMU sensor with the given configuration.
    pub fn new(config: ImuConfig) -> Self {
        ImuSensor {
            config,
            samples: vec![],
        }
    }

    /// Push a new sample.
    pub fn push_sample(&mut self, sample: ImuSample) {
        self.samples.push(sample);
    }

    /// Return the number of stored samples.
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Return the latest sample, if any.
    pub fn latest(&self) -> Option<&ImuSample> {
        self.samples.last()
    }

    /// Clear all samples.
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

/// Compute the magnitude of a 3-vector.
pub fn vec3_magnitude(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Integrate angular velocity over a time step to obtain angle increment (rad).
pub fn integrate_gyro(gyro: [f32; 3], dt: f32) -> [f32; 3] {
    [gyro[0] * dt, gyro[1] * dt, gyro[2] * dt]
}

/// Compute roll and pitch from accelerometer data (static assumption).
pub fn accel_to_roll_pitch(accel: [f32; 3]) -> (f32, f32) {
    let roll = accel[1].atan2(accel[2]);
    let pitch = (-accel[0]).atan2((accel[1] * accel[1] + accel[2] * accel[2]).sqrt());
    (roll, pitch)
}

/// Return `true` if the accelerometer is near 1 g (static).
pub fn is_static(accel: [f32; 3], tolerance: f32) -> bool {
    let mag = vec3_magnitude(accel);
    (mag - 9.80665).abs() < tolerance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        /* default config has sensible sample rate */
        let cfg = ImuConfig::default();
        assert!(cfg.sample_rate_hz > 0.0);
    }

    #[test]
    fn test_push_and_count() {
        /* push_sample increments count */
        let mut imu = ImuSensor::new(ImuConfig::default());
        imu.push_sample(ImuSample {
            time: 0.0,
            accel: [0.0, 0.0, 9.8],
            gyro: [0.0; 3],
        });
        assert_eq!(imu.sample_count(), 1);
    }

    #[test]
    fn test_latest_sample() {
        /* latest returns last pushed sample */
        let mut imu = ImuSensor::new(ImuConfig::default());
        imu.push_sample(ImuSample {
            time: 1.0,
            accel: [0.0, 0.0, 9.8],
            gyro: [0.0; 3],
        });
        assert_eq!(imu.latest().unwrap().time, 1.0);
    }

    #[test]
    fn test_clear() {
        /* clear removes all samples */
        let mut imu = ImuSensor::new(ImuConfig::default());
        imu.push_sample(ImuSample {
            time: 0.0,
            accel: [0.0; 3],
            gyro: [0.0; 3],
        });
        imu.clear();
        assert_eq!(imu.sample_count(), 0);
    }

    #[test]
    fn test_vec3_magnitude_unit() {
        /* unit vector along z has magnitude 1 */
        let mag = vec3_magnitude([0.0, 0.0, 1.0]);
        assert!((mag - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_integrate_gyro() {
        /* gyro integration scales by dt */
        let result = integrate_gyro([1.0, 0.0, 0.0], 0.01);
        assert!((result[0] - 0.01).abs() < 1e-7);
    }

    #[test]
    fn test_accel_to_roll_pitch_flat() {
        /* flat orientation: roll and pitch near zero */
        let (roll, pitch) = accel_to_roll_pitch([0.0, 0.0, 9.8]);
        assert!(roll.abs() < 1e-5);
        assert!(pitch.abs() < 1e-5);
    }

    #[test]
    fn test_is_static_near_1g() {
        /* 1 g reading is static */
        assert!(is_static([0.0, 0.0, 9.80665], 0.1));
    }

    #[test]
    fn test_is_static_dynamic() {
        /* large acceleration is not static */
        assert!(!is_static([20.0, 0.0, 0.0], 0.5));
    }

    #[test]
    fn test_empty_sensor_no_latest() {
        /* new sensor has no latest sample */
        let imu = ImuSensor::new(ImuConfig::default());
        assert!(imu.latest().is_none());
    }
}
