// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! BlendWeightSolver — iterative weight fitting for blend shapes.

#![allow(dead_code)]

/// Configuration for the iterative solver.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SolverConfig {
    pub max_iterations: u32,
    pub tolerance: f32,
    pub learning_rate: f32,
}

impl Default for SolverConfig {
    fn default() -> Self {
        SolverConfig { max_iterations: 100, tolerance: 1e-4, learning_rate: 0.01 }
    }
}

/// Solver state for blend weight optimisation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendWeightSolver {
    pub config: SolverConfig,
    pub weights: Vec<f32>,
}

/// Create a new solver with `n_weights` zero-initialised weights.
#[allow(dead_code)]
pub fn new_blend_weight_solver(n_weights: usize, config: SolverConfig) -> BlendWeightSolver {
    BlendWeightSolver { config, weights: vec![0.0; n_weights] }
}

/// Run one gradient-descent step toward `target` given `basis` columns.
/// Returns the updated weight vector.
#[allow(dead_code)]
pub fn solve_weights(solver: &mut BlendWeightSolver, basis: &[Vec<f32>], target: &[f32]) -> Vec<f32> {
    let lr = solver.config.learning_rate;
    for _ in 0..solver.config.max_iterations {
        let residuals = weight_residual(solver, basis, target);
        let err: f32 = residuals.iter().map(|r| r * r).sum::<f32>().sqrt();
        if err < solver.config.tolerance {
            break;
        }
        for (j, col) in basis.iter().enumerate() {
            if j >= solver.weights.len() {
                break;
            }
            let grad: f32 = col.iter().zip(residuals.iter()).map(|(c, r)| c * r).sum();
            solver.weights[j] -= lr * grad;
        }
    }
    solver.weights.clone()
}

/// Compute the RMS error between the current weighted sum and `target`.
#[allow(dead_code)]
pub fn weight_error(solver: &BlendWeightSolver, basis: &[Vec<f32>], target: &[f32]) -> f32 {
    let res = weight_residual(solver, basis, target);
    let mse: f32 = res.iter().map(|r| r * r).sum::<f32>() / res.len().max(1) as f32;
    mse.sqrt()
}

/// Project each weight into [0, 1] and re-normalise so they sum to 1.
#[allow(dead_code)]
pub fn normalize_blend_weights(weights: &mut [f32]) {
    for w in weights.iter_mut() {
        *w = w.clamp(0.0, 1.0);
    }
    let sum: f32 = weights.iter().sum();
    if sum > f32::EPSILON {
        for w in weights.iter_mut() {
            *w /= sum;
        }
    }
}

/// Clamp each weight to [lo, hi].
#[allow(dead_code)]
pub fn clamp_blend_weights(weights: &mut [f32], lo: f32, hi: f32) {
    for w in weights.iter_mut() {
        *w = w.clamp(lo, hi);
    }
}

/// Return the number of weights in the solver.
#[allow(dead_code)]
pub fn weight_count(solver: &BlendWeightSolver) -> usize {
    solver.weights.len()
}

/// Run the solver with non-negativity constraints.
#[allow(dead_code)]
pub fn solve_constrained(solver: &mut BlendWeightSolver, basis: &[Vec<f32>], target: &[f32]) -> Vec<f32> {
    solve_weights(solver, basis, target);
    clamp_blend_weights(&mut solver.weights, 0.0, 1.0);
    solver.weights.clone()
}

/// Compute the residual vector: target - Basis * weights.
#[allow(dead_code)]
pub fn weight_residual(solver: &BlendWeightSolver, basis: &[Vec<f32>], target: &[f32]) -> Vec<f32> {
    let n = target.len();
    let mut res: Vec<f32> = target.to_vec();
    for (j, col) in basis.iter().enumerate() {
        if j >= solver.weights.len() {
            break;
        }
        let w = solver.weights[j];
        for (i, c) in col.iter().enumerate() {
            if i < n {
                res[i] -= c * w;
            }
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_solver_zero_weights() {
        let s = new_blend_weight_solver(4, SolverConfig::default());
        assert_eq!(weight_count(&s), 4);
        assert!(s.weights.iter().all(|&w| w == 0.0));
    }

    #[test]
    fn test_weight_count() {
        let s = new_blend_weight_solver(3, SolverConfig::default());
        assert_eq!(weight_count(&s), 3);
    }

    #[test]
    fn test_clamp_blend_weights() {
        let mut w = vec![-0.5, 0.5, 1.5];
        clamp_blend_weights(&mut w, 0.0, 1.0);
        assert_eq!(w, vec![0.0, 0.5, 1.0]);
    }

    #[test]
    fn test_normalize_blend_weights_sums_to_one() {
        let mut w = vec![1.0, 1.0, 2.0];
        normalize_blend_weights(&mut w);
        let sum: f32 = w.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_blend_weights_all_zero() {
        let mut w = vec![0.0, 0.0];
        normalize_blend_weights(&mut w);
        // Should not panic; weights stay 0
        let sum: f32 = w.iter().sum();
        assert!(sum.abs() < 1e-6);
    }

    #[test]
    fn test_weight_residual_zero_weights() {
        let s = new_blend_weight_solver(1, SolverConfig::default());
        let basis = vec![vec![1.0, 0.0]];
        let target = vec![3.0, 4.0];
        let res = weight_residual(&s, &basis, &target);
        assert!((res[0] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_weight_error_nonzero() {
        let s = new_blend_weight_solver(1, SolverConfig::default());
        let basis = vec![vec![1.0]];
        let target = vec![1.0];
        let e = weight_error(&s, &basis, &target);
        assert!(e > 0.0);
    }

    #[test]
    fn test_solve_constrained_nonnegative() {
        let mut s = new_blend_weight_solver(2, SolverConfig::default());
        let basis = vec![vec![1.0], vec![0.0]];
        let target = vec![0.5];
        let w = solve_constrained(&mut s, &basis, &target);
        assert!(w.iter().all(|&x| x >= 0.0));
    }

    #[test]
    fn test_solver_config_default() {
        let c = SolverConfig::default();
        assert!(c.max_iterations > 0);
        assert!(c.tolerance > 0.0);
    }
}
