// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Solver convergence statistics (residual, iterations).

/// Statistics for a single solve.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct SolveStats {
    pub iterations: usize,
    pub initial_residual: f32,
    pub final_residual: f32,
    pub converged: bool,
    pub time_ms: f32,
}

impl SolveStats {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Convergence ratio (final / initial residual).
    #[allow(dead_code)]
    pub fn convergence_ratio(&self) -> f32 {
        if self.initial_residual < 1e-15 {
            return 0.0;
        }
        self.final_residual / self.initial_residual
    }
}

/// Accumulated solver statistics over many solves.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct SolverStatistics {
    pub total_solves: usize,
    pub total_iterations: usize,
    pub converged_count: usize,
    pub total_time_ms: f32,
    pub max_residual: f32,
    pub min_residual: f32,
    pub sum_residual: f32,
    history: Vec<SolveStats>,
}

impl SolverStatistics {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            min_residual: f32::INFINITY,
            ..Default::default()
        }
    }

    /// Record a solve result.
    #[allow(dead_code)]
    pub fn record(&mut self, stats: SolveStats) {
        self.total_solves += 1;
        self.total_iterations += stats.iterations;
        if stats.converged {
            self.converged_count += 1;
        }
        self.total_time_ms += stats.time_ms;
        self.max_residual = self.max_residual.max(stats.final_residual);
        self.min_residual = self.min_residual.min(stats.final_residual);
        self.sum_residual += stats.final_residual;
        self.history.push(stats);
    }

    /// Mean number of iterations per solve.
    #[allow(dead_code)]
    pub fn mean_iterations(&self) -> f32 {
        if self.total_solves == 0 {
            return 0.0;
        }
        self.total_iterations as f32 / self.total_solves as f32
    }

    /// Mean final residual.
    #[allow(dead_code)]
    pub fn mean_residual(&self) -> f32 {
        if self.total_solves == 0 {
            return 0.0;
        }
        self.sum_residual / self.total_solves as f32
    }

    /// Convergence rate (fraction that converged).
    #[allow(dead_code)]
    pub fn convergence_rate(&self) -> f32 {
        if self.total_solves == 0 {
            return 0.0;
        }
        self.converged_count as f32 / self.total_solves as f32
    }

    /// Reset all statistics.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Access solve history.
    #[allow(dead_code)]
    pub fn history(&self) -> &[SolveStats] {
        &self.history
    }

    /// Last recorded stats.
    #[allow(dead_code)]
    pub fn last(&self) -> Option<&SolveStats> {
        self.history.last()
    }
}

/// Compute the L2 residual of Ax - b.
#[allow(dead_code)]
pub fn compute_residual(a: &[f32], x: &[f32], b: &[f32], n: usize) -> f32 {
    if n == 0 {
        return 0.0;
    }
    let mut sum = 0.0f32;
    for i in 0..n {
        let mut ax_i = 0.0f32;
        for j in 0..n {
            ax_i += a[i * n + j] * x[j];
        }
        let r = ax_i - b[i];
        sum += r * r;
    }
    sum.sqrt()
}

