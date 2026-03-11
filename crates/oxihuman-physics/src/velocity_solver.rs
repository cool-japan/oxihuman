#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Velocity-level constraint solver.

/// A velocity-level constraint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VelocityConstraint {
    pub jacobian: f32,
    pub bias: f32,
    pub lambda: f32,
    pub inv_mass: f32,
}

/// Solver for velocity constraints.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VelocitySolver {
    constraints: Vec<VelocityConstraint>,
    iterations: u32,
}

#[allow(dead_code)]
pub fn new_velocity_solver(iterations: u32) -> VelocitySolver {
    VelocitySolver {
        constraints: Vec::new(),
        iterations,
    }
}

#[allow(dead_code)]
pub fn add_velocity_constraint(
    solver: &mut VelocitySolver,
    jacobian: f32,
    bias: f32,
    inv_mass: f32,
) {
    solver.constraints.push(VelocityConstraint {
        jacobian,
        bias,
        lambda: 0.0,
        inv_mass,
    });
}

#[allow(dead_code)]
pub fn solve_velocities(solver: &mut VelocitySolver) -> f32 {
    let mut total_error = 0.0f32;
    for _ in 0..solver.iterations {
        total_error = 0.0;
        for c in &mut solver.constraints {
            let jv = c.jacobian * c.lambda;
            let residual = c.bias - jv;
            let eff_mass = if c.inv_mass > 0.0 {
                1.0 / (c.jacobian * c.jacobian * c.inv_mass)
            } else {
                0.0
            };
            let delta_lambda = residual * eff_mass;
            c.lambda += delta_lambda;
            total_error += residual.abs();
        }
    }
    total_error
}

#[allow(dead_code)]
pub fn velocity_iterations(solver: &VelocitySolver) -> u32 {
    solver.iterations
}

#[allow(dead_code)]
pub fn clear_velocity_solver(solver: &mut VelocitySolver) {
    solver.constraints.clear();
}

#[allow(dead_code)]
pub fn velocity_error(solver: &VelocitySolver) -> f32 {
    solver.constraints.iter().map(|c| (c.bias - c.jacobian * c.lambda).abs()).sum()
}

#[allow(dead_code)]
pub fn solver_converged(solver: &VelocitySolver, tolerance: f32) -> bool {
    velocity_error(solver) < tolerance
}

#[allow(dead_code)]
pub fn solver_residual(solver: &VelocitySolver) -> f32 {
    velocity_error(solver)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_solver() {
        let s = new_velocity_solver(10);
        assert_eq!(velocity_iterations(&s), 10);
    }

    #[test]
    fn test_add_constraint() {
        let mut s = new_velocity_solver(10);
        add_velocity_constraint(&mut s, 1.0, 0.5, 1.0);
        assert_eq!(s.constraints.len(), 1);
    }

    #[test]
    fn test_solve() {
        let mut s = new_velocity_solver(5);
        add_velocity_constraint(&mut s, 1.0, 0.1, 1.0);
        let err = solve_velocities(&mut s);
        let _ = err; // just ensure it runs
    }

    #[test]
    fn test_clear() {
        let mut s = new_velocity_solver(10);
        add_velocity_constraint(&mut s, 1.0, 0.0, 1.0);
        clear_velocity_solver(&mut s);
        assert_eq!(s.constraints.len(), 0);
    }

    #[test]
    fn test_velocity_error_no_constraints() {
        let s = new_velocity_solver(1);
        assert_eq!(velocity_error(&s), 0.0);
    }

    #[test]
    fn test_converged_empty() {
        let s = new_velocity_solver(1);
        assert!(solver_converged(&s, 0.01));
    }

    #[test]
    fn test_residual() {
        let mut s = new_velocity_solver(1);
        add_velocity_constraint(&mut s, 1.0, 1.0, 1.0);
        assert!(solver_residual(&s) > 0.0);
    }

    #[test]
    fn test_iterations() {
        let s = new_velocity_solver(42);
        assert_eq!(velocity_iterations(&s), 42);
    }

    #[test]
    fn test_solve_reduces_error() {
        let mut s = new_velocity_solver(10);
        add_velocity_constraint(&mut s, 1.0, 1.0, 1.0);
        let before = solver_residual(&s);
        solve_velocities(&mut s);
        let after = solver_residual(&s);
        assert!(after <= before);
    }

    #[test]
    fn test_zero_inv_mass() {
        let mut s = new_velocity_solver(5);
        add_velocity_constraint(&mut s, 1.0, 0.5, 0.0);
        solve_velocities(&mut s); // should not crash
    }
}
