// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

/// Rolling friction model.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct RollingFriction {
    pub coefficient: f32,
    pub spin_coeff: f32,
}

/// Create a default rolling friction model.
#[allow(dead_code)]
pub fn default_rolling_friction() -> RollingFriction {
    RollingFriction {
        coefficient: 0.01,
        spin_coeff: 0.005,
    }
}

/// Compute the rolling friction torque opposing rolling motion.
/// `normal_force` is the perpendicular contact force, `radius` is the wheel radius.
/// Returns torque magnitude (always non-negative).
#[allow(dead_code)]
pub fn rolling_friction_torque(rf: &RollingFriction, normal_force: f32, radius: f32) -> f32 {
    (rf.coefficient * normal_force * radius).max(0.0)
}

/// Compute the spin friction torque (opposing spinning about the normal axis).
/// Returns torque magnitude (always non-negative).
#[allow(dead_code)]
pub fn spin_friction_torque(rf: &RollingFriction, normal_force: f32) -> f32 {
    (rf.spin_coeff * normal_force).max(0.0)
}

/// Compute the effective deceleration due to rolling friction.
/// `normal_force` (N), `mass` (kg), `radius` (m).
/// Deceleration = rolling_torque / (mass * radius^2) → linear deceleration.
#[allow(dead_code)]
pub fn effective_rolling_decel(
    rf: &RollingFriction,
    normal_force: f32,
    mass: f32,
    radius: f32,
) -> f32 {
    if mass <= 0.0 || radius <= 0.0 {
        return 0.0;
    }
    let torque = rolling_friction_torque(rf, normal_force, radius);
    torque / (mass * radius)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_rolling_friction_positive_coeffs() {
        let rf = default_rolling_friction();
        assert!(rf.coefficient > 0.0);
        assert!(rf.spin_coeff > 0.0);
    }

    #[test]
    fn rolling_torque_proportional_to_normal_force() {
        let rf = default_rolling_friction();
        let t1 = rolling_friction_torque(&rf, 100.0, 0.3);
        let t2 = rolling_friction_torque(&rf, 200.0, 0.3);
        assert!((t2 - 2.0 * t1).abs() < 1e-5);
    }

    #[test]
    fn rolling_torque_proportional_to_radius() {
        let rf = default_rolling_friction();
        let t1 = rolling_friction_torque(&rf, 100.0, 0.3);
        let t2 = rolling_friction_torque(&rf, 100.0, 0.6);
        assert!((t2 - 2.0 * t1).abs() < 1e-5);
    }

    #[test]
    fn rolling_torque_zero_force() {
        let rf = default_rolling_friction();
        assert_eq!(rolling_friction_torque(&rf, 0.0, 0.3), 0.0);
    }

    #[test]
    fn spin_torque_proportional_to_normal_force() {
        let rf = default_rolling_friction();
        let t1 = spin_friction_torque(&rf, 50.0);
        let t2 = spin_friction_torque(&rf, 100.0);
        assert!((t2 - 2.0 * t1).abs() < 1e-5);
    }

    #[test]
    fn spin_torque_nonnegative() {
        let rf = default_rolling_friction();
        assert!(spin_friction_torque(&rf, 0.0) >= 0.0);
    }

    #[test]
    fn effective_decel_positive() {
        let rf = default_rolling_friction();
        let a = effective_rolling_decel(&rf, 500.0, 10.0, 0.3);
        assert!(a > 0.0);
    }

    #[test]
    fn effective_decel_zero_mass_returns_zero() {
        let rf = default_rolling_friction();
        assert_eq!(effective_rolling_decel(&rf, 100.0, 0.0, 0.3), 0.0);
    }

    #[test]
    fn effective_decel_zero_radius_returns_zero() {
        let rf = default_rolling_friction();
        assert_eq!(effective_rolling_decel(&rf, 100.0, 10.0, 0.0), 0.0);
    }

    #[test]
    fn rolling_torque_value() {
        let rf = RollingFriction { coefficient: 0.01, spin_coeff: 0.005 };
        // torque = 0.01 * 1000 * 0.3 = 3.0
        let t = rolling_friction_torque(&rf, 1000.0, 0.3);
        assert!((t - 3.0).abs() < 1e-4);
    }
}
