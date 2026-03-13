// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Torsional spring constraint between two rigid bodies or bones.
//!
//! A torsion spring resists angular displacement from a rest angle.  The
//! restoring torque follows the linear spring law: `τ = -k * (θ - θ_rest)`,
//! with optional viscous damping `τ_d = -d * ω`.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Configuration for a torsion spring.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TorsionSpringConfig {
    /// Spring stiffness coefficient (N·m / rad).
    pub stiffness: f32,
    /// Damping coefficient (N·m·s / rad).
    pub damping: f32,
    /// Rest angle in radians.
    pub rest_angle: f32,
    /// Minimum allowed angle (rad).
    pub min_angle: f32,
    /// Maximum allowed angle (rad).
    pub max_angle: f32,
}

/// Returns a sensible default [`TorsionSpringConfig`].
#[allow(dead_code)]
pub fn default_torsion_spring_config() -> TorsionSpringConfig {
    TorsionSpringConfig {
        stiffness: 10.0,
        damping: 0.5,
        rest_angle: 0.0,
        min_angle: -PI,
        max_angle: PI,
    }
}

/// State of a torsion spring.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TorsionSpring {
    /// Current angle (rad).
    pub angle: f32,
    /// Current angular velocity (rad / s).
    pub angular_velocity: f32,
    /// Moment of inertia of the body being driven (kg·m²).
    pub inertia: f32,
    /// Configuration.
    pub config: TorsionSpringConfig,
}

/// Output of a single torsion-spring step.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct TorsionSpringResult {
    /// Net torque applied this step (N·m).
    pub torque: f32,
    /// Elastic potential energy (J).
    pub energy: f32,
    /// Angle after this step (rad).
    pub angle: f32,
    /// Whether the spring is at rest (within tolerance).
    pub at_rest: bool,
}

/// Create a new [`TorsionSpring`] with the given inertia and config.
#[allow(dead_code)]
pub fn new_torsion_spring(inertia: f32, config: TorsionSpringConfig) -> TorsionSpring {
    TorsionSpring {
        angle: config.rest_angle,
        angular_velocity: 0.0,
        inertia,
        config,
    }
}

/// Compute the restoring + damping torque for the current state.
#[allow(dead_code)]
pub fn torsion_spring_torque(spring: &TorsionSpring) -> f32 {
    let displacement = spring.angle - spring.config.rest_angle;
    -spring.config.stiffness * displacement - spring.config.damping * spring.angular_velocity
}

/// Elastic potential energy stored in the spring (J).
#[allow(dead_code)]
pub fn torsion_spring_energy(spring: &TorsionSpring) -> f32 {
    let d = spring.angle - spring.config.rest_angle;
    0.5 * spring.config.stiffness * d * d
}

/// Advance the spring by `dt` seconds (semi-implicit Euler integration).
#[allow(dead_code)]
pub fn torsion_spring_step(spring: &mut TorsionSpring, dt: f32) -> TorsionSpringResult {
    let torque = torsion_spring_torque(spring);
    let alpha = torque / spring.inertia.max(1e-12);
    spring.angular_velocity += alpha * dt;
    spring.angle += spring.angular_velocity * dt;
    spring.angle = torsion_spring_clamp_angle(spring, spring.angle);
    let energy = torsion_spring_energy(spring);
    let at_rest = torsion_spring_at_rest(spring, 1e-4);
    TorsionSpringResult { torque, energy, angle: spring.angle, at_rest }
}

/// Current angle of the spring (rad).
#[allow(dead_code)]
pub fn torsion_spring_angle(spring: &TorsionSpring) -> f32 {
    spring.angle
}

/// Returns `true` if the spring is at rest within `tol` rad.
#[allow(dead_code)]
pub fn torsion_spring_at_rest(spring: &TorsionSpring, tol: f32) -> bool {
    (spring.angle - spring.config.rest_angle).abs() < tol
        && spring.angular_velocity.abs() < tol
}

