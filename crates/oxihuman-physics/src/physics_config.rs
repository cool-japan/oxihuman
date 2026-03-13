#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// Configuration for physics simulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhysicsConfig {
    gravity: [f32; 3],
    time_step_val: f32,
    substeps: u32,
}

#[allow(dead_code)]
pub fn new_physics_config(gravity: [f32; 3], time_step_val: f32, substeps: u32) -> PhysicsConfig {
    PhysicsConfig {
        gravity,
        time_step_val,
        substeps,
    }
}

#[allow(dead_code)]
pub fn default_physics_config() -> PhysicsConfig {
    PhysicsConfig {
        gravity: [0.0, -9.81, 0.0],
        time_step_val: 1.0 / 60.0,
        substeps: 4,
    }
}

#[allow(dead_code)]
pub fn set_gravity_config(config: &mut PhysicsConfig, gravity: [f32; 3]) {
    config.gravity = gravity;
}

#[allow(dead_code)]
pub fn gravity_config(config: &PhysicsConfig) -> [f32; 3] {
    config.gravity
}

#[allow(dead_code)]
pub fn set_time_step(config: &mut PhysicsConfig, dt: f32) {
    config.time_step_val = dt;
}

#[allow(dead_code)]
pub fn time_step(config: &PhysicsConfig) -> f32 {
    config.time_step_val
}

#[allow(dead_code)]
pub fn config_to_json(config: &PhysicsConfig) -> String {
    format!(
        "{{\"gravity\":[{:.6},{:.6},{:.6}],\"time_step\":{:.6},\"substeps\":{}}}",
        config.gravity[0], config.gravity[1], config.gravity[2],
        config.time_step_val, config.substeps
    )
}

#[allow(dead_code)]
pub fn config_substeps(config: &PhysicsConfig) -> u32 {
    config.substeps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_physics_config() {
        let c = new_physics_config([0.0, -10.0, 0.0], 0.01, 8);
        assert_eq!(config_substeps(&c), 8);
    }

    #[test]
    fn test_default_physics_config() {
        let c = default_physics_config();
        let g = gravity_config(&c);
        assert!((g[1] - (-9.81)).abs() < 1e-3);
    }

    #[test]
    fn test_set_gravity_config() {
        let mut c = default_physics_config();
        set_gravity_config(&mut c, [0.0, -20.0, 0.0]);
        assert!((gravity_config(&c)[1] - (-20.0)).abs() < 1e-6);
    }

    #[test]
    fn test_gravity_config() {
        let c = default_physics_config();
        let g = gravity_config(&c);
        assert!(g[0].abs() < 1e-6);
    }

    #[test]
    fn test_set_time_step() {
        let mut c = default_physics_config();
        set_time_step(&mut c, 0.001);
        assert!((time_step(&c) - 0.001).abs() < 1e-6);
    }

    #[test]
    fn test_time_step() {
        let c = default_physics_config();
        assert!(time_step(&c) > 0.0);
    }

    #[test]
    fn test_config_to_json() {
        let c = default_physics_config();
        let json = config_to_json(&c);
        assert!(json.contains("\"gravity\""));
        assert!(json.contains("\"substeps\""));
    }

    #[test]
    fn test_config_substeps() {
        let c = default_physics_config();
        assert_eq!(config_substeps(&c), 4);
    }

    #[test]
    fn test_custom_config() {
        let c = new_physics_config([0.0, 0.0, 0.0], 0.02, 2);
        assert!((time_step(&c) - 0.02).abs() < 1e-6);
    }

    #[test]
    fn test_gravity_zero() {
        let c = new_physics_config([0.0, 0.0, 0.0], 0.01, 1);
        let g = gravity_config(&c);
        assert!(g[0].abs() < 1e-6);
        assert!(g[1].abs() < 1e-6);
        assert!(g[2].abs() < 1e-6);
    }
}
