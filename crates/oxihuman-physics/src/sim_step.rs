#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Simulation step accumulator for fixed-timestep loops.

/// Configuration for simulation stepping.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StepConfig {
    pub fixed_dt: f32,
    pub substeps: u32,
}

/// Simulation stepper with accumulator.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SimStep {
    pub config: StepConfig,
    accumulator: f32,
    step_count: u64,
}

/// Create a default `StepConfig` (60 Hz, 4 substeps).
#[allow(dead_code)]
pub fn default_step_config() -> StepConfig {
    StepConfig { fixed_dt: 1.0 / 60.0, substeps: 4 }
}

/// Create a new `SimStep`.
#[allow(dead_code)]
pub fn new_sim_step(config: StepConfig) -> SimStep {
    SimStep { config, accumulator: 0.0, step_count: 0 }
}

/// Perform fixed steps for the given elapsed time. Returns number of steps taken.
#[allow(dead_code)]
pub fn step_fixed(sim: &mut SimStep, elapsed: f32) -> u32 {
    sim.accumulator += elapsed;
    let mut steps = 0u32;
    while sim.accumulator >= sim.config.fixed_dt {
        sim.accumulator -= sim.config.fixed_dt;
        sim.step_count += 1;
        steps += 1;
    }
    steps
}

/// Perform `n` substeps and return the substep count.
#[allow(dead_code)]
pub fn step_substeps(sim: &SimStep) -> u32 {
    sim.config.substeps
}

/// Return the fixed timestep `dt`.
#[allow(dead_code)]
pub fn step_dt(sim: &SimStep) -> f32 {
    sim.config.fixed_dt
}

/// Return the per-substep timestep (`dt / substeps`).
#[allow(dead_code)]
pub fn step_substep_dt(sim: &SimStep) -> f32 {
    sim.config.fixed_dt / sim.config.substeps as f32
}

/// Return the total number of fixed steps taken.
#[allow(dead_code)]
pub fn step_count(sim: &SimStep) -> u64 {
    sim.step_count
}

/// Add elapsed time to the accumulator without stepping.
#[allow(dead_code)]
pub fn accumulate_time(sim: &mut SimStep, dt: f32) {
    sim.accumulator += dt;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sim_step() {
        let s = new_sim_step(default_step_config());
        assert_eq!(step_count(&s), 0);
    }

    #[test]
    fn test_step_fixed_one_step() {
        let mut s = new_sim_step(StepConfig { fixed_dt: 0.016, substeps: 1 });
        let steps = step_fixed(&mut s, 0.016);
        assert_eq!(steps, 1);
        assert_eq!(step_count(&s), 1);
    }

    #[test]
    fn test_step_fixed_multiple() {
        let mut s = new_sim_step(StepConfig { fixed_dt: 0.01, substeps: 1 });
        let steps = step_fixed(&mut s, 0.035);
        assert_eq!(steps, 3);
    }

    #[test]
    fn test_step_fixed_no_step() {
        let mut s = new_sim_step(StepConfig { fixed_dt: 0.016, substeps: 1 });
        let steps = step_fixed(&mut s, 0.005);
        assert_eq!(steps, 0);
    }

    #[test]
    fn test_step_dt() {
        let s = new_sim_step(StepConfig { fixed_dt: 0.02, substeps: 2 });
        assert!((step_dt(&s) - 0.02).abs() < 1e-6);
    }

    #[test]
    fn test_step_substep_dt() {
        let s = new_sim_step(StepConfig { fixed_dt: 0.04, substeps: 4 });
        assert!((step_substep_dt(&s) - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_step_substeps() {
        let s = new_sim_step(StepConfig { fixed_dt: 0.016, substeps: 8 });
        assert_eq!(step_substeps(&s), 8);
    }

    #[test]
    fn test_accumulate_time() {
        let mut s = new_sim_step(StepConfig { fixed_dt: 0.016, substeps: 1 });
        accumulate_time(&mut s, 0.008);
        accumulate_time(&mut s, 0.008);
        let steps = step_fixed(&mut s, 0.0);
        assert_eq!(steps, 1);
    }
}