/// Clamp an angle to the spring's angle limits.
#[allow(dead_code)]
pub fn torsion_spring_clamp_angle(spring: &TorsionSpring, angle: f32) -> f32 {
    angle.clamp(spring.config.min_angle, spring.config.max_angle)
}

/// Set a new rest angle and optionally reset angular velocity.
#[allow(dead_code)]
pub fn torsion_spring_set_rest(spring: &mut TorsionSpring, rest_angle: f32) {
    spring.config.rest_angle = rest_angle;
}

/// Reset the spring to its rest angle with zero angular velocity.
#[allow(dead_code)]
pub fn torsion_spring_reset(spring: &mut TorsionSpring) {
    spring.angle = spring.config.rest_angle;
    spring.angular_velocity = 0.0;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_spring(stiffness: f32, damping: f32, rest: f32) -> TorsionSpring {
        let mut cfg = default_torsion_spring_config();
        cfg.stiffness = stiffness;
        cfg.damping = damping;
        cfg.rest_angle = rest;
        new_torsion_spring(1.0, cfg)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_torsion_spring_config();
        assert!(cfg.stiffness > 0.0);
        assert!(cfg.damping >= 0.0);
    }

    #[test]
    fn test_new_spring_at_rest() {
        let spring = make_spring(10.0, 0.5, 0.0);
        assert!(torsion_spring_at_rest(&spring, 1e-9));
    }

    #[test]
    fn test_torque_at_rest_is_zero() {
        let spring = make_spring(10.0, 0.0, 0.0);
        assert!(torsion_spring_torque(&spring).abs() < 1e-9);
    }

    #[test]
    fn test_torque_positive_displacement() {
        let mut spring = make_spring(10.0, 0.0, 0.0);
        spring.angle = 1.0; // displaced by +1 rad
        let torque = torsion_spring_torque(&spring);
        assert!(torque < 0.0, "restoring torque should oppose displacement");
    }

    #[test]
    fn test_energy_zero_at_rest() {
        let spring = make_spring(10.0, 0.0, 0.0);
        assert!(torsion_spring_energy(&spring).abs() < 1e-9);
    }

    #[test]
    fn test_energy_positive_when_displaced() {
        let mut spring = make_spring(10.0, 0.0, 0.0);
        spring.angle = 0.5;
        assert!(torsion_spring_energy(&spring) > 0.0);
    }

    #[test]
    fn test_step_moves_toward_rest() {
        let mut spring = make_spring(100.0, 1.0, 0.0);
        spring.angle = 1.0;
        let mut prev = spring.angle;
        for _ in 0..200 {
            torsion_spring_step(&mut spring, 0.01);
            // angle should trend toward 0
            let _ = prev; // used below
            prev = spring.angle;
        }
        // After many steps, should be close to rest.
        assert!(spring.angle.abs() < 0.5, "expected convergence toward rest, got {}", prev);
    }

    #[test]
    fn test_clamp_angle_within_limits() {
        let spring = make_spring(10.0, 0.0, 0.0);
        assert_eq!(torsion_spring_clamp_angle(&spring, 10.0), spring.config.max_angle);
        assert_eq!(torsion_spring_clamp_angle(&spring, -10.0), spring.config.min_angle);
        assert_eq!(torsion_spring_clamp_angle(&spring, 0.5), 0.5);
    }

    #[test]
    fn test_set_rest() {
        let mut spring = make_spring(10.0, 0.0, 0.0);
        torsion_spring_set_rest(&mut spring, 0.5);
        assert_eq!(spring.config.rest_angle, 0.5);
    }

    #[test]
    fn test_reset() {
        let mut spring = make_spring(10.0, 0.0, 0.0);
        spring.angle = 1.0;
        spring.angular_velocity = 5.0;
        torsion_spring_reset(&mut spring);
        assert!(torsion_spring_at_rest(&spring, 1e-9));
    }

    #[test]
    fn test_result_fields() {
        let mut spring = make_spring(10.0, 0.5, 0.0);
        spring.angle = 0.3;
        let result = torsion_spring_step(&mut spring, 0.01);
        assert!(result.energy >= 0.0);
        assert!(result.angle.is_finite());
    }
}
