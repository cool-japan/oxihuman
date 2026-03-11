#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// A contact solved using Projected Gauss-Seidel.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct PgsContact {
    normal: [f32; 3],
    penetration: f32,
    impulse: f32,
}

/// Projected Gauss-Seidel contact solver.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PgsContactSolver {
    contacts: Vec<PgsContact>,
    max_iterations: u32,
    tolerance: f32,
    residual: f32,
}

#[allow(dead_code)]
pub fn new_pgs_solver(max_iterations: u32, tolerance: f32) -> PgsContactSolver {
    PgsContactSolver {
        contacts: Vec::new(),
        max_iterations,
        tolerance,
        residual: 0.0,
    }
}

#[allow(dead_code)]
pub fn add_pgs_contact(solver: &mut PgsContactSolver, normal: [f32; 3], penetration: f32) {
    solver.contacts.push(PgsContact {
        normal,
        penetration,
        impulse: 0.0,
    });
}

#[allow(dead_code)]
pub fn solve_pgs(solver: &mut PgsContactSolver) -> u32 {
    let mut iterations = 0u32;
    for _ in 0..solver.max_iterations {
        iterations += 1;
        let mut max_delta = 0.0f32;
        for contact in &mut solver.contacts {
            let delta = contact.penetration * 0.5;
            let new_impulse = (contact.impulse + delta).max(0.0);
            let change = (new_impulse - contact.impulse).abs();
            if change > max_delta {
                max_delta = change;
            }
            contact.impulse = new_impulse;
            contact.penetration *= 0.5;
        }
        solver.residual = max_delta;
        if max_delta < solver.tolerance {
            break;
        }
    }
    iterations
}

#[allow(dead_code)]
pub fn pgs_iterations(solver: &PgsContactSolver) -> u32 {
    solver.max_iterations
}

#[allow(dead_code)]
pub fn pgs_residual(solver: &PgsContactSolver) -> f32 {
    solver.residual
}

#[allow(dead_code)]
pub fn pgs_contact_count(solver: &PgsContactSolver) -> usize {
    solver.contacts.len()
}

#[allow(dead_code)]
pub fn pgs_clear(solver: &mut PgsContactSolver) {
    solver.contacts.clear();
    solver.residual = 0.0;
}

#[allow(dead_code)]
pub fn pgs_converged(solver: &PgsContactSolver) -> bool {
    solver.residual < solver.tolerance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pgs_solver() {
        let s = new_pgs_solver(10, 0.001);
        assert_eq!(pgs_contact_count(&s), 0);
    }

    #[test]
    fn test_add_pgs_contact() {
        let mut s = new_pgs_solver(10, 0.001);
        add_pgs_contact(&mut s, [0.0, 1.0, 0.0], 0.1);
        assert_eq!(pgs_contact_count(&s), 1);
    }

    #[test]
    fn test_solve_pgs() {
        let mut s = new_pgs_solver(100, 0.0001);
        add_pgs_contact(&mut s, [0.0, 1.0, 0.0], 1.0);
        let iters = solve_pgs(&mut s);
        assert!(iters > 0);
    }

    #[test]
    fn test_pgs_iterations() {
        let s = new_pgs_solver(20, 0.001);
        assert_eq!(pgs_iterations(&s), 20);
    }

    #[test]
    fn test_pgs_residual() {
        let mut s = new_pgs_solver(100, 0.0001);
        add_pgs_contact(&mut s, [0.0, 1.0, 0.0], 1.0);
        solve_pgs(&mut s);
        assert!(pgs_residual(&s) < 1.0);
    }

    #[test]
    fn test_pgs_contact_count() {
        let mut s = new_pgs_solver(10, 0.001);
        add_pgs_contact(&mut s, [0.0, 1.0, 0.0], 0.1);
        add_pgs_contact(&mut s, [1.0, 0.0, 0.0], 0.2);
        assert_eq!(pgs_contact_count(&s), 2);
    }

    #[test]
    fn test_pgs_clear() {
        let mut s = new_pgs_solver(10, 0.001);
        add_pgs_contact(&mut s, [0.0, 1.0, 0.0], 0.1);
        pgs_clear(&mut s);
        assert_eq!(pgs_contact_count(&s), 0);
    }

    #[test]
    fn test_pgs_converged() {
        let mut s = new_pgs_solver(100, 0.01);
        add_pgs_contact(&mut s, [0.0, 1.0, 0.0], 0.001);
        solve_pgs(&mut s);
        assert!(pgs_converged(&s));
    }

    #[test]
    fn test_solve_empty() {
        let mut s = new_pgs_solver(10, 0.001);
        let iters = solve_pgs(&mut s);
        assert_eq!(iters, 1);
    }

    #[test]
    fn test_pgs_converged_initial() {
        let s = new_pgs_solver(10, 0.001);
        assert!(pgs_converged(&s));
    }
}
