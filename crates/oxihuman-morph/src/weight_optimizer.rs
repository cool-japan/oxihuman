// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Least-squares blend weight optimization via projected gradient descent.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightOptimizer {
    pub max_iterations: u32,
    pub learning_rate: f32,
    pub convergence_eps: f32,
    pub weight_min: f32,
    pub weight_max: f32,
}

impl Default for WeightOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub weights: Vec<f32>,
    pub final_error: f32,
    pub iterations: u32,
    pub converged: bool,
}

impl WeightOptimizer {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            max_iterations: 200,
            learning_rate: 0.01,
            convergence_eps: 1e-5,
            weight_min: 0.0,
            weight_max: 1.0,
        }
    }

    #[allow(dead_code)]
    pub fn optimize(
        &self,
        base: &[[f32; 3]],
        target: &[[f32; 3]],
        morph_deltas: &[Vec<[f32; 3]>],
    ) -> OptimizationResult {
        let k = morph_deltas.len();
        if k == 0 {
            let error = reconstruction_error(base, target, morph_deltas, &[]);
            return OptimizationResult {
                weights: vec![],
                final_error: error,
                iterations: 0,
                converged: true,
            };
        }

        let mut weights = vec![0.0f32; k];
        let mut converged = false;
        let mut iters = 0u32;

        for iter in 0..self.max_iterations {
            iters = iter + 1;
            let grad = gradient_wrt_weights(base, target, morph_deltas, &weights);
            let grad_norm: f32 = grad.iter().map(|g| g * g).sum::<f32>().sqrt();

            for i in 0..k {
                weights[i] -= self.learning_rate * grad[i];
            }
            clamp_weights(&mut weights, self.weight_min, self.weight_max);

            if grad_norm < self.convergence_eps {
                converged = true;
                break;
            }
        }

        let final_error = reconstruction_error(base, target, morph_deltas, &weights);
        OptimizationResult {
            weights,
            final_error,
            iterations: iters,
            converged,
        }
    }
}

#[allow(dead_code)]
pub fn reconstruction_error(
    base: &[[f32; 3]],
    target: &[[f32; 3]],
    deltas: &[Vec<[f32; 3]>],
    weights: &[f32],
) -> f32 {
    if base.is_empty() {
        return 0.0;
    }
    let blended = apply_weights(base, deltas, weights);
    let n = base.len() as f32;
    blended
        .iter()
        .zip(target.iter())
        .map(|(b, t)| {
            let dx = b[0] - t[0];
            let dy = b[1] - t[1];
            let dz = b[2] - t[2];
            dx * dx + dy * dy + dz * dz
        })
        .sum::<f32>()
        / n
}

#[allow(dead_code)]
pub fn gradient_wrt_weights(
    base: &[[f32; 3]],
    target: &[[f32; 3]],
    deltas: &[Vec<[f32; 3]>],
    weights: &[f32],
) -> Vec<f32> {
    let k = weights.len();
    let n = base.len();
    if n == 0 || k == 0 {
        return vec![0.0; k];
    }

    let blended = apply_weights(base, deltas, weights);
    let scale = 2.0 / n as f32;

    (0..k)
        .map(|i| {
            let delta_i = &deltas[i];
            let dlen = delta_i.len().min(n);
            let mut g = 0.0f32;
            for v in 0..dlen {
                let rx = blended[v][0] - target[v][0];
                let ry = blended[v][1] - target[v][1];
                let rz = blended[v][2] - target[v][2];
                g += rx * delta_i[v][0] + ry * delta_i[v][1] + rz * delta_i[v][2];
            }
            g * scale
        })
        .collect()
}

#[allow(dead_code)]
pub fn apply_weights(
    base: &[[f32; 3]],
    deltas: &[Vec<[f32; 3]>],
    weights: &[f32],
) -> Vec<[f32; 3]> {
    let n = base.len();
    let mut result: Vec<[f32; 3]> = base.to_vec();
    for (i, w) in weights.iter().enumerate() {
        if i >= deltas.len() {
            break;
        }
        let d = &deltas[i];
        let dlen = d.len().min(n);
        for v in 0..dlen {
            result[v][0] += w * d[v][0];
            result[v][1] += w * d[v][1];
            result[v][2] += w * d[v][2];
        }
    }
    result
}

