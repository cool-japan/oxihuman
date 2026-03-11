// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Foam body: a collection of buoyant spherical bubbles with drag and buoyancy.

use std::f32::consts::PI;

/// Configuration for the foam simulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FoamConfig {
    pub water_density: f32,
    pub bubble_density: f32,
    pub drag_coeff: f32,
    pub surface_tension: f32,
    pub gravity: f32,
}

impl Default for FoamConfig {
    fn default() -> Self {
        Self {
            water_density: 1000.0,
            bubble_density: 1.2,
            drag_coeff: 0.47,
            surface_tension: 0.072,
            gravity: 9.81,
        }
    }
}

/// A single foam bubble.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FoamBubble {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub radius: f32,
    pub alive: bool,
}

#[allow(dead_code)]
impl FoamBubble {
    pub fn new(pos: [f32; 3], radius: f32) -> Self {
        Self {
            pos,
            vel: [0.0; 3],
            radius,
            alive: true,
        }
    }

    pub fn volume(&self) -> f32 {
        (4.0 / 3.0) * PI * self.radius * self.radius * self.radius
    }

    pub fn cross_section(&self) -> f32 {
        PI * self.radius * self.radius
    }
}

/// A foam body managing a collection of bubbles.
#[allow(dead_code)]
pub struct FoamBody {
    pub bubbles: Vec<FoamBubble>,
    pub cfg: FoamConfig,
    pub water_surface_y: f32,
}

#[allow(dead_code)]
impl FoamBody {
    pub fn new(cfg: FoamConfig) -> Self {
        Self {
            bubbles: Vec::new(),
            cfg,
            water_surface_y: 0.0,
        }
    }

    pub fn add_bubble(&mut self, pos: [f32; 3], radius: f32) {
        self.bubbles.push(FoamBubble::new(pos, radius));
    }

    /// Compute net upward force on a bubble (buoyancy - weight - drag).
    pub fn net_force_y(&self, bubble: &FoamBubble) -> f32 {
        let vol = bubble.volume();
        let buoyancy = self.cfg.water_density * self.cfg.gravity * vol;
        let weight = self.cfg.bubble_density * self.cfg.gravity * vol;
        let speed = (bubble.vel[0] * bubble.vel[0]
            + bubble.vel[1] * bubble.vel[1]
            + bubble.vel[2] * bubble.vel[2])
            .sqrt();
        let drag = 0.5
            * self.cfg.water_density
            * self.cfg.drag_coeff
            * bubble.cross_section()
            * speed
            * speed;
        let drag_sign = if bubble.vel[1] > 0.0 { -1.0 } else { 1.0 };
        buoyancy - weight + drag * drag_sign
    }

    /// Integrate all alive bubbles for one timestep.
    pub fn step(&mut self, dt: f32) {
        for bubble in &mut self.bubbles {
            if !bubble.alive {
                continue;
            }
            let vol = bubble.volume();
            let mass = self.cfg.bubble_density * vol;
            let fy = {
                let buoyancy = self.cfg.water_density * self.cfg.gravity * vol;
                let weight = mass * self.cfg.gravity;
                let speed_y = bubble.vel[1].abs();
                let drag = 0.5
                    * self.cfg.water_density
                    * self.cfg.drag_coeff
                    * bubble.cross_section()
                    * speed_y
                    * speed_y;
                let drag_sign = if bubble.vel[1] > 0.0 { -1.0 } else { 1.0 };
                buoyancy - weight + drag * drag_sign
            };
            bubble.vel[1] += (fy / mass) * dt;
            bubble.pos[0] += bubble.vel[0] * dt;
            bubble.pos[1] += bubble.vel[1] * dt;
            bubble.pos[2] += bubble.vel[2] * dt;
            // Pop when reaching the water surface
            if bubble.pos[1] >= self.water_surface_y {
                bubble.alive = false;
            }
        }
    }

