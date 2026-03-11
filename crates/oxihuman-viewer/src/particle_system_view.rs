#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Particle system UI state.

/// UI state for a single particle system.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParticleSystemView {
    pub name: String,
    pub count: u32,
    pub seed: u32,
    pub frame_start: f32,
    pub frame_end: f32,
    pub visible: bool,
    pub expanded: bool,
}

/// Create a new `ParticleSystemView` with sensible defaults.
#[allow(dead_code)]
pub fn new_particle_system_view(name: &str) -> ParticleSystemView {
    ParticleSystemView {
        name: name.to_string(),
        count: 1000,
        seed: 0,
        frame_start: 1.0,
        frame_end: 250.0,
        visible: true,
        expanded: false,
    }
}

/// Set the frame range; `start` is clamped to be ≤ `end_`.
#[allow(dead_code)]
pub fn set_frame_range(view: &mut ParticleSystemView, start: f32, end_: f32) {
    view.frame_start = start;
    view.frame_end = end_.max(start);
}

/// Toggle the visibility of the particle system.
#[allow(dead_code)]
pub fn toggle_visibility(view: &mut ParticleSystemView) {
    view.visible = !view.visible;
}

/// Return the particle count.
#[allow(dead_code)]
pub fn particle_count(view: &ParticleSystemView) -> u32 {
    view.count
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_view_has_name() {
        let v = new_particle_system_view("Sparks");
        assert_eq!(v.name, "Sparks");
    }

    #[test]
    fn new_view_defaults_visible() {
        let v = new_particle_system_view("P");
        assert!(v.visible);
    }

    #[test]
    fn particle_count_default() {
        let v = new_particle_system_view("P");
        assert_eq!(particle_count(&v), 1000);
    }

    #[test]
    fn toggle_visibility_hides() {
        let mut v = new_particle_system_view("P");
        toggle_visibility(&mut v);
        assert!(!v.visible);
    }

    #[test]
    fn toggle_visibility_shows() {
        let mut v = new_particle_system_view("P");
        toggle_visibility(&mut v);
        toggle_visibility(&mut v);
        assert!(v.visible);
    }

    #[test]
    fn set_frame_range_valid() {
        let mut v = new_particle_system_view("P");
        set_frame_range(&mut v, 10.0, 200.0);
        assert!((v.frame_start - 10.0).abs() < 1e-6);
        assert!((v.frame_end - 200.0).abs() < 1e-6);
    }

    #[test]
    fn set_frame_range_clamps_end_below_start() {
        let mut v = new_particle_system_view("P");
        set_frame_range(&mut v, 50.0, 20.0);
        assert!(v.frame_end >= v.frame_start);
    }

    #[test]
    fn frame_end_at_least_start() {
        let mut v = new_particle_system_view("P");
        set_frame_range(&mut v, 100.0, 5.0);
        assert!((v.frame_end - 100.0).abs() < 1e-6);
    }

    #[test]
    fn new_view_defaults_not_expanded() {
        let v = new_particle_system_view("P");
        assert!(!v.expanded);
    }

    #[test]
    fn seed_defaults_zero() {
        let v = new_particle_system_view("P");
        assert_eq!(v.seed, 0);
    }
}
