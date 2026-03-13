// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Bilateral constraint — equality constraint enforcing C(q) = 0.

/// A bilateral constraint (equality) between two bodies or a body and world.
#[derive(Debug, Clone)]
pub struct BilateralConstraint {
    pub body_a: usize,
    pub body_b: Option<usize>, /* None = world */
    pub jacobian_a: [f64; 6],  /* 6-DOF Jacobian row for body A */
    pub jacobian_b: [f64; 6],  /* 6-DOF Jacobian row for body B */
    pub bias: f64,             /* position correction bias */
    pub lambda: f64,           /* accumulated Lagrange multiplier */
    pub compliance: f64,       /* soft constraint compliance (0 = rigid) */
}

impl BilateralConstraint {
    /// Create a new bilateral constraint.
    pub fn new(
        body_a: usize,
        body_b: Option<usize>,
        jacobian_a: [f64; 6],
        jacobian_b: [f64; 6],
        bias: f64,
    ) -> Self {
        BilateralConstraint {
            body_a,
            body_b,
            jacobian_a,
            jacobian_b,
            bias,
            lambda: 0.0,
            compliance: 0.0,
        }
    }

    /// Compute the constraint velocity error J * v.
    pub fn velocity_error(&self, va: &[f64; 6], vb: &[f64; 6]) -> f64 {
        let jva: f64 = (0..6).map(|i| self.jacobian_a[i] * va[i]).sum();
        let jvb: f64 = (0..6).map(|i| self.jacobian_b[i] * vb[i]).sum();
        jva + jvb
    }

    /// Effective mass (J M^-1 J^T) for a diagonal mass matrix.
    pub fn effective_mass(
        &self,
        inv_mass_a: f64,
        inv_inertia_a: f64,
        inv_mass_b: f64,
        inv_inertia_b: f64,
    ) -> f64 {
        let trans_rot = |j: &[f64; 6], im: f64, ii: f64| -> f64 {
            im * (j[0] * j[0] + j[1] * j[1] + j[2] * j[2])
                + ii * (j[3] * j[3] + j[4] * j[4] + j[5] * j[5])
        };
        trans_rot(&self.jacobian_a, inv_mass_a, inv_inertia_a)
            + trans_rot(&self.jacobian_b, inv_mass_b, inv_inertia_b)
            + self.compliance
    }

    /// Solve for lambda increment and update lambda.
    pub fn solve(
        &mut self,
        va: &[f64; 6],
        vb: &[f64; 6],
        inv_mass_a: f64,
        inv_inertia_a: f64,
        inv_mass_b: f64,
        inv_inertia_b: f64,
    ) -> f64 {
        let jv = self.velocity_error(va, vb);
        let em = self.effective_mass(inv_mass_a, inv_inertia_a, inv_mass_b, inv_inertia_b);
        if em.abs() < 1e-12 {
            return 0.0;
        }
        let delta_lambda = -(jv + self.bias) / em;
        self.lambda += delta_lambda;
        delta_lambda
    }

    /// Reset accumulated impulse (for warm-starting purposes).
    pub fn reset_lambda(&mut self) {
        self.lambda = 0.0;
    }
}

/// Apply impulse to a velocity vector: v += inv_mass * J^T * lambda.
pub fn apply_impulse(
    vel: &mut [f64; 6],
    jacobian: &[f64; 6],
    lambda: f64,
    inv_mass: f64,
    inv_inertia: f64,
) {
    for k in 0..3 {
        vel[k] += inv_mass * jacobian[k] * lambda;
    }
    for k in 3..6 {
        vel[k] += inv_inertia * jacobian[k] * lambda;
    }
}

/// Compute constraint position violation given two positions.
pub fn position_violation(pos_a: [f64; 3], pos_b: [f64; 3], axis: [f64; 3]) -> f64 {
    let diff = [
        pos_a[0] - pos_b[0],
        pos_a[1] - pos_b[1],
        pos_a[2] - pos_b[2],
    ];
    diff[0] * axis[0] + diff[1] * axis[1] + diff[2] * axis[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_velocity_error_zero() {
        let c = BilateralConstraint::new(0, None, [1.0, 0.0, 0.0, 0.0, 0.0, 0.0], [0.0; 6], 0.0);
        let va = [0.0f64; 6];
        let vb = [0.0f64; 6];
        assert_eq!(
            c.velocity_error(&va, &vb),
            0.0 /* zero velocity => zero error */
        );
    }

    #[test]
    fn test_velocity_error_nonzero() {
        let c = BilateralConstraint::new(0, None, [1.0, 0.0, 0.0, 0.0, 0.0, 0.0], [0.0; 6], 0.0);
        let va = [2.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let vb = [0.0f64; 6];
        assert_eq!(c.velocity_error(&va, &vb), 2.0 /* Jv = 2 */);
    }

    #[test]
    fn test_effective_mass_positive() {
        let c = BilateralConstraint::new(0, None, [1.0, 0.0, 0.0, 0.0, 0.0, 0.0], [0.0; 6], 0.0);
        let em = c.effective_mass(1.0, 1.0, 0.0, 0.0);
        assert!(em > 0.0 /* effective mass is positive */);
    }

    #[test]
    fn test_solve_returns_delta_lambda() {
        let mut c =
            BilateralConstraint::new(0, None, [1.0, 0.0, 0.0, 0.0, 0.0, 0.0], [0.0; 6], 0.0);
        let va = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let vb = [0.0f64; 6];
        let dl = c.solve(&va, &vb, 1.0, 1.0, 0.0, 0.0);
        assert!(dl != 0.0 /* non-zero velocity gives non-zero delta lambda */);
    }

    #[test]
    fn test_reset_lambda() {
        let mut c = BilateralConstraint::new(0, None, [1.0; 6], [0.0; 6], 0.0);
        c.lambda = 5.0;
        c.reset_lambda();
        assert_eq!(c.lambda, 0.0 /* lambda reset to zero */);
    }

    #[test]
    fn test_apply_impulse() {
        let mut vel = [0.0f64; 6];
        let j = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        apply_impulse(&mut vel, &j, 2.0, 0.5, 1.0);
        assert!((vel[0] - 1.0).abs() < 1e-10 /* vel[0] += 0.5 * 1 * 2 = 1 */);
    }

    #[test]
    fn test_position_violation() {
        let pv = position_violation([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!((pv - 1.0).abs() < 1e-10 /* violation along x-axis */);
    }

    #[test]
    fn test_compliance_adds_to_mass() {
        let mut c =
            BilateralConstraint::new(0, None, [1.0, 0.0, 0.0, 0.0, 0.0, 0.0], [0.0; 6], 0.0);
        c.compliance = 0.1;
        let em_soft = c.effective_mass(1.0, 0.0, 0.0, 0.0);
        c.compliance = 0.0;
        let em_rigid = c.effective_mass(1.0, 0.0, 0.0, 0.0);
        assert!(em_soft > em_rigid /* compliance increases effective mass */);
    }

    #[test]
    fn test_world_body_b() {
        let c = BilateralConstraint::new(0, None, [0.0; 6], [0.0; 6], 0.0);
        assert!(c.body_b.is_none() /* None means world */);
    }
}
