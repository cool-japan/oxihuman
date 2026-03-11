// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Adaptive time stepping with stability heuristics.

/// Configuration for the adaptive time stepper.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TimeStepConfig {
    pub dt_min: f32,
    pub dt_max: f32,
    pub dt_initial: f32,
    pub safety_factor: f32,
    pub max_substeps: usize,
    pub target_error: f32,
}

impl Default for TimeStepConfig {
    fn default() -> Self {
        Self {
            dt_min: 1e-5,
            dt_max: 1.0 / 30.0,
            dt_initial: 1.0 / 60.0,
            safety_factor: 0.9,
            max_substeps: 16,
            target_error: 1e-4,
        }
    }
}

/// Adaptive time stepper state.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TimeStepper {
    pub config: TimeStepConfig,
    pub current_dt: f32,
    pub total_time: f32,
    pub step_count: usize,
    pub rejected_count: usize,
}

impl TimeStepper {
    #[allow(dead_code)]
    pub fn new(config: TimeStepConfig) -> Self {
        let dt = config.dt_initial;
        Self {
            config,
            current_dt: dt,
            total_time: 0.0,
            step_count: 0,
            rejected_count: 0,
        }
    }

    /// Compute the next dt based on error estimate (PID-like step size control).
    /// `error` is the estimated local error, `order` is the method order.
    #[allow(dead_code)]
    pub fn adapt(&mut self, error: f32, order: u32) -> f32 {
        if error < 1e-15 {
            self.current_dt = (self.current_dt * 2.0).min(self.config.dt_max);
            return self.current_dt;
        }
        let tol = self.config.target_error;
        let factor = self.config.safety_factor * (tol / error).powf(1.0 / (order + 1) as f32);
        let new_dt = self.current_dt * factor.clamp(0.2, 5.0);
        self.current_dt = new_dt.clamp(self.config.dt_min, self.config.dt_max);
        self.current_dt
    }

    /// Advance time by `dt`, recording success.
    #[allow(dead_code)]
    pub fn advance(&mut self, dt: f32) {
        self.total_time += dt;
        self.step_count += 1;
    }

    /// Record a rejected step.
    #[allow(dead_code)]
    pub fn reject(&mut self) {
        self.rejected_count += 1;
        self.current_dt = (self.current_dt * 0.5).max(self.config.dt_min);
    }

    /// CFL-based dt limit for a wave speed and cell size.
    #[allow(dead_code)]
    pub fn cfl_limit(&self, wave_speed: f32, cell_size: f32, cfl_num: f32) -> f32 {
        if wave_speed < 1e-10 {
            return self.config.dt_max;
        }
        (cfl_num * cell_size / wave_speed).clamp(self.config.dt_min, self.config.dt_max)
    }

    /// Returns the number of substeps needed to integrate `frame_time`.
    #[allow(dead_code)]
    pub fn substep_count(&self, frame_time: f32) -> usize {
        let n = (frame_time / self.current_dt).ceil() as usize;
        n.clamp(1, self.config.max_substeps)
    }

    /// Substep dt to exactly cover `frame_time` in `n` steps.
    #[allow(dead_code)]
    pub fn substep_dt(&self, frame_time: f32) -> f32 {
        let n = self.substep_count(frame_time);
        frame_time / n as f32
    }

    /// Reset total time.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.total_time = 0.0;
        self.step_count = 0;
        self.rejected_count = 0;
        self.current_dt = self.config.dt_initial;
    }

    /// Success rate (accepted / total).
    #[allow(dead_code)]
    pub fn success_rate(&self) -> f32 {
        let total = self.step_count + self.rejected_count;
        if total == 0 {
            1.0
        } else {
            self.step_count as f32 / total as f32
        }
    }
}

/// Euler integration stability limit (for spring-mass system).
/// dt < 2 / omega where omega = sqrt(k/m).
#[allow(dead_code)]
pub fn euler_stability_limit(stiffness: f32, mass: f32) -> f32 {
    if stiffness < 1e-10 || mass < 1e-10 {
        return f32::INFINITY;
    }
    let omega = (stiffness / mass).sqrt();
    2.0 / omega
}

/// Verlet stability limit (same as Euler for second-order).
#[allow(dead_code)]
pub fn verlet_stability_limit(stiffness: f32, mass: f32) -> f32 {
    euler_stability_limit(stiffness, mass)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_stepper() -> TimeStepper {
        TimeStepper::new(TimeStepConfig::default())
    }

    #[test]
    fn initial_dt_matches_config() {
        let s = default_stepper();
        assert!((s.current_dt - TimeStepConfig::default().dt_initial).abs() < 1e-8);
    }

    #[test]
    fn advance_increments_time() {
        let mut s = default_stepper();
        s.advance(0.01);
        assert!((s.total_time - 0.01).abs() < 1e-8);
    }

    #[test]
    fn reject_reduces_dt() {
        let mut s = default_stepper();
        let before = s.current_dt;
        s.reject();
        assert!(s.current_dt <= before);
    }

    #[test]
    fn adapt_small_error_increases_dt() {
        let mut s = default_stepper();
        let before = s.current_dt;
        s.adapt(1e-8, 2);
        assert!(s.current_dt >= before);
    }

    #[test]
    fn adapt_large_error_decreases_dt() {
        let mut s = default_stepper();
        s.adapt(1.0, 2);
        assert!(s.current_dt < s.config.dt_initial * 1.1);
    }

    #[test]
    fn cfl_limit_bounded() {
        let s = default_stepper();
        let dt = s.cfl_limit(100.0, 0.01, 0.5);
        assert!(dt >= s.config.dt_min && dt <= s.config.dt_max);
    }

    #[test]
    fn substep_count_at_least_one() {
        let s = default_stepper();
        assert!(s.substep_count(0.016) >= 1);
    }

    #[test]
    fn euler_stability_limit_positive() {
        let dt = euler_stability_limit(100.0, 1.0);
        assert!(dt > 0.0);
    }

    #[test]
    fn success_rate_initial_is_one() {
        let s = default_stepper();
        assert!((s.success_rate() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn reset_clears_time() {
        let mut s = default_stepper();
        s.advance(1.0);
        s.reset();
        assert!(s.total_time < 1e-8);
    }
}