#[allow(dead_code)]
pub fn clamp_weights(weights: &mut [f32], min: f32, max: f32) {
    for w in weights.iter_mut() {
        *w = w.clamp(min, max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_base(n: usize) -> Vec<[f32; 3]> {
        (0..n).map(|i| [i as f32, 0.0, 0.0]).collect()
    }

    #[test]
    fn test_apply_weights_no_deltas() {
        let base = make_base(3);
        let result = apply_weights(&base, &[], &[]);
        assert_eq!(result, base);
    }

    #[test]
    fn test_apply_weights_single() {
        let base = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let deltas = vec![vec![[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]]];
        let weights = [0.5f32];
        let result = apply_weights(&base, &deltas, &weights);
        assert!((result[0][0] - 0.5).abs() < 1e-6);
        assert!((result[1][0] - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_weights() {
        let mut w = vec![-0.5, 0.5, 1.5, 0.0];
        clamp_weights(&mut w, 0.0, 1.0);
        assert!((w[0] - 0.0).abs() < 1e-6);
        assert!((w[1] - 0.5).abs() < 1e-6);
        assert!((w[2] - 1.0).abs() < 1e-6);
        assert!((w[3] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_reconstruction_error_zero() {
        let base = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let target = base.clone();
        let err = reconstruction_error(&base, &target, &[], &[]);
        assert!(err.abs() < 1e-6, "identical base/target: {err}");
    }

    #[test]
    fn test_reconstruction_error_nonzero() {
        let base = vec![[0.0, 0.0, 0.0]];
        let target = vec![[1.0, 0.0, 0.0]];
        let err = reconstruction_error(&base, &target, &[], &[]);
        assert!((err - 1.0).abs() < 1e-6, "error = 1.0: {err}");
    }

    #[test]
    fn test_gradient_direction() {
        // If blended > target on x, gradient for that morph should be positive
        let base = vec![[0.0, 0.0, 0.0]];
        let target = vec![[1.0, 0.0, 0.0]];
        let deltas = vec![vec![[1.0, 0.0, 0.0]]];
        let weights = [0.0f32];
        let grad = gradient_wrt_weights(&base, &target, &deltas, &weights);
        // residual = blended - target = -1.0 => gradient should be negative => step toward target
        assert!(
            grad[0] < 0.0,
            "gradient should be negative to increase weight: {}",
            grad[0]
        );
    }

    #[test]
    fn test_gradient_zero_at_perfect_fit() {
        let base = vec![[0.0, 0.0, 0.0]];
        let target = vec![[1.0, 0.0, 0.0]];
        let deltas = vec![vec![[1.0, 0.0, 0.0]]];
        let weights = [1.0f32];
        let grad = gradient_wrt_weights(&base, &target, &deltas, &weights);
        assert!(
            grad[0].abs() < 1e-6,
            "at perfect fit gradient is zero: {}",
            grad[0]
        );
    }

    #[test]
    fn test_single_target_perfect_fit() {
        // Single morph that exactly spans base→target; optimizer should find weight≈1
        let n = 4;
        let base: Vec<[f32; 3]> = (0..n).map(|i| [i as f32, 0.0, 0.0]).collect();
        let target: Vec<[f32; 3]> = (0..n).map(|i| [i as f32 + 1.0, 0.0, 0.0]).collect();
        let deltas = vec![(0..n).map(|_| [1.0f32, 0.0, 0.0]).collect::<Vec<_>>()];
        let opt = WeightOptimizer {
            max_iterations: 1000,
            learning_rate: 0.1,
            convergence_eps: 1e-6,
            weight_min: 0.0,
            weight_max: 1.0,
        };
        let result = opt.optimize(&base, &target, &deltas);
        assert!(
            (result.weights[0] - 1.0).abs() < 0.01,
            "should converge to 1.0, got {}",
            result.weights[0]
        );
        assert!(result.final_error < 1e-4);
    }

    #[test]
    fn test_zero_target_weight_stays_zero() {
        // Target == base → zero delta needed → weight stays 0.0
        let n = 3;
        let base: Vec<[f32; 3]> = (0..n).map(|i| [i as f32, 0.0, 0.0]).collect();
        let target = base.clone();
        let deltas = vec![(0..n).map(|_| [1.0f32, 0.0, 0.0]).collect::<Vec<_>>()];
        let opt = WeightOptimizer::new();
        let result = opt.optimize(&base, &target, &deltas);
        assert!(
            result.weights[0] < 0.01,
            "weight should stay near 0: {}",
            result.weights[0]
        );
    }

    #[test]
    fn test_empty_deltas() {
        let base = vec![[0.0, 0.0, 0.0]];
        let target = vec![[1.0, 0.0, 0.0]];
        let opt = WeightOptimizer::new();
        let result = opt.optimize(&base, &target, &[]);
        assert_eq!(result.weights.len(), 0);
        assert!(result.converged);
    }

    #[test]
    fn test_convergence_flag() {
        let base = vec![[0.0f32, 0.0, 0.0]];
        let target = vec![[0.0f32, 0.0, 0.0]];
        let deltas = vec![vec![[1.0f32, 0.0, 0.0]]];
        let opt = WeightOptimizer {
            max_iterations: 500,
            learning_rate: 0.1,
            convergence_eps: 1e-5,
            weight_min: 0.0,
            weight_max: 1.0,
        };
        let result = opt.optimize(&base, &target, &deltas);
        assert!(result.converged, "should converge when target==base");
    }

    #[test]
    fn test_reconstruction_error_formula() {
        // MSE = sum(||blended - target||^2) / n
        let base = vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let target = vec![[2.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        // (2^2 + 2^2) / 2 = 4.0
        let err = reconstruction_error(&base, &target, &[], &[]);
        assert!((err - 4.0).abs() < 1e-5, "err={err}");
    }

    #[test]
    fn test_multiple_morphs_add() {
        let base = vec![[0.0, 0.0, 0.0]];
        let target = vec![[2.0, 0.0, 0.0]];
        let d1 = vec![vec![[1.0f32, 0.0, 0.0]]];
        let d2 = vec![vec![[1.0f32, 0.0, 0.0]], vec![[1.0f32, 0.0, 0.0]]];
        let opt = WeightOptimizer {
            max_iterations: 2000,
            learning_rate: 0.05,
            convergence_eps: 1e-6,
            weight_min: 0.0,
            weight_max: 1.0,
        };
        let r1 = opt.optimize(&base, &target, &d1);
        let r2 = opt.optimize(&base, &target, &d2);
        // single morph needs weight=2 but clamped to 1; two morphs can each have weight=1
        assert!(
            r2.final_error < r1.final_error + 0.01,
            "two morphs should fit better: {} vs {}",
            r2.final_error,
            r1.final_error
        );
    }
}