/// Run a simple Jacobi iteration and return stats.
#[allow(dead_code)]
pub fn jacobi_solve_stats(a: &[f32], b: &[f32], n: usize, max_iter: usize, tol: f32) -> SolveStats {
    let mut x = vec![0.0f32; n];
    let init_res = compute_residual(a, &x, b, n);
    let mut final_res = init_res;
    let mut iters = 0;
    for _ in 0..max_iter {
        let mut x_new = vec![0.0f32; n];
        for i in 0..n {
            let mut sum = b[i];
            for j in 0..n {
                if j != i {
                    sum -= a[i * n + j] * x[j];
                }
            }
            let diag = a[i * n + i];
            if diag.abs() > 1e-10 {
                x_new[i] = sum / diag;
            }
        }
        x = x_new;
        final_res = compute_residual(a, &x, b, n);
        iters += 1;
        if final_res < tol {
            break;
        }
    }
    SolveStats {
        iterations: iters,
        initial_residual: init_res,
        final_residual: final_res,
        converged: final_res < tol,
        time_ms: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_increments_solve_count() {
        let mut stats = SolverStatistics::new();
        stats.record(SolveStats {
            iterations: 5,
            initial_residual: 1.0,
            final_residual: 0.001,
            converged: true,
            time_ms: 0.5,
        });
        assert_eq!(stats.total_solves, 1);
    }

    #[test]
    fn mean_iterations_correct() {
        let mut stats = SolverStatistics::new();
        stats.record(SolveStats {
            iterations: 4,
            initial_residual: 1.0,
            final_residual: 0.001,
            converged: true,
            time_ms: 0.0,
        });
        stats.record(SolveStats {
            iterations: 6,
            initial_residual: 1.0,
            final_residual: 0.001,
            converged: true,
            time_ms: 0.0,
        });
        assert!((stats.mean_iterations() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn convergence_rate_all_converged() {
        let mut stats = SolverStatistics::new();
        for _ in 0..5 {
            stats.record(SolveStats {
                iterations: 1,
                initial_residual: 1.0,
                final_residual: 0.0,
                converged: true,
                time_ms: 0.0,
            });
        }
        assert!((stats.convergence_rate() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn reset_clears_all() {
        let mut stats = SolverStatistics::new();
        stats.record(SolveStats {
            iterations: 1,
            initial_residual: 1.0,
            final_residual: 0.5,
            converged: false,
            time_ms: 0.0,
        });
        stats.reset();
        assert_eq!(stats.total_solves, 0);
    }

    #[test]
    fn jacobi_solve_simple_diagonal() {
        let a = vec![2.0f32, 0.0, 0.0, 2.0];
        let b = vec![4.0f32, 6.0];
        let s = jacobi_solve_stats(&a, &b, 2, 20, 1e-5);
        assert!(s.converged);
    }

    #[test]
    fn residual_zero_for_exact_solution() {
        let a = vec![1.0f32, 0.0, 0.0, 1.0];
        let x = vec![3.0f32, 5.0];
        let b = vec![3.0f32, 5.0];
        let r = compute_residual(&a, &x, &b, 2);
        assert!(r < 1e-5);
    }

    #[test]
    fn solve_stats_convergence_ratio() {
        let s = SolveStats {
            iterations: 5,
            initial_residual: 1.0,
            final_residual: 0.1,
            converged: true,
            time_ms: 0.0,
        };
        assert!((s.convergence_ratio() - 0.1).abs() < 1e-5);
    }

    #[test]
    fn last_returns_last_recorded() {
        let mut stats = SolverStatistics::new();
        stats.record(SolveStats {
            iterations: 2,
            initial_residual: 1.0,
            final_residual: 0.5,
            converged: false,
            time_ms: 0.0,
        });
        assert!(stats.last().is_some());
    }

    #[test]
    fn mean_residual_correct() {
        let mut stats = SolverStatistics::new();
        stats.record(SolveStats {
            iterations: 1,
            initial_residual: 1.0,
            final_residual: 0.2,
            converged: true,
            time_ms: 0.0,
        });
        stats.record(SolveStats {
            iterations: 1,
            initial_residual: 1.0,
            final_residual: 0.4,
            converged: true,
            time_ms: 0.0,
        });
        assert!((stats.mean_residual() - 0.3).abs() < 1e-5);
    }

    #[test]
    fn max_residual_tracked() {
        let mut stats = SolverStatistics::new();
        stats.record(SolveStats {
            iterations: 1,
            initial_residual: 1.0,
            final_residual: 0.1,
            converged: true,
            time_ms: 0.0,
        });
        stats.record(SolveStats {
            iterations: 1,
            initial_residual: 1.0,
            final_residual: 0.5,
            converged: false,
            time_ms: 0.0,
        });
        assert!((stats.max_residual - 0.5).abs() < 1e-5);
    }
}
