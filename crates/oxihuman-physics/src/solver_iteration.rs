// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! PBD solver iteration manager.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SolverIterConfig {
    pub iterations: usize,
    pub sub_steps: usize,
    pub convergence_threshold: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SolverIterState {
    pub current_iter: usize,
    pub converged: bool,
    pub last_error: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IterResult {
    pub iterations_done: usize,
    pub converged: bool,
    pub final_error: f32,
}

#[allow(dead_code)]
pub fn default_solver_iter_config() -> SolverIterConfig {
    SolverIterConfig {
        iterations: 10,
        sub_steps: 4,
        convergence_threshold: 1e-4,
    }
}

#[allow(dead_code)]
pub fn new_solver_iter_state() -> SolverIterState {
    SolverIterState { current_iter: 0, converged: false, last_error: f32::INFINITY }
}

#[allow(dead_code)]
pub fn si_begin_frame(state: &mut SolverIterState) {
    state.current_iter = 0;
    state.converged = false;
    state.last_error = f32::INFINITY;
}

/// Returns true if iteration should continue.
#[allow(dead_code)]
pub fn si_step(state: &mut SolverIterState, error: f32, config: &SolverIterConfig) -> bool {
    state.last_error = error;
    state.current_iter += 1;
    if error < config.convergence_threshold {
        state.converged = true;
        return false;
    }
    state.current_iter < config.iterations
}

#[allow(dead_code)]
pub fn si_end_frame(state: &SolverIterState) -> IterResult {
    IterResult {
        iterations_done: state.current_iter,
        converged: state.converged,
        final_error: state.last_error,
    }
}

#[allow(dead_code)]
pub fn si_iterations_done(state: &SolverIterState) -> usize {
    state.current_iter
}

#[allow(dead_code)]
pub fn si_is_converged(state: &SolverIterState) -> bool {
    state.converged
}

#[allow(dead_code)]
pub fn si_to_json(state: &SolverIterState) -> String {
    format!(
        "{{\"current_iter\":{},\"converged\":{},\"last_error\":{}}}",
        state.current_iter, state.converged, state.last_error
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_solver_iter_config();
        assert_eq!(cfg.iterations, 10);
        assert_eq!(cfg.sub_steps, 4);
    }

    #[test]
    fn test_new_state() {
        let s = new_solver_iter_state();
        assert_eq!(s.current_iter, 0);
        assert!(!s.converged);
    }

    #[test]
    fn test_begin_frame_resets() {
        let mut s = new_solver_iter_state();
        s.current_iter = 5;
        si_begin_frame(&mut s);
        assert_eq!(s.current_iter, 0);
    }

    #[test]
    fn test_si_step_continues() {
        let cfg = default_solver_iter_config();
        let mut s = new_solver_iter_state();
        si_begin_frame(&mut s);
        let cont = si_step(&mut s, 1.0, &cfg);
        assert!(cont);
    }

    #[test]
    fn test_si_step_converges() {
        let cfg = default_solver_iter_config();
        let mut s = new_solver_iter_state();
        si_begin_frame(&mut s);
        let cont = si_step(&mut s, 1e-6, &cfg);
        assert!(!cont);
        assert!(si_is_converged(&s));
    }

    #[test]
    fn test_si_step_max_iters() {
        let cfg = SolverIterConfig { iterations: 3, sub_steps: 1, convergence_threshold: 1e-10 };
        let mut s = new_solver_iter_state();
        si_begin_frame(&mut s);
        si_step(&mut s, 1.0, &cfg);
        si_step(&mut s, 1.0, &cfg);
        let cont = si_step(&mut s, 1.0, &cfg);
        assert!(!cont);
    }

    #[test]
    fn test_si_end_frame() {
        let cfg = default_solver_iter_config();
        let mut s = new_solver_iter_state();
        si_begin_frame(&mut s);
        si_step(&mut s, 0.5, &cfg);
        let r = si_end_frame(&s);
        assert_eq!(r.iterations_done, 1);
    }

    #[test]
    fn test_si_to_json() {
        let s = new_solver_iter_state();
        let j = si_to_json(&s);
        assert!(j.contains("current_iter"));
    }

    #[test]
    fn test_iterations_done() {
        let cfg = default_solver_iter_config();
        let mut s = new_solver_iter_state();
        si_begin_frame(&mut s);
        si_step(&mut s, 1.0, &cfg);
        si_step(&mut s, 1.0, &cfg);
        assert_eq!(si_iterations_done(&s), 2);
    }
}
