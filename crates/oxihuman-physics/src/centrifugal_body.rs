// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A body subject to centrifugal force in a rotating reference frame.

use std::f32::consts::PI;

/// A body in a rotating frame experiencing centrifugal and Coriolis forces.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CentrifugalBody {
    /// Position in the rotating frame.
    pub pos: [f32; 3],
    /// Velocity in the rotating frame.
    pub vel: [f32; 3],
    pub mass: f32,
    /// Angular velocity of the frame (rad/s), about z-axis.
    pub omega: f32,
    pub time: f32,
    pub steps: u64,
}

#[allow(dead_code)]
impl CentrifugalBody {
    pub fn new(mass: f32, omega_rpm: f32) -> Self {
        Self {
            pos: [0.0; 3],
            vel: [0.0; 3],
            mass: mass.max(1e-6),
            omega: omega_rpm * 2.0 * PI / 60.0,
            time: 0.0,
            steps: 0,
        }
    }

    pub fn with_pos(mut self, pos: [f32; 3]) -> Self {
        self.pos = pos;
        self
    }

    pub fn with_vel(mut self, vel: [f32; 3]) -> Self {
        self.vel = vel;
        self
    }

    /// Centrifugal force = m * omega^2 * r_perp (outward, in rotating frame).
    pub fn centrifugal_force(&self) -> [f32; 3] {
        // Rotation about z-axis: centrifugal is outward in xy-plane.
        let omega2 = self.omega * self.omega;
        [
            self.mass * omega2 * self.pos[0],
            self.mass * omega2 * self.pos[1],
            0.0,
        ]
    }

    /// Coriolis force = -2m * omega × v (for z-axis rotation).
    pub fn coriolis_force(&self) -> [f32; 3] {
        [
            -2.0 * self.mass * self.omega * self.vel[1],
            2.0 * self.mass * self.omega * self.vel[0],
            0.0,
        ]
    }

    /// Radial distance from rotation axis.
    pub fn radius(&self) -> f32 {
        (self.pos[0] * self.pos[0] + self.pos[1] * self.pos[1]).sqrt()
    }

    /// Centrifugal potential energy (per unit mass: -0.5 * omega^2 * r^2).
    pub fn centrifugal_potential(&self) -> f32 {
        -0.5 * self.mass * self.omega * self.omega * self.radius() * self.radius()
    }

    pub fn step(&mut self, dt: f32, external: [f32; 3]) {
        let cf = self.centrifugal_force();
        let cor = self.coriolis_force();
        let inv_m = 1.0 / self.mass;
        for i in 0..3 {
            let acc = (cf[i] + cor[i] + external[i]) * inv_m;
            self.vel[i] += acc * dt;
            self.pos[i] += self.vel[i] * dt;
        }
        self.time += dt;
        self.steps += 1;
    }

    pub fn speed(&self) -> f32 {
        (self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1] + self.vel[2] * self.vel[2]).sqrt()
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.speed() * self.speed()
    }

    pub fn reset(&mut self) {
        self.pos = [0.0; 3];
        self.vel = [0.0; 3];
        self.time = 0.0;
        self.steps = 0;
    }
}

impl Default for CentrifugalBody {
    fn default() -> Self {
        Self::new(1.0, 60.0) // 60 RPM
    }
}

pub fn new_centrifugal_body(mass: f32, omega_rpm: f32) -> CentrifugalBody {
    CentrifugalBody::new(mass, omega_rpm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centrifugal_outward() {
        let b = CentrifugalBody::new(1.0, 60.0).with_pos([1.0, 0.0, 0.0]);
        let f = b.centrifugal_force();
        assert!(f[0] > 0.0);
    }

    #[test]
    fn centrifugal_zero_at_origin() {
        let b = CentrifugalBody::new(1.0, 60.0);
        let f = b.centrifugal_force();
        assert!(f.iter().all(|&x| x.abs() < 1e-5));
    }

    #[test]
    fn coriolis_perpendicular() {
        let b = CentrifugalBody::new(1.0, 60.0).with_vel([1.0, 0.0, 0.0]);
        let f = b.coriolis_force();
        // For vel=[1,0,0] and omega about z: coriolis_y = 2*m*omega*vx
        assert!(f[1].abs() > 0.0);
    }

    #[test]
    fn radius_computed() {
        let b = CentrifugalBody::new(1.0, 60.0).with_pos([3.0, 4.0, 0.0]);
        assert!((b.radius() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn centrifugal_force_moves_body_outward() {
        let mut b = CentrifugalBody::new(1.0, 600.0).with_pos([1.0, 0.0, 0.0]);
        b.step(0.1, [0.0; 3]);
        assert!(b.pos[0] > 1.0);
    }

    #[test]
    fn step_count() {
        let mut b = new_centrifugal_body(1.0, 60.0);
        b.step(0.01, [0.0; 3]);
        b.step(0.01, [0.0; 3]);
        assert_eq!(b.steps, 2);
    }

    #[test]
    fn time_advances() {
        let mut b = new_centrifugal_body(1.0, 60.0);
        b.step(0.1, [0.0; 3]);
        assert!((b.time - 0.1).abs() < 1e-5);
    }

    #[test]
    fn kinetic_energy_non_negative() {
        let mut b = new_centrifugal_body(1.0, 60.0).with_pos([1.0, 0.0, 0.0]);
        b.step(0.1, [0.0; 3]);
        assert!(b.kinetic_energy() >= 0.0);
    }

    #[test]
    fn reset_zeroes() {
        let mut b = new_centrifugal_body(1.0, 60.0).with_pos([1.0, 0.0, 0.0]);
        b.step(1.0, [0.0; 3]);
        b.reset();
        assert!(b.speed() < 1e-5);
    }

    #[test]
    fn omega_converts_from_rpm() {
        let b = CentrifugalBody::new(1.0, 60.0);
        assert!((b.omega - 2.0 * PI).abs() < 1e-4);
    }
}
