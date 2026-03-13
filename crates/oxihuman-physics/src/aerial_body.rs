// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Aerial rigid body: a rigid body subject to aerodynamic lift and drag forces.

use std::f32::consts::PI;

/// Configuration for an aerial body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AerialBodyConfig {
    pub mass: f32,
    pub drag_coeff: f32,
    pub lift_coeff: f32,
    pub wing_area: f32,
    pub air_density: f32,
}

impl Default for AerialBodyConfig {
    fn default() -> Self {
        Self {
            mass: 1.0,
            drag_coeff: 0.47,
            lift_coeff: 1.2,
            wing_area: 0.5,
            air_density: 1.225,
        }
    }
}

/// State of an aerial body in 2-D vertical plane (x horizontal, y vertical).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AerialBody {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub angle: f32,
    pub cfg: AerialBodyConfig,
}

#[allow(dead_code)]
impl AerialBody {
    pub fn new(cfg: AerialBodyConfig) -> Self {
        Self {
            pos: [0.0; 2],
            vel: [0.0; 2],
            angle: 0.0,
            cfg,
        }
    }

    /// Dynamic pressure q = ½ ρ v²
    pub fn dynamic_pressure(&self) -> f32 {
        let speed_sq = self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1];
        0.5 * self.cfg.air_density * speed_sq
    }

    /// Drag force magnitude.
    pub fn drag_magnitude(&self) -> f32 {
        self.dynamic_pressure() * self.cfg.drag_coeff * self.cfg.wing_area
    }

    /// Lift force magnitude (perpendicular to velocity).
    pub fn lift_magnitude(&self) -> f32 {
        self.dynamic_pressure() * self.cfg.lift_coeff * self.cfg.wing_area
    }

    /// Integrate by one time step `dt` under gravity and aerodynamics.
    pub fn step(&mut self, dt: f32, gravity: f32) {
        let speed = (self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1]).sqrt();
        let (drag_x, drag_y, lift_x, lift_y) = if speed > 1e-8 {
            let vx_n = self.vel[0] / speed;
            let vy_n = self.vel[1] / speed;
            let drag = self.drag_magnitude();
            let lift = self.lift_magnitude();
            // Drag opposes velocity; lift is perpendicular (rotate 90°)
            (-drag * vx_n, -drag * vy_n, -lift * vy_n, lift * vx_n)
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };
        let ax = (drag_x + lift_x) / self.cfg.mass;
        let ay = (drag_y + lift_y) / self.cfg.mass - gravity;
        self.vel[0] += ax * dt;
        self.vel[1] += ay * dt;
        self.pos[0] += self.vel[0] * dt;
        self.pos[1] += self.vel[1] * dt;
        self.angle = self.vel[1].atan2(self.vel[0]);
    }

    pub fn speed(&self) -> f32 {
        (self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1]).sqrt()
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.cfg.mass * self.vel[0] * self.vel[0]
            + 0.5 * self.cfg.mass * self.vel[1] * self.vel[1]
    }
}

pub fn new_aerial_body(cfg: AerialBodyConfig) -> AerialBody {
    AerialBody::new(cfg)
}

pub fn ab_step(body: &mut AerialBody, dt: f32, gravity: f32) {
    body.step(dt, gravity);
}

pub fn ab_dynamic_pressure(body: &AerialBody) -> f32 {
    body.dynamic_pressure()
}

pub fn ab_drag(body: &AerialBody) -> f32 {
    body.drag_magnitude()
}

pub fn ab_lift(body: &AerialBody) -> f32 {
    body.lift_magnitude()
}

/// Angle of attack for a given velocity vector and reference direction.
#[allow(dead_code)]
pub fn angle_of_attack(vel: [f32; 2], ref_dir: [f32; 2]) -> f32 {
    let dot = vel[0] * ref_dir[0] + vel[1] * ref_dir[1];
    let mag_v = (vel[0] * vel[0] + vel[1] * vel[1]).sqrt();
    let mag_r = (ref_dir[0] * ref_dir[0] + ref_dir[1] * ref_dir[1]).sqrt();
    if mag_v < 1e-8 || mag_r < 1e-8 {
        return 0.0;
    }
    (dot / (mag_v * mag_r)).clamp(-1.0, 1.0).acos()
}

/// Stall angle in radians (approx 15°).
pub const STALL_ANGLE: f32 = PI / 12.0;

#[cfg(test)]
mod tests {
    use super::*;

    fn default_body() -> AerialBody {
        let cfg = AerialBodyConfig {
            mass: 1.0,
            drag_coeff: 0.1,
            lift_coeff: 0.5,
            wing_area: 1.0,
            air_density: 1.0,
        };
        let mut b = new_aerial_body(cfg);
        b.vel = [10.0, 0.0];
        b
    }

    #[test]
    fn dynamic_pressure_at_rest_is_zero() {
        let b = new_aerial_body(AerialBodyConfig::default());
        assert_eq!(ab_dynamic_pressure(&b), 0.0);
    }

    #[test]
    fn dynamic_pressure_with_speed() {
        let b = default_body();
        let q = ab_dynamic_pressure(&b);
        assert!((q - 50.0).abs() < 1e-4); // 0.5 * 1.0 * 100.0
    }

    #[test]
    fn drag_positive_at_speed() {
        let b = default_body();
        assert!(ab_drag(&b) > 0.0);
    }

    #[test]
    fn lift_positive_at_speed() {
        let b = default_body();
        assert!(ab_lift(&b) > 0.0);
    }

    #[test]
    fn step_changes_position() {
        let mut b = default_body();
        let old_pos = b.pos;
        ab_step(&mut b, 0.01, 9.81);
        assert!((b.pos[0] - old_pos[0]).abs() > 1e-6);
    }

    #[test]
    fn gravity_pulls_down() {
        let cfg = AerialBodyConfig {
            mass: 1.0,
            drag_coeff: 0.0,
            lift_coeff: 0.0,
            wing_area: 0.0,
            air_density: 0.0,
        };
        let mut b = new_aerial_body(cfg);
        ab_step(&mut b, 1.0, 9.81);
        assert!(b.vel[1] < 0.0);
    }

    #[test]
    fn speed_computation() {
        let mut b = new_aerial_body(AerialBodyConfig::default());
        b.vel = [3.0, 4.0];
        assert!((b.speed() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn kinetic_energy_positive() {
        let b = default_body();
        assert!(b.kinetic_energy() > 0.0);
    }

    #[test]
    fn angle_of_attack_zero_parallel() {
        let aoa = angle_of_attack([1.0, 0.0], [1.0, 0.0]);
        assert!(aoa.abs() < 1e-5);
    }

    #[test]
    fn stall_angle_is_15_degrees() {
        let expected = std::f32::consts::PI / 12.0;
        assert!((STALL_ANGLE - expected).abs() < 1e-6);
    }
}
