// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Force limiting and clamping utilities for physics simulation.

/// Clamp force magnitude to `max_force`.
#[allow(dead_code)]
pub fn clamp_force(force: [f32; 3], max_force: f32) -> [f32; 3] {
    let mag = (force[0] * force[0] + force[1] * force[1] + force[2] * force[2]).sqrt();
    if mag > max_force && mag > 1e-9 {
        let s = max_force / mag;
        [force[0] * s, force[1] * s, force[2] * s]
    } else {
        force
    }
}

/// Scale a force so it does not exceed a max impulse over `dt`.
#[allow(dead_code)]
pub fn limit_impulse(force: [f32; 3], max_impulse: f32, dt: f32) -> [f32; 3] {
    if dt < 1e-12 {
        return [0.0; 3];
    }
    let max_force = max_impulse / dt;
    clamp_force(force, max_force)
}

/// Force limiter with per-axis limits.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct PerAxisForceLimit {
    pub max_x: f32,
    pub max_y: f32,
    pub max_z: f32,
}

#[allow(dead_code)]
impl PerAxisForceLimit {
    pub fn uniform(max: f32) -> Self {
        Self {
            max_x: max,
            max_y: max,
            max_z: max,
        }
    }

    pub fn apply(&self, force: [f32; 3]) -> [f32; 3] {
        [
            force[0].clamp(-self.max_x, self.max_x),
            force[1].clamp(-self.max_y, self.max_y),
            force[2].clamp(-self.max_z, self.max_z),
        ]
    }
}

/// Force accumulator with a max-force budget.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BudgetedForceAccum {
    pub accumulated: [f32; 3],
    pub budget: f32,
    pub used: f32,
}

#[allow(dead_code)]
impl BudgetedForceAccum {
    pub fn new(budget: f32) -> Self {
        Self {
            accumulated: [0.0; 3],
            budget,
            used: 0.0,
        }
    }

    /// Add force if budget allows; returns amount actually added.
    pub fn add(&mut self, force: [f32; 3]) -> [f32; 3] {
        let mag = (force[0] * force[0] + force[1] * force[1] + force[2] * force[2]).sqrt();
        let remaining = (self.budget - self.used).max(0.0);
        let actual_mag = mag.min(remaining);
        if mag < 1e-9 {
            return [0.0; 3];
        }
        let s = actual_mag / mag;
        let actual = [force[0] * s, force[1] * s, force[2] * s];
        self.accumulated[0] += actual[0];
        self.accumulated[1] += actual[1];
        self.accumulated[2] += actual[2];
        self.used += actual_mag;
        actual
    }

    pub fn reset(&mut self) {
        self.accumulated = [0.0; 3];
        self.used = 0.0;
    }

    pub fn remaining_budget(&self) -> f32 {
        (self.budget - self.used).max(0.0)
    }
}

/// Safe force magnitude for a body to not exceed `max_acceleration`.
#[allow(dead_code)]
pub fn safe_force(mass: f32, max_acceleration: f32) -> f32 {
    mass * max_acceleration
}

/// Check if two diagonal force components are below threshold.
#[allow(dead_code)]
pub fn force_within_limit(force: [f32; 3], limit: f32) -> bool {
    let mag_sq = force[0] * force[0] + force[1] * force[1] + force[2] * force[2];
    mag_sq <= limit * limit
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::SQRT_2;

    #[test]
    fn clamp_force_no_change_within_limit() {
        let f = clamp_force([1.0, 0.0, 0.0], 10.0);
        assert!((f[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn clamp_force_limits_magnitude() {
        let f = clamp_force([100.0, 0.0, 0.0], 5.0);
        let mag = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
        assert!((mag - 5.0).abs() < 1e-5);
    }

    #[test]
    fn clamp_force_diagonal() {
        // [SQRT_2, SQRT_2, 0] has magnitude 2; limit to 1
        let f = clamp_force([SQRT_2, SQRT_2, 0.0], 1.0);
        let mag = (f[0] * f[0] + f[1] * f[1]).sqrt();
        assert!((mag - 1.0).abs() < 1e-5);
    }

    #[test]
    fn limit_impulse_zero_dt() {
        let f = limit_impulse([100.0, 0.0, 0.0], 10.0, 0.0);
        assert_eq!(f, [0.0; 3]);
    }

    #[test]
    fn per_axis_limit_applies_per_axis() {
        let lim = PerAxisForceLimit {
            max_x: 1.0,
            max_y: 2.0,
            max_z: 3.0,
        };
        let f = lim.apply([5.0, -5.0, 2.0]);
        assert!((f[0] - 1.0).abs() < 1e-6);
        assert!((f[1] - (-2.0)).abs() < 1e-6);
        assert!((f[2] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn budgeted_accum_respects_budget() {
        let mut b = BudgetedForceAccum::new(10.0);
        b.add([8.0, 0.0, 0.0]);
        let actual = b.add([5.0, 0.0, 0.0]); // only 2 remaining
        let mag = actual[0].abs();
        assert!(mag <= 2.01);
    }

    #[test]
    fn budgeted_accum_reset() {
        let mut b = BudgetedForceAccum::new(10.0);
        b.add([5.0, 0.0, 0.0]);
        b.reset();
        assert_eq!(b.used, 0.0);
        assert_eq!(b.accumulated, [0.0; 3]);
    }

    #[test]
    fn safe_force_formula() {
        assert!((safe_force(2.0, 3.0) - 6.0).abs() < 1e-6);
    }

    #[test]
    fn force_within_limit_true() {
        assert!(force_within_limit([3.0, 4.0, 0.0], 5.1));
    }

    #[test]
    fn force_within_limit_false() {
        assert!(!force_within_limit([3.0, 4.0, 0.0], 4.9));
    }
}
