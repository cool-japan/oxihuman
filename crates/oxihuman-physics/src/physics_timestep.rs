// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Fixed timestep management for physics simulation.

#![allow(dead_code)]

/// Fixed timestep manager with accumulator.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhysicsTimestep {
    pub fixed_dt: f32,
    pub accumulator: f32,
    pub max_steps: u32,
    pub steps_this_frame: u32,
}

/// Creates a new physics timestep manager.
#[allow(dead_code)]
pub fn new_physics_timestep(fixed_dt: f32, max_steps: u32) -> PhysicsTimestep {
    PhysicsTimestep {
        fixed_dt: fixed_dt.max(1e-6),
        accumulator: 0.0,
        max_steps,
        steps_this_frame: 0,
    }
}

/// Advances the accumulator by `dt` and returns how many fixed steps to take.
/// The step count is capped at `max_steps`.
#[allow(dead_code)]
pub fn update_timestep(ts: &mut PhysicsTimestep, dt: f32) -> u32 {
    ts.accumulator += dt.max(0.0);
    let mut steps = 0u32;
    while ts.accumulator >= ts.fixed_dt && steps < ts.max_steps {
        ts.accumulator -= ts.fixed_dt;
        steps += 1;
    }
    ts.steps_this_frame = steps;
    steps
}

/// Returns the interpolation alpha (leftover accumulator / fixed_dt) in [0, 1].
#[allow(dead_code)]
pub fn timestep_alpha(ts: &PhysicsTimestep) -> f32 {
    (ts.accumulator / ts.fixed_dt).clamp(0.0, 1.0)
}

/// Resets the accumulator and step counter.
#[allow(dead_code)]
pub fn reset_timestep(ts: &mut PhysicsTimestep) {
    ts.accumulator = 0.0;
    ts.steps_this_frame = 0;
}

/// Returns the fixed timestep value.
#[allow(dead_code)]
pub fn timestep_fixed_dt(ts: &PhysicsTimestep) -> f32 {
    ts.fixed_dt
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-5;

    #[test]
    fn test_new_defaults() {
        let ts = new_physics_timestep(1.0 / 60.0, 4);
        assert!((ts.fixed_dt - 1.0 / 60.0).abs() < EPS);
        assert_eq!(ts.max_steps, 4);
        assert!((ts.accumulator).abs() < EPS);
    }

    #[test]
    fn test_update_one_step() {
        let mut ts = new_physics_timestep(1.0 / 60.0, 4);
        let steps = update_timestep(&mut ts, 1.0 / 60.0);
        assert_eq!(steps, 1);
    }

    #[test]
    fn test_update_no_step() {
        let mut ts = new_physics_timestep(1.0 / 60.0, 4);
        let steps = update_timestep(&mut ts, 0.001);
        assert_eq!(steps, 0);
    }

    #[test]
    fn test_update_capped() {
        let mut ts = new_physics_timestep(1.0 / 60.0, 3);
        let steps = update_timestep(&mut ts, 1.0); // would be ~60 steps
        assert_eq!(steps, 3);
    }

    #[test]
    fn test_alpha_zero_after_step() {
        let mut ts = new_physics_timestep(1.0 / 60.0, 4);
        update_timestep(&mut ts, 1.0 / 60.0);
        let alpha = timestep_alpha(&ts);
        assert!(alpha < EPS);
    }

    #[test]
    fn test_alpha_partial() {
        let mut ts = new_physics_timestep(1.0, 4);
        update_timestep(&mut ts, 0.5);
        let alpha = timestep_alpha(&ts);
        assert!((alpha - 0.5).abs() < EPS);
    }

    #[test]
    fn test_reset() {
        let mut ts = new_physics_timestep(1.0 / 60.0, 4);
        update_timestep(&mut ts, 0.03);
        reset_timestep(&mut ts);
        assert!((ts.accumulator).abs() < EPS);
        assert_eq!(ts.steps_this_frame, 0);
    }

    #[test]
    fn test_fixed_dt() {
        let ts = new_physics_timestep(0.02, 8);
        assert!((timestep_fixed_dt(&ts) - 0.02).abs() < EPS);
    }

    #[test]
    fn test_negative_dt_ignored() {
        let mut ts = new_physics_timestep(1.0 / 60.0, 4);
        let steps = update_timestep(&mut ts, -1.0);
        assert_eq!(steps, 0);
    }
}
