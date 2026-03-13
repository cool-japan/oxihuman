// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Position-based distance / length constraint for PBD solvers.
//!
//! Given two particles `a` and `b` with positions `pa` and `pb`, the
//! constraint enforces `|pa - pb| = rest_length`.  The correction is split
//! proportionally to the inverse masses of the two particles.

#![allow(dead_code)]

/// Configuration for a length constraint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LengthConstraintConfig {
    /// Rest (target) length.
    pub rest_length: f32,
    /// XPBD compliance (α): 0 = fully rigid, > 0 = elastic.
    pub compliance: f32,
    /// Inverse mass of particle A (0 = static).
    pub inv_mass_a: f32,
    /// Inverse mass of particle B (0 = static).
    pub inv_mass_b: f32,
}

/// Returns a sensible default [`LengthConstraintConfig`].
#[allow(dead_code)]
pub fn default_length_constraint_config() -> LengthConstraintConfig {
    LengthConstraintConfig {
        rest_length: 1.0,
        compliance: 0.0,
        inv_mass_a: 1.0,
        inv_mass_b: 1.0,
    }
}

/// A length constraint between two particles.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LengthConstraint {
    /// Particle A position.
    pub pa: [f32; 3],
    /// Particle B position.
    pub pb: [f32; 3],
    /// Accumulated XPBD Lagrange multiplier.
    pub lambda: f32,
    /// Configuration.
    pub config: LengthConstraintConfig,
}

/// Result of a single constraint solve.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct LengthConstraintResult {
    /// Correction applied to particle A.
    pub delta_a: [f32; 3],
    /// Correction applied to particle B.
    pub delta_b: [f32; 3],
    /// Constraint error (|pa - pb| - rest_length) before correction.
    pub error: f32,
    /// Whether the constraint was satisfied before solving.
    pub was_satisfied: bool,
}

/// Create a new [`LengthConstraint`].
#[allow(dead_code)]
pub fn new_length_constraint(
    pa: [f32; 3],
    pb: [f32; 3],
    config: LengthConstraintConfig,
) -> LengthConstraint {
    LengthConstraint { pa, pb, lambda: 0.0, config }
}

/// Constraint error = current length − rest length.
#[allow(dead_code)]
pub fn length_constraint_error(c: &LengthConstraint) -> f32 {
    let dx = c.pb[0]-c.pa[0];
    let dy = c.pb[1]-c.pa[1];
    let dz = c.pb[2]-c.pa[2];
    let len = (dx*dx + dy*dy + dz*dz).sqrt();
    len - c.config.rest_length
}

/// Whether the constraint is satisfied within floating-point tolerance.
#[allow(dead_code)]
pub fn length_constraint_is_satisfied(c: &LengthConstraint, tol: f32) -> bool {
    length_constraint_error(c).abs() < tol
}

/// Stiffness of the constraint (inverse of compliance; clamped to finite value).
#[allow(dead_code)]
pub fn length_constraint_stiffness(c: &LengthConstraint) -> f32 {
    if c.config.compliance < 1e-12 { f32::MAX } else { 1.0 / c.config.compliance }
}

/// XPBD compliance value.
#[allow(dead_code)]
pub fn length_constraint_compliance(c: &LengthConstraint) -> f32 {
    c.config.compliance
}

/// Compute the position corrections without applying them.
#[allow(dead_code)]
pub fn length_constraint_delta(c: &LengthConstraint) -> ([f32; 3], [f32; 3]) {
    let dx = c.pb[0]-c.pa[0];
    let dy = c.pb[1]-c.pa[1];
    let dz = c.pb[2]-c.pa[2];
    let len = (dx*dx + dy*dy + dz*dz).sqrt();
    if len < 1e-12 { return ([0.0;3], [0.0;3]); }
    let n = [dx/len, dy/len, dz/len];
    let err = len - c.config.rest_length;
    let w = c.config.inv_mass_a + c.config.inv_mass_b;
    if w < 1e-12 { return ([0.0;3], [0.0;3]); }
    let scale = err / w;
    let da = [c.config.inv_mass_a * scale * n[0],
              c.config.inv_mass_a * scale * n[1],
              c.config.inv_mass_a * scale * n[2]];
    let db = [-c.config.inv_mass_b * scale * n[0],
              -c.config.inv_mass_b * scale * n[1],
              -c.config.inv_mass_b * scale * n[2]];
    (da, db)
}

