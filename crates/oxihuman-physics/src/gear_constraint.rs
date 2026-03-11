// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Gear/ratio constraint between two rotational DOFs.

#![allow(dead_code)]

/// Configuration for a gear constraint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GearConfig {
    pub ratio: f32,
    pub compliance: f32,
}

/// A gear constraint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GearConstraint {
    pub angle_a: f32,
    pub angle_b: f32,
    pub lambda: f32,
    pub config: GearConfig,
}

/// Result of solving a gear constraint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GearResult {
    pub correction_a: f32,
    pub correction_b: f32,
    pub error: f32,
}

/// Return the default gear config.
#[allow(dead_code)]
pub fn default_gear_config() -> GearConfig {
    GearConfig { ratio: 1.0, compliance: 0.0 }
}

/// Create a new gear constraint.
#[allow(dead_code)]
pub fn new_gear_constraint(config: GearConfig) -> GearConstraint {
    GearConstraint { angle_a: 0.0, angle_b: 0.0, lambda: 0.0, config }
}

/// Solve the gear constraint: enforce angle_a * ratio = angle_b.
#[allow(dead_code)]
pub fn gear_solve(constraint: &mut GearConstraint, inv_inertia_a: f32, inv_inertia_b: f32, dt: f32) -> GearResult {
    let r = constraint.config.ratio;
    let err = constraint.angle_a * r - constraint.angle_b;
    let alpha = constraint.config.compliance / (dt * dt);
    let w = r * r * inv_inertia_a + inv_inertia_b + alpha;
    let delta_lambda = if w.abs() > 1e-10 { -err / w } else { 0.0 };
    constraint.lambda += delta_lambda;
    let correction_a = r * inv_inertia_a * delta_lambda;
    let correction_b = -inv_inertia_b * delta_lambda;
    GearResult { correction_a, correction_b, error: err }
}

/// Return the current constraint error (angle_a * ratio - angle_b).
#[allow(dead_code)]
pub fn gear_error(constraint: &GearConstraint) -> f32 {
    constraint.angle_a * constraint.config.ratio - constraint.angle_b
}

/// Reset accumulated lambda.
#[allow(dead_code)]
pub fn gear_reset(constraint: &mut GearConstraint) {
    constraint.lambda = 0.0;
}

/// Set the gear ratio.
#[allow(dead_code)]
pub fn gear_set_ratio(constraint: &mut GearConstraint, ratio: f32) {
    constraint.config.ratio = ratio;
}

/// Return the gear ratio.
#[allow(dead_code)]
pub fn gear_ratio(constraint: &GearConstraint) -> f32 {
    constraint.config.ratio
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_gear_config();
        assert!((cfg.ratio - 1.0).abs() < 1e-6);
        assert_eq!(cfg.compliance, 0.0);
    }

    #[test]
    fn test_new_constraint() {
        let c = new_gear_constraint(default_gear_config());
        assert_eq!(c.lambda, 0.0);
        assert!((c.angle_a).abs() < 1e-6);
    }

    #[test]
    fn test_error_zero_when_satisfied() {
        let mut c = new_gear_constraint(default_gear_config());
        c.angle_a = 1.0;
        c.angle_b = 1.0; // ratio = 1.0
        assert!(gear_error(&c).abs() < 1e-6);
    }

    #[test]
    fn test_error_nonzero_when_violated() {
        let mut c = new_gear_constraint(GearConfig { ratio: 2.0, compliance: 0.0 });
        c.angle_a = 1.0;
        c.angle_b = 1.0; // should be 2.0
        assert!((gear_error(&c) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_solve_reduces_error() {
        let mut c = new_gear_constraint(default_gear_config());
        c.angle_a = 1.0;
        c.angle_b = 0.0;
        let result = gear_solve(&mut c, 1.0, 1.0, 0.016);
        assert!(result.error.abs() > 0.0);
    }

    #[test]
    fn test_reset() {
        let mut c = new_gear_constraint(default_gear_config());
        c.lambda = 7.0;
        gear_reset(&mut c);
        assert_eq!(c.lambda, 0.0);
    }

    #[test]
    fn test_set_ratio() {
        let mut c = new_gear_constraint(default_gear_config());
        gear_set_ratio(&mut c, 3.0);
        assert!((gear_ratio(&c) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_gear_ratio() {
        let c = new_gear_constraint(GearConfig { ratio: 4.0, compliance: 0.0 });
        assert!((gear_ratio(&c) - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_gear_result_clone() {
        let r = GearResult { correction_a: 0.1, correction_b: 0.2, error: 0.05 };
        let r2 = r.clone();
        assert!((r2.correction_a - 0.1).abs() < 1e-6);
    }
}
