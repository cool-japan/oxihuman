#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Solver iteration configuration.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SolverIterationConfig {
    velocity_iters: usize,
    position_iters: usize,
}

#[allow(dead_code)]
pub fn new_solver_iteration_config() -> SolverIterationConfig {
    SolverIterationConfig {
        velocity_iters: 8,
        position_iters: 3,
    }
}

#[allow(dead_code)]
pub fn set_velocity_iters(cfg: &mut SolverIterationConfig, iters: usize) {
    cfg.velocity_iters = iters;
}

#[allow(dead_code)]
pub fn set_position_iters(cfg: &mut SolverIterationConfig, iters: usize) {
    cfg.position_iters = iters;
}

#[allow(dead_code)]
pub fn velocity_iters(cfg: &SolverIterationConfig) -> usize {
    cfg.velocity_iters
}

#[allow(dead_code)]
pub fn position_iters(cfg: &SolverIterationConfig) -> usize {
    cfg.position_iters
}

#[allow(dead_code)]
pub fn total_iters(cfg: &SolverIterationConfig) -> usize {
    cfg.velocity_iters + cfg.position_iters
}

#[allow(dead_code)]
pub fn iteration_to_json(cfg: &SolverIterationConfig) -> String {
    format!(
        r#"{{"velocity_iters":{},"position_iters":{},"total":{}}}"#,
        cfg.velocity_iters,
        cfg.position_iters,
        total_iters(cfg)
    )
}

#[allow(dead_code)]
pub fn default_iteration_config() -> SolverIterationConfig {
    new_solver_iteration_config()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cfg = new_solver_iteration_config();
        assert_eq!(velocity_iters(&cfg), 8);
        assert_eq!(position_iters(&cfg), 3);
    }

    #[test]
    fn test_set_velocity_iters() {
        let mut cfg = new_solver_iteration_config();
        set_velocity_iters(&mut cfg, 16);
        assert_eq!(velocity_iters(&cfg), 16);
    }

    #[test]
    fn test_set_position_iters() {
        let mut cfg = new_solver_iteration_config();
        set_position_iters(&mut cfg, 6);
        assert_eq!(position_iters(&cfg), 6);
    }

    #[test]
    fn test_total() {
        let cfg = new_solver_iteration_config();
        assert_eq!(total_iters(&cfg), 11);
    }

    #[test]
    fn test_to_json() {
        let cfg = new_solver_iteration_config();
        let json = iteration_to_json(&cfg);
        assert!(json.contains("\"velocity_iters\":8"));
    }

    #[test]
    fn test_default_config_fn() {
        let cfg = default_iteration_config();
        assert_eq!(velocity_iters(&cfg), 8);
    }

    #[test]
    fn test_custom_total() {
        let mut cfg = new_solver_iteration_config();
        set_velocity_iters(&mut cfg, 4);
        set_position_iters(&mut cfg, 2);
        assert_eq!(total_iters(&cfg), 6);
    }

    #[test]
    fn test_zero_iters() {
        let mut cfg = new_solver_iteration_config();
        set_velocity_iters(&mut cfg, 0);
        assert_eq!(velocity_iters(&cfg), 0);
    }

    #[test]
    fn test_high_iters() {
        let mut cfg = new_solver_iteration_config();
        set_velocity_iters(&mut cfg, 100);
        assert_eq!(velocity_iters(&cfg), 100);
    }

    #[test]
    fn test_json_total() {
        let mut cfg = new_solver_iteration_config();
        set_velocity_iters(&mut cfg, 10);
        set_position_iters(&mut cfg, 5);
        let json = iteration_to_json(&cfg);
        assert!(json.contains("\"total\":15"));
    }
}
