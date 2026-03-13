// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Angular/rotation constraint between bodies.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AngularConfig {
    pub min_angle: f32,
    pub max_angle: f32,
    pub compliance: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AngularConstraint {
    pub angle: f32,
    pub lambda: f32,
    pub config: AngularConfig,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AngularResult {
    pub correction: f32,
    pub clamped: bool,
    pub was_satisfied: bool,
}

#[allow(dead_code)]
pub fn default_angular_config() -> AngularConfig {
    AngularConfig {
        min_angle: -std::f32::consts::PI,
        max_angle: std::f32::consts::PI,
        compliance: 0.0,
    }
}

#[allow(dead_code)]
pub fn new_angular_constraint(angle: f32, config: AngularConfig) -> AngularConstraint {
    AngularConstraint { angle, lambda: 0.0, config }
}

#[allow(dead_code)]
pub fn angular_solve(ac: &mut AngularConstraint, dt: f32) -> AngularResult {
    let clamped_angle = ac.angle.clamp(ac.config.min_angle, ac.config.max_angle);
    let clamped = (clamped_angle - ac.angle).abs() > 1e-9;

    let error = ac.angle - clamped_angle;
    let was_satisfied = error.abs() < 1e-4;

    let compliance = ac.config.compliance / (dt * dt);
    let denom = 1.0 + compliance;
    let correction = if denom.abs() > 1e-12 {
        (-error - compliance * ac.lambda) / denom
    } else {
        0.0
    };
    ac.lambda += correction;
    ac.angle += correction;

    AngularResult { correction, clamped, was_satisfied }
}

#[allow(dead_code)]
pub fn angular_error(ac: &AngularConstraint) -> f32 {
    let clamped = ac.angle.clamp(ac.config.min_angle, ac.config.max_angle);
    (ac.angle - clamped).abs()
}

#[allow(dead_code)]
pub fn angular_is_at_limit(ac: &AngularConstraint) -> bool {
    (ac.angle - ac.config.min_angle).abs() < 1e-4
        || (ac.angle - ac.config.max_angle).abs() < 1e-4
}

#[allow(dead_code)]
pub fn angular_reset(ac: &mut AngularConstraint) {
    ac.lambda = 0.0;
}

#[allow(dead_code)]
pub fn angular_set_angle(ac: &mut AngularConstraint, angle: f32) {
    ac.angle = angle;
}

#[allow(dead_code)]
pub fn angular_stiffness(ac: &AngularConstraint) -> f32 {
    if ac.config.compliance > 1e-12 {
        1.0 / ac.config.compliance
    } else {
        f32::INFINITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_constraint() {
        let cfg = default_angular_config();
        let ac = new_angular_constraint(0.5, cfg);
        assert!((ac.angle - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_error_in_range() {
        let cfg = default_angular_config();
        let ac = new_angular_constraint(0.5, cfg);
        assert!(angular_error(&ac) < 1e-5);
    }

    #[test]
    fn test_error_out_of_range() {
        let cfg = AngularConfig { min_angle: -1.0, max_angle: 1.0, compliance: 0.0 };
        let ac = new_angular_constraint(2.0, cfg);
        assert!(angular_error(&ac) > 0.0);
    }

    #[test]
    fn test_solve_corrects_angle() {
        let cfg = AngularConfig { min_angle: -1.0, max_angle: 1.0, compliance: 0.0 };
        let mut ac = new_angular_constraint(2.0, cfg);
        angular_solve(&mut ac, 0.016);
        assert!(ac.angle <= 1.0 + 1e-4);
    }

    #[test]
    fn test_is_at_limit() {
        let cfg = AngularConfig { min_angle: -1.0, max_angle: 1.0, compliance: 0.0 };
        let ac = new_angular_constraint(1.0, cfg);
        assert!(angular_is_at_limit(&ac));
    }

    #[test]
    fn test_reset_lambda() {
        let cfg = default_angular_config();
        let mut ac = new_angular_constraint(0.0, cfg);
        ac.lambda = 3.0;
        angular_reset(&mut ac);
        assert!(ac.lambda.abs() < 1e-9);
    }

    #[test]
    fn test_set_angle() {
        let cfg = default_angular_config();
        let mut ac = new_angular_constraint(0.0, cfg);
        angular_set_angle(&mut ac, 1.5);
        assert!((ac.angle - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_stiffness_infinite_for_rigid() {
        let cfg = AngularConfig { min_angle: -1.0, max_angle: 1.0, compliance: 0.0 };
        let ac = new_angular_constraint(0.0, cfg);
        assert!(angular_stiffness(&ac).is_infinite());
    }

    #[test]
    fn test_stiffness_finite_for_compliant() {
        let cfg = AngularConfig { min_angle: -1.0, max_angle: 1.0, compliance: 0.01 };
        let ac = new_angular_constraint(0.0, cfg);
        assert!(angular_stiffness(&ac).is_finite());
    }
}