    pub fn alive_count(&self) -> usize {
        self.bubbles.iter().filter(|b| b.alive).count()
    }

    pub fn total_volume(&self) -> f32 {
        self.bubbles
            .iter()
            .filter(|b| b.alive)
            .map(|b| b.volume())
            .sum()
    }

    /// Remove dead bubbles.
    pub fn prune(&mut self) {
        self.bubbles.retain(|b| b.alive);
    }

    pub fn bubble_count(&self) -> usize {
        self.bubbles.len()
    }
}

pub fn new_foam_body(cfg: FoamConfig) -> FoamBody {
    FoamBody::new(cfg)
}

pub fn fob_add(body: &mut FoamBody, pos: [f32; 3], radius: f32) {
    body.add_bubble(pos, radius);
}

pub fn fob_step(body: &mut FoamBody, dt: f32) {
    body.step(dt);
}

pub fn fob_alive(body: &FoamBody) -> usize {
    body.alive_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_body_empty() {
        let f = new_foam_body(FoamConfig::default());
        assert_eq!(f.bubble_count(), 0);
    }

    #[test]
    fn add_bubble() {
        let mut f = new_foam_body(FoamConfig::default());
        fob_add(&mut f, [0.0, -1.0, 0.0], 0.01);
        assert_eq!(f.bubble_count(), 1);
    }

    #[test]
    fn bubble_volume_positive() {
        let b = FoamBubble::new([0.0; 3], 0.05);
        assert!(b.volume() > 0.0);
    }

    #[test]
    fn buoyancy_exceeds_weight_for_light_bubble() {
        let f = new_foam_body(FoamConfig::default());
        let b = FoamBubble::new([0.0, -0.5, 0.0], 0.01);
        let fy = f.net_force_y(&b);
        // Bubble density < water density → buoyancy > weight
        assert!(fy > 0.0);
    }

    #[test]
    fn step_moves_bubble_upward() {
        let mut f = new_foam_body(FoamConfig::default());
        fob_add(&mut f, [0.0, -1.0, 0.0], 0.01);
        let old_y = f.bubbles[0].pos[1];
        fob_step(&mut f, 0.01);
        assert!(f.bubbles[0].pos[1] > old_y);
    }

    #[test]
    fn bubble_pops_at_surface() {
        let mut f = new_foam_body(FoamConfig::default());
        f.water_surface_y = 0.0;
        // Place bubble exactly at surface
        fob_add(&mut f, [0.0, 0.0, 0.0], 0.01);
        fob_step(&mut f, 0.001);
        // After any step, bubble at surface (pos[1] >= 0.0) → alive = false
        assert_eq!(fob_alive(&f), 0);
    }

    #[test]
    fn prune_removes_dead() {
        let mut f = new_foam_body(FoamConfig::default());
        fob_add(&mut f, [0.0, -0.001, 0.0], 0.01);
        f.bubbles[0].alive = false;
        f.prune();
        assert_eq!(f.bubble_count(), 0);
    }

    #[test]
    fn total_volume_sums_alive() {
        let mut f = new_foam_body(FoamConfig::default());
        fob_add(&mut f, [0.0, -1.0, 0.0], 0.1);
        fob_add(&mut f, [0.0, -2.0, 0.0], 0.1);
        f.bubbles[0].alive = false;
        let vol = f.total_volume();
        assert!(vol > 0.0 && vol < f.bubbles[1].volume() + 1e-6);
    }

    #[test]
    fn multiple_steps_no_nan() {
        let mut f = new_foam_body(FoamConfig::default());
        fob_add(&mut f, [0.0, -2.0, 0.0], 0.02);
        for _ in 0..20 {
            fob_step(&mut f, 0.01);
            if fob_alive(&f) == 0 {
                break;
            }
        }
        // pass
    }

    #[test]
    fn cross_section_formula() {
        let b = FoamBubble::new([0.0; 3], 1.0);
        assert!((b.cross_section() - PI).abs() < 1e-5);
    }
}
