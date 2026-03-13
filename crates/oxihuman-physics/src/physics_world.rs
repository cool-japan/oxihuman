#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Physics world: top-level simulation container.

/// Configuration for the physics world.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WorldConfig {
    pub gravity: [f32; 3],
    pub fixed_dt: f32,
    pub max_substeps: u32,
}

/// The physics world holding bodies and configuration.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PhysicsWorld {
    pub config: WorldConfig,
    pub body_count: usize,
    pub time_elapsed: f32,
    pub step_count: u64,
}

/// Create a default `WorldConfig` (earth gravity, 60 Hz).
#[allow(dead_code)]
pub fn default_world_config() -> WorldConfig {
    WorldConfig { gravity: [0.0, -9.81, 0.0], fixed_dt: 1.0 / 60.0, max_substeps: 8 }
}

/// Create a new `PhysicsWorld`.
#[allow(dead_code)]
pub fn new_physics_world(config: WorldConfig) -> PhysicsWorld {
    PhysicsWorld { config, body_count: 0, time_elapsed: 0.0, step_count: 0 }
}

/// Advance the world by one fixed timestep.
#[allow(dead_code)]
pub fn world_step(world: &mut PhysicsWorld) {
    world.time_elapsed += world.config.fixed_dt;
    world.step_count += 1;
}

/// Return the number of bodies in the world.
#[allow(dead_code)]
pub fn world_body_count(world: &PhysicsWorld) -> usize {
    world.body_count
}

/// Return the gravity vector.
#[allow(dead_code)]
pub fn world_gravity(world: &PhysicsWorld) -> [f32; 3] {
    world.config.gravity
}

/// Set the gravity vector.
#[allow(dead_code)]
pub fn set_world_gravity(world: &mut PhysicsWorld, gravity: [f32; 3]) {
    world.config.gravity = gravity;
}

/// Return the total elapsed simulation time.
#[allow(dead_code)]
pub fn world_time_elapsed(world: &PhysicsWorld) -> f32 {
    world.time_elapsed
}

/// Reset the world to initial state.
#[allow(dead_code)]
pub fn reset_physics_world(world: &mut PhysicsWorld) {
    world.time_elapsed = 0.0;
    world.step_count = 0;
    world.body_count = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_physics_world() {
        let w = new_physics_world(default_world_config());
        assert_eq!(world_body_count(&w), 0);
        assert!((world_time_elapsed(&w)).abs() < 1e-6);
    }

    #[test]
    fn test_world_step() {
        let mut w = new_physics_world(default_world_config());
        world_step(&mut w);
        assert!(world_time_elapsed(&w) > 0.0);
        assert_eq!(w.step_count, 1);
    }

    #[test]
    fn test_world_gravity() {
        let w = new_physics_world(default_world_config());
        let g = world_gravity(&w);
        assert!((g[1] + 9.81).abs() < 1e-4);
    }

    #[test]
    fn test_set_world_gravity() {
        let mut w = new_physics_world(default_world_config());
        set_world_gravity(&mut w, [0.0, -1.62, 0.0]); // moon gravity
        assert!((world_gravity(&w)[1] + 1.62).abs() < 1e-4);
    }

    #[test]
    fn test_reset_physics_world() {
        let mut w = new_physics_world(default_world_config());
        world_step(&mut w);
        world_step(&mut w);
        reset_physics_world(&mut w);
        assert!((world_time_elapsed(&w)).abs() < 1e-6);
        assert_eq!(w.step_count, 0);
    }

    #[test]
    fn test_world_body_count() {
        let mut w = new_physics_world(default_world_config());
        w.body_count = 5;
        assert_eq!(world_body_count(&w), 5);
    }

    #[test]
    fn test_default_world_config() {
        let c = default_world_config();
        assert_eq!(c.max_substeps, 8);
    }

    #[test]
    fn test_multiple_steps() {
        let mut w = new_physics_world(default_world_config());
        let dt = w.config.fixed_dt;
        world_step(&mut w);
        world_step(&mut w);
        world_step(&mut w);
        assert!((world_time_elapsed(&w) - dt * 3.0).abs() < 1e-5);
    }
}
