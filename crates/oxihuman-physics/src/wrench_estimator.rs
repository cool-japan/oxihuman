// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! External wrench estimator stub.

/// A 6D wrench: [fx, fy, fz, tx, ty, tz].
#[derive(Debug, Clone, PartialEq)]
pub struct Wrench {
    pub force: [f32; 3],
    pub torque: [f32; 3],
}

impl Default for Wrench {
    fn default() -> Self {
        Self {
            force: [0.0; 3],
            torque: [0.0; 3],
        }
    }
}

impl Wrench {
    pub fn new(force: [f32; 3], torque: [f32; 3]) -> Self {
        Self { force, torque }
    }

    pub fn force_magnitude(&self) -> f32 {
        self.force.iter().map(|v| v * v).sum::<f32>().sqrt()
    }

    pub fn torque_magnitude(&self) -> f32 {
        self.torque.iter().map(|v| v * v).sum::<f32>().sqrt()
    }
}

/// Wrench estimator configuration.
#[derive(Debug, Clone)]
pub struct WrenchEstimatorConfig {
    pub alpha: f32,
    pub force_threshold: f32,
}

impl Default for WrenchEstimatorConfig {
    fn default() -> Self {
        Self {
            alpha: 0.2,
            force_threshold: 1.0,
        }
    }
}

/// Wrench estimator state.
#[derive(Debug, Clone, Default)]
pub struct WrenchEstimator {
    pub estimated: Wrench,
    pub config: WrenchEstimatorConfig,
}

impl WrenchEstimator {
    pub fn new(config: WrenchEstimatorConfig) -> Self {
        Self {
            estimated: Wrench::default(),
            config,
        }
    }

    pub fn default_estimator() -> Self {
        Self::new(WrenchEstimatorConfig::default())
    }
}

/// Update the wrench estimate given measured and expected wrenches.
pub fn update_wrench_estimate(
    estimator: &mut WrenchEstimator,
    measured: &Wrench,
    expected: &Wrench,
) {
    /* stub: residual = measured - expected, filtered by alpha */
    let alpha = estimator.config.alpha;
    for i in 0..3 {
        let f_residual = measured.force[i] - expected.force[i];
        let t_residual = measured.torque[i] - expected.torque[i];
        estimator.estimated.force[i] =
            (1.0 - alpha) * estimator.estimated.force[i] + alpha * f_residual;
        estimator.estimated.torque[i] =
            (1.0 - alpha) * estimator.estimated.torque[i] + alpha * t_residual;
    }
}

/// Return whether a significant external wrench has been detected.
pub fn external_wrench_detected(estimator: &WrenchEstimator) -> bool {
    estimator.estimated.force_magnitude() > estimator.config.force_threshold
}

/// Return the direction of the estimated force (unit vector), or None if near zero.
pub fn force_direction(wrench: &Wrench) -> Option<[f32; 3]> {
    let mag = wrench.force_magnitude();
    if mag < 1e-6 {
        return None;
    }
    Some([
        wrench.force[0] / mag,
        wrench.force[1] / mag,
        wrench.force[2] / mag,
    ])
}

/// Add two wrenches component-wise.
pub fn add_wrenches(a: &Wrench, b: &Wrench) -> Wrench {
    Wrench {
        force: [
            a.force[0] + b.force[0],
            a.force[1] + b.force[1],
            a.force[2] + b.force[2],
        ],
        torque: [
            a.torque[0] + b.torque[0],
            a.torque[1] + b.torque[1],
            a.torque[2] + b.torque[2],
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_wrench_zero() {
        /* default wrench is zero */
        let w = Wrench::default();
        assert_eq!(w.force_magnitude(), 0.0);
    }

    #[test]
    fn test_force_magnitude() {
        /* 3-4-5 triangle in force */
        let w = Wrench::new([3.0, 4.0, 0.0], [0.0; 3]);
        assert!((w.force_magnitude() - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_torque_magnitude() {
        /* torque magnitude */
        let w = Wrench::new([0.0; 3], [0.0, 0.0, 2.0]);
        assert!((w.torque_magnitude() - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_update_converges() {
        /* estimated force converges towards residual */
        let mut est = WrenchEstimator::default_estimator();
        let measured = Wrench::new([10.0, 0.0, 0.0], [0.0; 3]);
        let expected = Wrench::default();
        for _ in 0..50 {
            update_wrench_estimate(&mut est, &measured, &expected);
        }
        assert!(est.estimated.force[0] > 5.0);
    }

    #[test]
    fn test_external_wrench_not_detected_initially() {
        /* no external wrench at start */
        let est = WrenchEstimator::default_estimator();
        assert!(!external_wrench_detected(&est));
    }

    #[test]
    fn test_force_direction_none_when_zero() {
        /* zero force returns None */
        let w = Wrench::default();
        assert!(force_direction(&w).is_none());
    }

    #[test]
    fn test_force_direction_unit() {
        /* direction has unit length */
        let w = Wrench::new([1.0, 2.0, 3.0], [0.0; 3]);
        let d = force_direction(&w).unwrap();
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_add_wrenches() {
        /* component-wise addition */
        let a = Wrench::new([1.0, 2.0, 3.0], [0.1, 0.2, 0.3]);
        let b = Wrench::new([4.0, 5.0, 6.0], [0.4, 0.5, 0.6]);
        let c = add_wrenches(&a, &b);
        assert!((c.force[0] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_alpha_range() {
        /* alpha is in [0,1] for default config */
        let cfg = WrenchEstimatorConfig::default();
        assert!((0.0..=1.0).contains(&cfg.alpha));
    }
}
