// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hydrofoil body: a foil generating lift in a fluid (water or dense medium).

use std::f32::consts::PI;

/// Hydrofoil geometry and fluid parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HydrofoilConfig {
    pub chord_length: f32,
    pub span: f32,
    pub mass: f32,
    pub fluid_density: f32,
    pub lift_slope: f32, // dCl/d(alpha) per radian (~2π for thin foil)
    pub drag_base: f32,  // zero-lift drag coefficient
}

impl Default for HydrofoilConfig {
    fn default() -> Self {
        Self {
            chord_length: 0.3,
            span: 1.0,
            mass: 5.0,
            fluid_density: 1025.0, // sea water
            lift_slope: 2.0 * PI,
            drag_base: 0.01,
        }
    }
}

/// State of a hydrofoil body in 2-D.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HydrofoilBody {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub angle_of_attack: f32, // radians
    pub cfg: HydrofoilConfig,
}

#[allow(dead_code)]
impl HydrofoilBody {
    pub fn new(cfg: HydrofoilConfig) -> Self {
        Self {
            pos: [0.0; 2],
            vel: [0.0; 2],
            angle_of_attack: 0.0,
            cfg,
        }
    }

    pub fn wing_area(&self) -> f32 {
        self.cfg.chord_length * self.cfg.span
    }

    pub fn dynamic_pressure(&self) -> f32 {
        let v_sq = self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1];
        0.5 * self.cfg.fluid_density * v_sq
    }

    pub fn lift_coeff(&self) -> f32 {
        self.cfg.lift_slope * self.angle_of_attack
    }

    pub fn drag_coeff(&self) -> f32 {
        let cl = self.lift_coeff();
        self.cfg.drag_base + cl * cl / (PI * self.cfg.span / self.cfg.chord_length)
    }

    pub fn lift_force(&self) -> f32 {
        self.dynamic_pressure() * self.wing_area() * self.lift_coeff()
    }

    pub fn drag_force(&self) -> f32 {
        self.dynamic_pressure() * self.wing_area() * self.drag_coeff()
    }

    pub fn step(&mut self, dt: f32, gravity: f32) {
        let speed = (self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1]).sqrt();
        let lift = self.lift_force();
        let drag = self.drag_force();
        let (ax, ay) = if speed > 1e-8 {
            let vxn = self.vel[0] / speed;
            let vyn = self.vel[1] / speed;
            // Lift is perpendicular to velocity (rotate 90° left)
            let lx = -vyn * lift;
            let ly = vxn * lift;
            // Drag opposes velocity
            let dx = -vxn * drag;
            let dy = -vyn * drag;
            (
                (lx + dx) / self.cfg.mass,
                (ly + dy) / self.cfg.mass - gravity,
            )
        } else {
            (0.0, -gravity)
        };
        self.vel[0] += ax * dt;
        self.vel[1] += ay * dt;
        self.pos[0] += self.vel[0] * dt;
        self.pos[1] += self.vel[1] * dt;
    }

    pub fn speed(&self) -> f32 {
        (self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1]).sqrt()
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.cfg.mass * (self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1])
    }

    /// Lift-to-drag ratio.
    pub fn lift_to_drag_ratio(&self) -> f32 {
        let cd = self.drag_coeff();
        if cd < 1e-9 {
            return 0.0;
        }
        self.lift_coeff() / cd
    }
}

pub fn new_hydrofoil_body(cfg: HydrofoilConfig) -> HydrofoilBody {
    HydrofoilBody::new(cfg)
}

pub fn hf_step(body: &mut HydrofoilBody, dt: f32, gravity: f32) {
    body.step(dt, gravity);
}

pub fn hf_lift(body: &HydrofoilBody) -> f32 {
    body.lift_force()
}

pub fn hf_drag(body: &HydrofoilBody) -> f32 {
    body.drag_force()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fast_foil() -> HydrofoilBody {
        let mut b = new_hydrofoil_body(HydrofoilConfig::default());
        b.vel = [5.0, 0.0];
        b.angle_of_attack = 0.1;
        b
    }

    #[test]
    fn zero_speed_zero_lift() {
        let b = new_hydrofoil_body(HydrofoilConfig::default());
        assert_eq!(hf_lift(&b), 0.0);
    }

    #[test]
    fn positive_aoa_positive_lift() {
        let b = fast_foil();
        assert!(hf_lift(&b) > 0.0);
    }

    #[test]
    fn drag_positive_at_speed() {
        let b = fast_foil();
        assert!(hf_drag(&b) > 0.0);
    }

    #[test]
    fn wing_area_formula() {
        let b = new_hydrofoil_body(HydrofoilConfig::default());
        assert!((b.wing_area() - 0.3).abs() < 1e-5);
    }

    #[test]
    fn step_changes_pos() {
        let mut b = fast_foil();
        let old = b.pos;
        hf_step(&mut b, 0.01, 9.81);
        assert!(b.pos[0] != old[0] || b.pos[1] != old[1]);
    }

    #[test]
    fn gravity_pulls_down_at_zero_speed() {
        let mut b = new_hydrofoil_body(HydrofoilConfig::default());
        hf_step(&mut b, 1.0, 9.81);
        assert!(b.vel[1] < 0.0);
    }

    #[test]
    fn kinetic_energy_positive_with_speed() {
        let b = fast_foil();
        assert!(b.kinetic_energy() > 0.0);
    }

    #[test]
    fn lift_to_drag_positive() {
        let b = fast_foil();
        assert!(b.lift_to_drag_ratio() > 0.0);
    }

    #[test]
    fn negative_aoa_negative_lift() {
        let mut b = fast_foil();
        b.angle_of_attack = -0.1;
        assert!(hf_lift(&b) < 0.0);
    }

    #[test]
    fn speed_formula() {
        let mut b = new_hydrofoil_body(HydrofoilConfig::default());
        b.vel = [3.0, 4.0];
        assert!((b.speed() - 5.0).abs() < 1e-5);
    }
}
