// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Force plate sensor model.

/// A force plate measurement (forces + moments at the plate origin).
#[derive(Debug, Clone, PartialEq)]
pub struct ForcePlateSample {
    pub time: f32,
    /// Ground reaction forces [Fx, Fy, Fz] in Newtons.
    pub force: [f32; 3],
    /// Moments [Mx, My, Mz] in Newton-metres.
    pub moment: [f32; 3],
}

/// Force plate configuration.
#[derive(Debug, Clone)]
pub struct ForcePlateConfig {
    /// Plate dimensions in metres [width, depth].
    pub dimensions: [f32; 2],
    /// Maximum measurable force in Newtons.
    pub max_force_n: f32,
    /// Sensor noise RMS in Newtons.
    pub noise_rms_n: f32,
    /// Sample rate in Hz.
    pub sample_rate_hz: f32,
}

impl Default for ForcePlateConfig {
    fn default() -> Self {
        ForcePlateConfig {
            dimensions: [0.6, 0.4],
            max_force_n: 5000.0,
            noise_rms_n: 0.1,
            sample_rate_hz: 1000.0,
        }
    }
}

/// Force plate sensor.
#[derive(Debug)]
pub struct ForcePlateSensor {
    pub config: ForcePlateConfig,
    samples: Vec<ForcePlateSample>,
}

impl ForcePlateSensor {
    /// Create a new force plate sensor.
    pub fn new(config: ForcePlateConfig) -> Self {
        ForcePlateSensor {
            config,
            samples: vec![],
        }
    }

    /// Record a sample.
    pub fn push_sample(&mut self, s: ForcePlateSample) {
        self.samples.push(s);
    }

    /// Return sample count.
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Return the latest sample.
    pub fn latest(&self) -> Option<&ForcePlateSample> {
        self.samples.last()
    }

    /// Clear all samples.
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

/// Compute the centre of pressure (CoP) from force and moment data.
/// Returns `[x, y]` on the plate, or `None` if vertical force is near zero.
pub fn centre_of_pressure(force: [f32; 3], moment: [f32; 3]) -> Option<[f32; 2]> {
    if force[2].abs() < 1e-6 {
        return None;
    }
    let x = -moment[1] / force[2];
    let y = moment[0] / force[2];
    Some([x, y])
}

/// Compute the resultant force magnitude.
pub fn resultant_force(force: [f32; 3]) -> f32 {
    (force[0] * force[0] + force[1] * force[1] + force[2] * force[2]).sqrt()
}

/// Return `true` if a sample exceeds the plate's maximum force.
pub fn force_overrange(config: &ForcePlateConfig, force: [f32; 3]) -> bool {
    resultant_force(force) > config.max_force_n
}

/// Compute mean vertical (Fz) force over all samples.
pub fn mean_fz(samples: &[ForcePlateSample]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.iter().map(|s| s.force[2]).sum::<f32>() / samples.len() as f32
}

/// Return `true` if there is a valid ground contact (Fz > threshold).
pub fn is_contact(force: [f32; 3], threshold: f32) -> bool {
    force[2] > threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_sample_rate() {
        /* default sample rate is positive */
        assert!(ForcePlateConfig::default().sample_rate_hz > 0.0);
    }

    #[test]
    fn test_push_and_count() {
        /* push increments count */
        let mut fp = ForcePlateSensor::new(ForcePlateConfig::default());
        fp.push_sample(ForcePlateSample {
            time: 0.0,
            force: [0.0; 3],
            moment: [0.0; 3],
        });
        assert_eq!(fp.sample_count(), 1);
    }

    #[test]
    fn test_clear() {
        /* clear removes samples */
        let mut fp = ForcePlateSensor::new(ForcePlateConfig::default());
        fp.push_sample(ForcePlateSample {
            time: 0.0,
            force: [0.0; 3],
            moment: [0.0; 3],
        });
        fp.clear();
        assert_eq!(fp.sample_count(), 0);
    }

    #[test]
    fn test_cop_vertical_only() {
        /* pure vertical force at origin */
        let cop = centre_of_pressure([0.0, 0.0, 100.0], [0.0, 0.0, 0.0]);
        assert_eq!(cop, Some([0.0, 0.0]));
    }

    #[test]
    fn test_cop_zero_fz() {
        /* zero Fz returns None */
        assert!(centre_of_pressure([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]).is_none());
    }

    #[test]
    fn test_resultant_force() {
        /* resultant of unit vectors */
        let f = resultant_force([1.0, 0.0, 0.0]);
        assert!((f - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_force_overrange() {
        /* force exceeding max flags as overrange */
        let cfg = ForcePlateConfig::default();
        assert!(force_overrange(&cfg, [0.0, 0.0, cfg.max_force_n + 1.0]));
    }

    #[test]
    fn test_mean_fz_empty() {
        /* mean of empty samples is 0 */
        assert_eq!(mean_fz(&[]), 0.0);
    }

    #[test]
    fn test_is_contact_true() {
        /* Fz above threshold means contact */
        assert!(is_contact([0.0, 0.0, 50.0], 10.0));
    }

    #[test]
    fn test_is_contact_false() {
        /* Fz below threshold means no contact */
        assert!(!is_contact([0.0, 0.0, 5.0], 10.0));
    }
}
