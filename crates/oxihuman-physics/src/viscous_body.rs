// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A rigid body moving through a viscous medium (Stokes drag).

/// A body subject to viscous drag in 3-D.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ViscousBody {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub mass: f32,
    /// Viscous drag coefficient (b): F_drag = -b * v.
    pub drag_coeff: f32,
    pub time: f32,
    pub steps: u64,
}

#[allow(dead_code)]
impl ViscousBody {
    pub fn new(mass: f32, drag_coeff: f32) -> Self {
        Self {
            pos: [0.0; 3],
            vel: [0.0; 3],
            mass: mass.max(1e-6),
            drag_coeff: drag_coeff.max(0.0),
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

    /// Terminal velocity magnitude for a constant force F.
    pub fn terminal_velocity(&self, force_mag: f32) -> f32 {
        if self.drag_coeff > 0.0 {
            force_mag / self.drag_coeff
        } else {
            f32::INFINITY
        }
    }

    /// Time constant τ = m / b.
    pub fn time_constant(&self) -> f32 {
        self.mass / self.drag_coeff.max(1e-12)
    }

    pub fn speed(&self) -> f32 {
        (self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1] + self.vel[2] * self.vel[2]).sqrt()
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.speed() * self.speed()
    }

    /// Step using Euler integration; `force` is an external force (e.g. gravity).
    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self, dt: f32, force: [f32; 3]) {
        let inv_m = 1.0 / self.mass;
        for i in 0..3 {
            let drag = -self.drag_coeff * self.vel[i];
            let acc = (force[i] + drag) * inv_m;
            self.vel[i] += acc * dt;
            self.pos[i] += self.vel[i] * dt;
        }
        self.time += dt;
        self.steps += 1;
    }

    #[allow(clippy::needless_range_loop)]
    pub fn apply_impulse(&mut self, impulse: [f32; 3]) {
        let inv_m = 1.0 / self.mass;
        for i in 0..3 {
            self.vel[i] += impulse[i] * inv_m;
        }
    }

    pub fn reset(&mut self) {
        self.pos = [0.0; 3];
        self.vel = [0.0; 3];
        self.time = 0.0;
        self.steps = 0;
    }
}

impl Default for ViscousBody {
    fn default() -> Self {
        Self::new(1.0, 0.5)
    }
}

pub fn new_viscous_body(mass: f32, drag_coeff: f32) -> ViscousBody {
    ViscousBody::new(mass, drag_coeff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_slows_body() {
        let mut b = new_viscous_body(1.0, 2.0).with_vel([10.0, 0.0, 0.0]);
        b.step(0.1, [0.0; 3]);
        assert!(b.vel[0] < 10.0);
    }

    #[test]
    fn terminal_velocity_computed() {
        let b = new_viscous_body(1.0, 2.0);
        assert!((b.terminal_velocity(10.0) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn time_constant() {
        let b = new_viscous_body(2.0, 0.5);
        assert!((b.time_constant() - 4.0).abs() < 1e-4);
    }

    #[test]
    fn gravity_accelerates_at_rest() {
        let mut b = new_viscous_body(1.0, 0.0);
        b.step(1.0, [0.0, -9.81, 0.0]);
        assert!(b.vel[1] < 0.0);
    }

    #[test]
    fn step_count() {
        let mut b = new_viscous_body(1.0, 0.1);
        b.step(0.01, [0.0; 3]);
        b.step(0.01, [0.0; 3]);
        assert_eq!(b.steps, 2);
    }

    #[test]
    fn time_advances() {
        let mut b = new_viscous_body(1.0, 0.1);
        b.step(0.5, [0.0; 3]);
        assert!((b.time - 0.5).abs() < 1e-6);
    }

    #[test]
    fn impulse_changes_vel() {
        let mut b = new_viscous_body(1.0, 0.0);
        b.apply_impulse([3.0, 0.0, 0.0]);
        assert!((b.vel[0] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn kinetic_energy_non_negative() {
        let mut b = new_viscous_body(1.0, 0.5).with_vel([2.0, 0.0, 0.0]);
        b.step(0.01, [0.0; 3]);
        assert!(b.kinetic_energy() >= 0.0);
    }

    #[test]
    fn reset_zeroes_state() {
        let mut b = new_viscous_body(1.0, 1.0).with_vel([5.0, 0.0, 0.0]);
        b.step(1.0, [0.0; 3]);
        b.reset();
        assert!(b.speed() < 1e-6);
    }

    #[test]
    fn zero_drag_constant_velocity() {
        let mut b = new_viscous_body(1.0, 0.0).with_vel([3.0, 0.0, 0.0]);
        b.step(1.0, [0.0; 3]);
        assert!((b.vel[0] - 3.0).abs() < 1e-5);
    }
}
