// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Sequential impulse resolver — Gauss-Seidel style constraint solver.

/// A single constraint entry for the sequential impulse solver.
#[derive(Debug, Clone)]
pub struct ImpulseConstraint {
    pub body_a: usize,
    pub body_b: usize,
    pub jacobian_a: [f64; 6],
    pub jacobian_b: [f64; 6],
    pub rhs: f64,
    pub lambda_min: f64,
    pub lambda_max: f64,
    pub lambda: f64,
    pub effective_mass_inv: f64,
}

impl ImpulseConstraint {
    /// Create a new impulse constraint.
    pub fn new(
        body_a: usize,
        body_b: usize,
        jacobian_a: [f64; 6],
        jacobian_b: [f64; 6],
        rhs: f64,
        lambda_min: f64,
        lambda_max: f64,
    ) -> Self {
        ImpulseConstraint {
            body_a,
            body_b,
            jacobian_a,
            jacobian_b,
            rhs,
            lambda_min,
            lambda_max,
            lambda: 0.0,
            effective_mass_inv: 0.0,
        }
    }

    /// Pre-compute effective mass inverse for faster solving.
    pub fn precompute_mass(
        &mut self,
        inv_mass_a: f64,
        inv_inertia_a: f64,
        inv_mass_b: f64,
        inv_inertia_b: f64,
    ) {
        let tr = |j: &[f64; 6], im: f64, ii: f64| -> f64 {
            im * (j[0] * j[0] + j[1] * j[1] + j[2] * j[2])
                + ii * (j[3] * j[3] + j[4] * j[4] + j[5] * j[5])
        };
        let em = tr(&self.jacobian_a, inv_mass_a, inv_inertia_a)
            + tr(&self.jacobian_b, inv_mass_b, inv_inertia_b);
        self.effective_mass_inv = if em.abs() > 1e-12 { 1.0 / em } else { 0.0 };
    }

    /// Solve one Gauss-Seidel iteration.
    pub fn solve_gs(&mut self, vel_a: &[f64; 6], vel_b: &[f64; 6]) -> f64 {
        let jva: f64 = (0..6).map(|i| self.jacobian_a[i] * vel_a[i]).sum();
        let jvb: f64 = (0..6).map(|i| self.jacobian_b[i] * vel_b[i]).sum();
        let jv = jva + jvb;
        let delta = (self.rhs - jv) * self.effective_mass_inv;
        let old = self.lambda;
        self.lambda = (self.lambda + delta).clamp(self.lambda_min, self.lambda_max);
        self.lambda - old
    }
}

/// Sequential impulse solver.
pub struct SequentialImpulseSolver {
    pub constraints: Vec<ImpulseConstraint>,
    pub iterations: usize,
}

impl SequentialImpulseSolver {
    /// Create a new solver with a given iteration count.
    pub fn new(iterations: usize) -> Self {
        SequentialImpulseSolver {
            constraints: vec![],
            iterations,
        }
    }

    /// Add a constraint.
    pub fn add_constraint(&mut self, c: ImpulseConstraint) {
        self.constraints.push(c);
    }

    /// Run one full iteration over all constraints.
    pub fn iterate(&mut self, velocities_a: &[[f64; 6]], velocities_b: &[[f64; 6]]) -> f64 {
        let mut total_change = 0.0;
        for c in self.constraints.iter_mut() {
            let va = velocities_a.get(c.body_a).copied().unwrap_or([0.0; 6]);
            let vb = velocities_b.get(c.body_b).copied().unwrap_or([0.0; 6]);
            total_change += c.solve_gs(&va, &vb).abs();
        }
        total_change
    }

    /// Reset all lambdas.
    pub fn reset(&mut self) {
        for c in self.constraints.iter_mut() {
            c.lambda = 0.0;
        }
    }

    /// Total accumulated impulse.
    pub fn total_lambda(&self) -> f64 {
        self.constraints.iter().map(|c| c.lambda).sum()
    }

    /// Number of constraints.
    pub fn num_constraints(&self) -> usize {
        self.constraints.len()
    }
}