/// Solve the constraint: modify particle positions and return the result.
#[allow(dead_code)]
pub fn length_constraint_solve(c: &mut LengthConstraint, dt: f32) -> LengthConstraintResult {
    let err = length_constraint_error(c);
    let was_satisfied = err.abs() < 1e-6;

    let dx = c.pb[0]-c.pa[0];
    let dy = c.pb[1]-c.pa[1];
    let dz = c.pb[2]-c.pa[2];
    let len = (dx*dx + dy*dy + dz*dz).sqrt();

    if len < 1e-12 {
        return LengthConstraintResult {
            delta_a: [0.0;3], delta_b: [0.0;3], error: err, was_satisfied,
        };
    }
    let n = [dx/len, dy/len, dz/len];
    let w = c.config.inv_mass_a + c.config.inv_mass_b;
    if w < 1e-12 {
        return LengthConstraintResult {
            delta_a: [0.0;3], delta_b: [0.0;3], error: err, was_satisfied,
        };
    }

    // XPBD: α_tilde = compliance / (dt*dt)
    let alpha_tilde = c.config.compliance / (dt * dt).max(1e-12);
    let d_lambda = (-err - alpha_tilde * c.lambda) / (w + alpha_tilde);
    c.lambda += d_lambda;

    let da = [-c.config.inv_mass_a * d_lambda * n[0],
              -c.config.inv_mass_a * d_lambda * n[1],
              -c.config.inv_mass_a * d_lambda * n[2]];
    let db = [c.config.inv_mass_b * d_lambda * n[0],
              c.config.inv_mass_b * d_lambda * n[1],
              c.config.inv_mass_b * d_lambda * n[2]];

    c.pa[0] += da[0]; c.pa[1] += da[1]; c.pa[2] += da[2];
    c.pb[0] += db[0]; c.pb[1] += db[1]; c.pb[2] += db[2];

    LengthConstraintResult { delta_a: da, delta_b: db, error: err, was_satisfied }
}

/// Set a new rest length.
#[allow(dead_code)]
pub fn length_constraint_set_rest_length(c: &mut LengthConstraint, rest: f32) {
    c.config.rest_length = rest.max(0.0);
}

/// Reset the Lagrange multiplier (call at the start of each PBD frame).
#[allow(dead_code)]
pub fn length_constraint_reset(c: &mut LengthConstraint) {
    c.lambda = 0.0;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_constraint(pa: [f32;3], pb: [f32;3], rest: f32) -> LengthConstraint {
        let mut cfg = default_length_constraint_config();
        cfg.rest_length = rest;
        new_length_constraint(pa, pb, cfg)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_length_constraint_config();
        assert!(cfg.rest_length > 0.0);
        assert!(cfg.compliance >= 0.0);
    }

    #[test]
    fn test_error_satisfied() {
        let c = make_constraint([0.0,0.0,0.0], [1.0,0.0,0.0], 1.0);
        assert!(length_constraint_error(&c).abs() < 1e-5);
        assert!(length_constraint_is_satisfied(&c, 1e-4));
    }

    #[test]
    fn test_error_unsatisfied() {
        let c = make_constraint([0.0,0.0,0.0], [2.0,0.0,0.0], 1.0);
        assert!((length_constraint_error(&c) - 1.0).abs() < 1e-5);
        assert!(!length_constraint_is_satisfied(&c, 1e-4));
    }

    #[test]
    fn test_solve_reduces_error() {
        let mut c = make_constraint([0.0,0.0,0.0], [2.0,0.0,0.0], 1.0);
        let before = length_constraint_error(&c).abs();
        length_constraint_solve(&mut c, 0.016);
        let after = length_constraint_error(&c).abs();
        assert!(after < before, "error should decrease after solve");
    }

    #[test]
    fn test_solve_rigid_satisfies() {
        let mut c = make_constraint([0.0,0.0,0.0], [2.0,0.0,0.0], 1.0);
        for _ in 0..10 {
            length_constraint_solve(&mut c, 0.016);
        }
        assert!(length_constraint_is_satisfied(&c, 1e-3));
    }

    #[test]
    fn test_static_particle_no_move() {
        let mut cfg = default_length_constraint_config();
        cfg.rest_length = 1.0;
        cfg.inv_mass_a = 0.0; // static
        cfg.inv_mass_b = 1.0;
        let mut c = new_length_constraint([0.0,0.0,0.0], [2.0,0.0,0.0], cfg);
        length_constraint_solve(&mut c, 0.016);
        // A should not move.
        assert!((c.pa[0]).abs() < 1e-9);
    }

    #[test]
    fn test_set_rest_length() {
        let mut c = make_constraint([0.0,0.0,0.0], [1.0,0.0,0.0], 1.0);
        length_constraint_set_rest_length(&mut c, 0.5);
        assert!((c.config.rest_length - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_reset_lambda() {
        let mut c = make_constraint([0.0,0.0,0.0], [2.0,0.0,0.0], 1.0);
        length_constraint_solve(&mut c, 0.016);
        length_constraint_reset(&mut c);
        assert_eq!(c.lambda, 0.0);
    }

    #[test]
    fn test_stiffness_infinite_rigid() {
        let c = make_constraint([0.0,0.0,0.0], [1.0,0.0,0.0], 1.0);
        assert_eq!(length_constraint_stiffness(&c), f32::MAX);
    }

    #[test]
    fn test_compliance_value() {
        let mut cfg = default_length_constraint_config();
        cfg.compliance = 0.01;
        let c = new_length_constraint([0.0,0.0,0.0], [1.0,0.0,0.0], cfg);
        assert!((length_constraint_compliance(&c) - 0.01).abs() < 1e-9);
    }

    #[test]
    fn test_delta_direction() {
        let c = make_constraint([0.0,0.0,0.0], [2.0,0.0,0.0], 1.0);
        let (da, db) = length_constraint_delta(&c);
        // A should move toward B (positive X) and B toward A (negative X).
        assert!(da[0] > 0.0);
        assert!(db[0] < 0.0);
    }
}
