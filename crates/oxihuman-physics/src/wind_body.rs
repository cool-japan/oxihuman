// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A body subject to aerodynamic wind forces (drag + lift).

/// A body affected by wind forces.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WindBody {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub mass: f32,
    /// Drag coefficient C_d.
    pub drag_coeff: f32,
    /// Cross-sectional area (m²).
    pub area: f32,
    /// Air density (kg/m³).
    pub air_density: f32,
    pub time: f32,
    pub steps: u64,
}

#[allow(dead_code)]
impl WindBody {
    pub fn new(mass: f32, drag_coeff: f32, area: f32) -> Self {
        Self {
            pos: [0.0; 3],
            vel: [0.0; 3],
            mass: mass.max(1e-6),
            drag_coeff: drag_coeff.max(0.0),
            area: area.max(0.0),
            air_density: 1.225,
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

    pub fn set_air_density(&mut self, rho: f32) {
        self.air_density = rho.max(0.0);
    }

    /// Compute aerodynamic drag force given wind velocity (world frame).
    pub fn drag_force(&self, wind_vel: [f32; 3]) -> [f32; 3] {
        // Relative velocity of body w.r.t. wind.
        let rv = [
            self.vel[0] - wind_vel[0],
            self.vel[1] - wind_vel[1],
            self.vel[2] - wind_vel[2],
        ];
        let speed2 = rv[0] * rv[0] + rv[1] * rv[1] + rv[2] * rv[2];
        if speed2 < 1e-10 {
            return [0.0; 3];
        }
        let speed = speed2.sqrt();
        let mag = 0.5 * self.air_density * self.drag_coeff * self.area * speed2;
        // Force opposes relative motion.
        [
            -mag * rv[0] / speed,
            -mag * rv[1] / speed,
            -mag * rv[2] / speed,
        ]
    }

    pub fn step(&mut self, dt: f32, gravity: [f32; 3], wind_vel: [f32; 3]) {
        let drag = self.drag_force(wind_vel);
        let inv_m = 1.0 / self.mass;
        for i in 0..3 {
            let acc = (gravity[i] + drag[i]) * inv_m;
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

impl Default for WindBody {
    fn default() -> Self {
        Self::new(1.0, 0.47, 0.1)
    }
}

pub fn new_wind_body(mass: f32, drag_coeff: f32, area: f32) -> WindBody {
    WindBody::new(mass, drag_coeff, area)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_opposes_motion() {
        let b = new_wind_body(1.0, 1.0, 1.0).with_vel([10.0, 0.0, 0.0]);
        let d = b.drag_force([0.0; 3]);
        assert!(d[0] < 0.0);
    }

    #[test]
    fn wind_pushes_body() {
        let mut b = new_wind_body(0.1, 2.0, 0.5);
        b.step(1.0, [0.0; 3], [10.0, 0.0, 0.0]);
        assert!(b.vel[0] > 0.0);
    }

    #[test]
    fn gravity_works() {
        let mut b = new_wind_body(1.0, 0.0, 0.0);
        b.step(1.0, [0.0, -9.81, 0.0], [0.0; 3]);
        assert!(b.vel[1] < 0.0);
    }

    #[test]
    fn zero_relative_vel_zero_drag() {
        let b = new_wind_body(1.0, 1.0, 1.0).with_vel([5.0, 0.0, 0.0]);
        let d = b.drag_force([5.0, 0.0, 0.0]);
        assert!(d.iter().all(|&x| x.abs() < 1e-5));
    }

    #[test]
    fn step_count() {
        let mut b = new_wind_body(1.0, 0.5, 0.1);
        b.step(0.01, [0.0; 3], [0.0; 3]);
        b.step(0.01, [0.0; 3], [0.0; 3]);
        assert_eq!(b.steps, 2);
    }

    #[test]
    fn time_advances() {
        let mut b = new_wind_body(1.0, 0.5, 0.1);
        b.step(0.5, [0.0; 3], [0.0; 3]);
        assert!((b.time - 0.5).abs() < 1e-6);
    }

    #[test]
    fn kinetic_energy_non_negative() {
        let mut b = new_wind_body(1.0, 0.5, 0.1).with_vel([2.0, 0.0, 0.0]);
        b.step(0.01, [0.0; 3], [0.0; 3]);
        assert!(b.kinetic_energy() >= 0.0);
    }

    #[test]
    fn reset_zeroes() {
        let mut b = new_wind_body(1.0, 0.5, 0.1);
        b.step(1.0, [0.0, -9.81, 0.0], [5.0, 0.0, 0.0]);
        b.reset();
        assert!(b.speed() < 1e-5);
        assert_eq!(b.steps, 0);
    }

    #[test]
    fn air_density_set() {
        let mut b = new_wind_body(1.0, 0.5, 0.1);
        b.set_air_density(0.5);
        assert!((b.air_density - 0.5).abs() < 1e-6);
    }

    #[test]
    fn higher_drag_coeff_greater_force() {
        let b1 = new_wind_body(1.0, 0.5, 1.0).with_vel([10.0, 0.0, 0.0]);
        let b2 = new_wind_body(1.0, 2.0, 1.0).with_vel([10.0, 0.0, 0.0]);
        let d1 = b1.drag_force([0.0; 3]);
        let d2 = b2.drag_force([0.0; 3]);
        assert!(d2[0].abs() > d1[0].abs());
    }
}