/// Compute velocity correction from a constraint impulse.
pub fn velocity_correction(
    jacobian: &[f64; 6],
    lambda: f64,
    inv_mass: f64,
    inv_inertia: f64,
) -> [f64; 6] {
    let mut dv = [0.0f64; 6];
    for k in 0..3 {
        dv[k] = inv_mass * jacobian[k] * lambda;
    }
    for k in 3..6 {
        dv[k] = inv_inertia * jacobian[k] * lambda;
    }
    dv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precompute_mass() {
        let mut c = ImpulseConstraint::new(
            0,
            1,
            [1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            0.0,
            f64::NEG_INFINITY,
            f64::INFINITY,
        );
        c.precompute_mass(1.0, 1.0, 1.0, 1.0);
        assert!(c.effective_mass_inv > 0.0 /* effective mass inverse positive */);
    }

    #[test]
    fn test_solve_gs_clamp_min() {
        let mut c = ImpulseConstraint::new(
            0,
            1,
            [1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0; 6],
            0.0,
            0.0,
            f64::INFINITY,
        );
        c.effective_mass_inv = 1.0;
        let va = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let vb = [0.0; 6];
        c.solve_gs(&va, &vb);
        assert!(c.lambda >= 0.0 /* clamped to min=0 */);
    }

    #[test]
    fn test_reset() {
        let mut s = SequentialImpulseSolver::new(10);
        let mut c = ImpulseConstraint::new(0, 1, [0.0; 6], [0.0; 6], 0.0, 0.0, f64::INFINITY);
        c.lambda = 5.0;
        s.add_constraint(c);
        s.reset();
        assert_eq!(s.total_lambda(), 0.0 /* reset to zero */);
    }

    #[test]
    fn test_num_constraints() {
        let mut s = SequentialImpulseSolver::new(10);
        s.add_constraint(ImpulseConstraint::new(
            0,
            1,
            [0.0; 6],
            [0.0; 6],
            0.0,
            0.0,
            f64::INFINITY,
        ));
        s.add_constraint(ImpulseConstraint::new(
            1,
            2,
            [0.0; 6],
            [0.0; 6],
            0.0,
            0.0,
            f64::INFINITY,
        ));
        assert_eq!(s.num_constraints(), 2);
    }

    #[test]
    fn test_total_lambda_sums() {
        let mut s = SequentialImpulseSolver::new(10);
        let mut c1 = ImpulseConstraint::new(0, 1, [0.0; 6], [0.0; 6], 0.0, 0.0, f64::INFINITY);
        let mut c2 = ImpulseConstraint::new(1, 2, [0.0; 6], [0.0; 6], 0.0, 0.0, f64::INFINITY);
        c1.lambda = 3.0;
        c2.lambda = 4.0;
        s.add_constraint(c1);
        s.add_constraint(c2);
        assert!((s.total_lambda() - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_velocity_correction() {
        let j = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let dv = velocity_correction(&j, 2.0, 0.5, 1.0);
        assert!((dv[0] - 1.0).abs() < 1e-10 /* 0.5 * 1 * 2 = 1 */);
    }

    #[test]
    fn test_iterate_returns_change() {
        let mut s = SequentialImpulseSolver::new(10);
        let mut c = ImpulseConstraint::new(
            0,
            0,
            [1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0; 6],
            1.0,
            0.0,
            f64::INFINITY,
        );
        c.precompute_mass(1.0, 0.0, 0.0, 0.0);
        s.add_constraint(c);
        let va = [[0.0f64; 6]; 1];
        let vb = [[0.0f64; 6]; 1];
        let change = s.iterate(&va, &vb);
        assert!(change >= 0.0 /* change is non-negative */);
    }

    #[test]
    fn test_lambda_clamp_max() {
        let mut c = ImpulseConstraint::new(0, 1, [0.0; 6], [0.0; 6], 100.0, 0.0, 5.0);
        c.effective_mass_inv = 1.0;
        let va = [0.0f64; 6];
        let vb = [0.0f64; 6];
        c.solve_gs(&va, &vb);
        assert!(c.lambda <= 5.0 /* clamped to max */);
    }

    #[test]
    fn test_new_solver_empty() {
        let s = SequentialImpulseSolver::new(5);
        assert_eq!(s.num_constraints(), 0 /* empty on creation */);
    }
}
