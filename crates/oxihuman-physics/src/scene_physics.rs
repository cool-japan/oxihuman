// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Physics scene manager: bodies + constraints registry.

#![allow(dead_code)]

/// A physics scene containing body and constraint id registries.
#[allow(dead_code)]
pub struct PhysicsScene {
    pub body_ids: Vec<u32>,
    pub constraint_ids: Vec<u32>,
    pub gravity: [f32; 3],
    pub time: f32,
    pub paused: bool,
}

/// Create a new physics scene with the given gravity vector.
#[allow(dead_code)]
pub fn new_physics_scene(gravity: [f32; 3]) -> PhysicsScene {
    PhysicsScene {
        body_ids: Vec::new(),
        constraint_ids: Vec::new(),
        gravity,
        time: 0.0,
        paused: false,
    }
}

/// Register a body id with the scene.
#[allow(dead_code)]
pub fn add_body_id(scene: &mut PhysicsScene, id: u32) {
    if !scene.body_ids.contains(&id) {
        scene.body_ids.push(id);
    }
}

/// Remove a body id from the scene. Returns true if it was present.
#[allow(dead_code)]
pub fn remove_body_id(scene: &mut PhysicsScene, id: u32) -> bool {
    if let Some(pos) = scene.body_ids.iter().position(|&b| b == id) {
        scene.body_ids.remove(pos);
        true
    } else {
        false
    }
}

/// Advance the scene simulation by `dt` seconds (if not paused).
#[allow(dead_code)]
pub fn step_scene(scene: &mut PhysicsScene, dt: f32) {
    if !scene.paused {
        scene.time += dt;
    }
}

/// Return the number of registered bodies.
#[allow(dead_code)]
pub fn body_count(scene: &PhysicsScene) -> usize {
    scene.body_ids.len()
}

/// Pause the scene simulation.
#[allow(dead_code)]
pub fn pause_scene(scene: &mut PhysicsScene) {
    scene.paused = true;
}

/// Resume the scene simulation.
#[allow(dead_code)]
pub fn resume_scene(scene: &mut PhysicsScene) {
    scene.paused = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_scene_empty() {
        let s = new_physics_scene([0.0, -9.8, 0.0]);
        assert_eq!(body_count(&s), 0);
        assert!((s.time).abs() < 1e-6);
    }

    #[test]
    fn gravity_stored() {
        let s = new_physics_scene([0.0, -9.8, 0.0]);
        assert!((s.gravity[1] - (-9.8)).abs() < 1e-5);
    }

    #[test]
    fn add_body_increments_count() {
        let mut s = new_physics_scene([0.0, -9.8, 0.0]);
        add_body_id(&mut s, 1);
        add_body_id(&mut s, 2);
        assert_eq!(body_count(&s), 2);
    }

    #[test]
    fn add_body_no_duplicates() {
        let mut s = new_physics_scene([0.0, -9.8, 0.0]);
        add_body_id(&mut s, 1);
        add_body_id(&mut s, 1);
        assert_eq!(body_count(&s), 1);
    }

    #[test]
    fn remove_body_decrements_count() {
        let mut s = new_physics_scene([0.0, -9.8, 0.0]);
        add_body_id(&mut s, 5);
        assert!(remove_body_id(&mut s, 5));
        assert_eq!(body_count(&s), 0);
    }

    #[test]
    fn remove_missing_body_returns_false() {
        let mut s = new_physics_scene([0.0, -9.8, 0.0]);
        assert!(!remove_body_id(&mut s, 99));
    }

    #[test]
    fn step_advances_time() {
        let mut s = new_physics_scene([0.0, -9.8, 0.0]);
        step_scene(&mut s, 0.016);
        assert!((s.time - 0.016).abs() < 1e-6);
    }

    #[test]
    fn paused_scene_does_not_advance_time() {
        let mut s = new_physics_scene([0.0, -9.8, 0.0]);
        pause_scene(&mut s);
        step_scene(&mut s, 0.1);
        assert!(s.time.abs() < 1e-6);
    }

    #[test]
    fn resume_allows_step() {
        let mut s = new_physics_scene([0.0, -9.8, 0.0]);
        pause_scene(&mut s);
        resume_scene(&mut s);
        step_scene(&mut s, 0.1);
        assert!((s.time - 0.1).abs() < 1e-6);
    }
}
