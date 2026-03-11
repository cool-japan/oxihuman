// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A body on a rotating planet subject to the Coriolis effect.

use std::f32::consts::PI;

/// A body moving on a rotating sphere (e.g. Earth) experiencing Coriolis deflection.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CoriolisBody {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub mass: f32,
    /// Earth's rotation rate (rad/s). Default: 7.27e-5.
    pub omega: f32,
    /// Latitude in radians.
    pub latitude: f32,
    pub time: f32,
    pub steps: u64,
}

#[allow(dead_code)]
impl CoriolisBody {
    pub fn new(mass: f32, latitude_deg: f32) -> Self {
        Self {
            pos: [0.0; 3],
            vel: [0.0; 3],
            mass: mass.max(1e-6),
            omega: 7.2921e-5, // Earth
            latitude: latitude_deg * PI / 180.0,
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

    /// Coriolis parameter f = 2 * omega * sin(lat).
    pub fn coriolis_param(&self) -> f32 {
        2.0 * self.omega * self.latitude.sin()
    }

    /// Coriolis acceleration for horizontal motion (x=East, y=North, z=Up).
    /// a_coriolis = -f * k × v (for horizontal motion).
    pub fn coriolis_acceleration(&self) -> [f32; 3] {
        let f = self.coriolis_param();
        // For 2-D horizontal plane: a_x = f * v_y, a_y = -f * v_x.
        [f * self.vel[1], -f * self.vel[0], 0.0]
    }

    pub fn step(&mut self, dt: f32, external: [f32; 3]) {
        let cor = self.coriolis_acceleration();
        let inv_m = 1.0 / self.mass;
        for i in 0..3 {
            let acc = (cor[i] + external[i]) * inv_m;
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

    pub fn deflection_angle_rad(&self) -> f32 {
        // Approximate: angle of velocity from initial direction.
        self.vel[0].atan2(self.vel[1])
    }

    pub fn reset(&mut self) {
        self.pos = [0.0; 3];
        self.vel = [0.0; 3];
        self.time = 0.0;
        self.steps = 0;
    }
}

impl Default for CoriolisBody {
    fn default() -> Self {
        Self::new(1.0, 45.0) // 45° latitude
    }
}

pub fn new_coriolis_body(mass: f32, latitude_deg: f32) -> CoriolisBody {
    CoriolisBody::new(mass, latitude_deg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coriolis_param_nonzero_at_pole() {
        let b = new_coriolis_body(1.0, 90.0);
        let f = b.coriolis_param();
        assert!((f - 2.0 * 7.2921e-5).abs() < 1e-8);
    }

    #[test]
    fn coriolis_param_zero_at_equator() {
        let b = new_coriolis_body(1.0, 0.0);
        assert!(b.coriolis_param().abs() < 1e-10);
    }

    #[test]
    fn northward_motion_deflects_right_nh() {
        // NH: f > 0. Moving north (vy>0) → cor_x = f*vy > 0 (deflects right/east).
        let b = new_coriolis_body(1.0, 45.0).with_vel([0.0, 100.0, 0.0]);
        let cor = b.coriolis_acceleration();
        assert!(cor[0] > 0.0); // eastward deflection
    }

    #[test]
    fn southward_motion_deflects_left_nh() {
        let b = new_coriolis_body(1.0, 45.0).with_vel([0.0, -100.0, 0.0]);
        let cor = b.coriolis_acceleration();
        assert!(cor[0] < 0.0);
    }

    #[test]
    fn step_count() {
        let mut b = new_coriolis_body(1.0, 45.0);
        b.step(0.01, [0.0; 3]);
        b.step(0.01, [0.0; 3]);
        assert_eq!(b.steps, 2);
    }

    #[test]
    fn time_advances() {
        let mut b = new_coriolis_body(1.0, 45.0);
        b.step(0.1, [0.0; 3]);
        assert!((b.time - 0.1).abs() < 1e-5);
    }

    #[test]
    fn external_force_moves_body() {
        let mut b = new_coriolis_body(1.0, 45.0);
        b.step(1.0, [100.0, 0.0, 0.0]);
        assert!(b.vel[0] > 0.0);
    }

    #[test]
    fn kinetic_energy_non_negative() {
        let mut b = new_coriolis_body(1.0, 45.0).with_vel([10.0, 0.0, 0.0]);
        b.step(0.01, [0.0; 3]);
        assert!(b.kinetic_energy() >= 0.0);
    }

    #[test]
    fn reset_zeroes() {
        let mut b = new_coriolis_body(1.0, 45.0).with_vel([5.0, 0.0, 0.0]);
        b.step(1.0, [0.0; 3]);
        b.reset();
        assert!(b.speed() < 1e-5);
    }

    #[test]
    fn latitude_converts_from_deg() {
        let b = new_coriolis_body(1.0, 45.0);
        assert!((b.latitude - PI / 4.0).abs() < 1e-5);
    }
}
