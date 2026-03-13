// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ballistic projectile simulation with linear aerodynamic drag.

/// Configuration for the projectile.
#[derive(Debug, Clone)]
pub struct ProjectileConfig {
    pub mass: f64,
    pub drag_coefficient: f64,
    pub cross_section: f64,
    pub air_density: f64,
    pub gravity: f64,
}

impl Default for ProjectileConfig {
    fn default() -> Self {
        ProjectileConfig {
            mass: 0.1,
            drag_coefficient: 0.47,
            cross_section: 0.01,
            air_density: 1.225,
            gravity: 9.81,
        }
    }
}

/// State of a projectile.
#[derive(Debug, Clone)]
pub struct ProjectileState {
    pub pos: [f64; 3],
    pub vel: [f64; 3],
    pub time: f64,
}

impl ProjectileState {
    pub fn new(pos: [f64; 3], vel: [f64; 3]) -> Self {
        ProjectileState { pos, vel, time: 0.0 }
    }
}

/// Step the projectile state forward by `dt`.
#[allow(clippy::needless_range_loop)]
pub fn projectile_step(state: &mut ProjectileState, cfg: &ProjectileConfig, dt: f64) {
    let speed = mag3(state.vel);
    let drag_force = 0.5 * cfg.air_density * cfg.drag_coefficient * cfg.cross_section * speed * speed;
    let mut acc = [0.0f64; 3];
    if speed > 1e-12 {
        for k in 0..3 {
            acc[k] -= drag_force / cfg.mass * (state.vel[k] / speed);
        }
    }
    acc[1] -= cfg.gravity;
    for k in 0..3 {
        state.vel[k] += acc[k] * dt;
        state.pos[k] += state.vel[k] * dt;
    }
    state.time += dt;
}

/// Simulate until `state.pos[1] <= 0` or `max_steps` reached.
pub fn projectile_simulate(
    state: &mut ProjectileState,
    cfg: &ProjectileConfig,
    dt: f64,
    max_steps: usize,
) -> usize {
    let mut steps = 0;
    while state.pos[1] > 0.0 && steps < max_steps {
        projectile_step(state, cfg, dt);
        steps += 1;
    }
    steps
}

/// Compute the range (horizontal distance) for a given angle and speed (no drag).
pub fn range_no_drag(speed: f64, angle_rad: f64, gravity: f64) -> f64 {
    speed * speed * (2.0 * angle_rad).sin() / gravity
}

/// Compute time of flight (no drag).
pub fn time_of_flight_no_drag(speed: f64, angle_rad: f64, gravity: f64) -> f64 {
    2.0 * speed * angle_rad.sin() / gravity
}

/// Maximum height reached (no drag).
pub fn max_height_no_drag(speed: f64, angle_rad: f64, gravity: f64) -> f64 {
    let vy = speed * angle_rad.sin();
    vy * vy / (2.0 * gravity)
}

fn mag3(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Create a default projectile config.
pub fn new_projectile_config() -> ProjectileConfig {
    ProjectileConfig::default()
}

/// Create a new projectile state.
pub fn new_projectile_state(pos: [f64; 3], vel: [f64; 3]) -> ProjectileState {
    ProjectileState::new(pos, vel)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_projectile_falls_under_gravity() {
        let cfg = new_projectile_config();
        let mut s = new_projectile_state([0.0, 10.0, 0.0], [0.0; 3]);
        projectile_step(&mut s, &cfg, 0.1);
        assert!(s.vel[1] < 0.0 /* gravity pulls down */);
    }

    #[test]
    fn test_range_no_drag_45_degrees() {
        let r = range_no_drag(10.0, PI / 4.0, 9.81);
        assert!((r - 10.18).abs() < 0.1 /* 45° maximizes range */);
    }

    #[test]
    fn test_time_of_flight() {
        let t = time_of_flight_no_drag(10.0, PI / 2.0, 9.81);
        assert!((t - 2.0 * 10.0 / 9.81).abs() < 1e-6 /* vertical launch */);
    }

    #[test]
    fn test_max_height() {
        let h = max_height_no_drag(10.0, PI / 2.0, 9.81);
        assert!((h - 100.0 / (2.0 * 9.81)).abs() < 1e-6 /* v²/2g */);
    }

    #[test]
    fn test_time_advances() {
        let cfg = new_projectile_config();
        let mut s = new_projectile_state([0.0, 1.0, 0.0], [5.0, 5.0, 0.0]);
        projectile_step(&mut s, &cfg, 0.01);
        assert!((s.time - 0.01).abs() < 1e-12 /* time advanced */);
    }

    #[test]
    fn test_simulate_hits_ground() {
        let cfg = new_projectile_config();
        let mut s = new_projectile_state([0.0, 10.0, 0.0], [5.0, 0.0, 0.0]);
        let steps = projectile_simulate(&mut s, &cfg, 0.01, 10000);
        assert!(steps < 10000 /* hit ground before max steps */);
        assert!(s.pos[1] <= 0.0 /* below ground */);
    }

    #[test]
    fn test_horizontal_component_positive() {
        let cfg = new_projectile_config();
        let mut s = new_projectile_state([0.0, 10.0, 0.0], [10.0, 0.0, 0.0]);
        projectile_step(&mut s, &cfg, 0.1);
        assert!(s.pos[0] > 0.0 /* moving forward */);
    }

    #[test]
    fn test_drag_reduces_speed() {
        let cfg = new_projectile_config();
        let mut s = new_projectile_state([0.0, 100.0, 0.0], [50.0, 0.0, 0.0]);
        let v0 = mag3(s.vel);
        projectile_step(&mut s, &cfg, 0.1);
        /* horizontal speed should decrease due to drag */
        assert!(s.vel[0] < 50.0 /* drag acts */);
        let _ = v0;
    }

    #[test]
    fn test_zero_gravity_no_fall() {
        let cfg = ProjectileConfig { gravity: 0.0, drag_coefficient: 0.0, ..Default::default() };
        let mut s = new_projectile_state([0.0, 5.0, 0.0], [0.0; 3]);
        projectile_step(&mut s, &cfg, 1.0);
        assert!((s.pos[1] - 5.0).abs() < 1e-9 /* no gravity, stays put */);
    }

    #[test]
    fn test_config_default_values() {
        let cfg = new_projectile_config();
        assert!(cfg.gravity > 0.0 /* gravity positive */);
        assert!(cfg.mass > 0.0);
    }
}
