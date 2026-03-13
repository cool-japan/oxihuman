// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A buoyant body floating in a fluid with Archimedes buoyancy.

/// A buoyant rigid body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BuoyantBody {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub mass: f32,
    /// Volume of the body (m³).
    pub volume: f32,
    /// Fluid density (kg/m³).
    pub fluid_density: f32,
    /// Linear drag coefficient.
    pub drag_coeff: f32,
    /// Water surface Y level.
    pub surface_y: f32,
    pub time: f32,
    pub steps: u64,
}

#[allow(dead_code)]
impl BuoyantBody {
    pub fn new(mass: f32, volume: f32) -> Self {
        Self {
            pos: [0.0; 3],
            vel: [0.0; 3],
            mass: mass.max(1e-6),
            volume: volume.max(0.0),
            fluid_density: 1000.0, // water
            drag_coeff: 1.0,
            surface_y: 0.0,
            time: 0.0,
            steps: 0,
        }
    }

    pub fn with_pos(mut self, pos: [f32; 3]) -> Self {
        self.pos = pos;
        self
    }

    /// Compute submerged fraction [0, 1].
    pub fn submerged_fraction(&self) -> f32 {
        // Simple: entirely above → 0, entirely below → 1.
        if self.pos[1] >= self.surface_y {
            0.0
        } else {
            1.0_f32.min((self.surface_y - self.pos[1]) / 0.5_f32.max(self.volume.cbrt()))
        }
    }

    /// Buoyancy force (upward).
    pub fn buoyancy_force(&self) -> f32 {
        let submerged_vol = self.volume * self.submerged_fraction();
        self.fluid_density * 9.81 * submerged_vol
    }

    /// Weight.
    pub fn weight(&self) -> f32 {
        self.mass * 9.81
    }

    /// Equilibrium depth below surface.
    pub fn equilibrium_depth(&self) -> f32 {
        // At equilibrium: buoyancy = weight → submerged_vol = mass / fluid_density.
        self.mass / self.fluid_density / self.volume.max(1e-9)
    }

    pub fn step(&mut self, dt: f32) {
        let gravity = -9.81;
        let buoy = self.buoyancy_force();
        let drag_y = if self.submerged_fraction() > 0.0 {
            -self.drag_coeff * self.vel[1]
        } else {
            0.0
        };
        let acc_y = (gravity * self.mass + buoy + drag_y) / self.mass;
        self.vel[1] += acc_y * dt;
        self.pos[1] += self.vel[1] * dt;
        self.time += dt;
        self.steps += 1;
    }

    pub fn speed(&self) -> f32 {
        self.vel[1].abs()
    }

    pub fn is_floating(&self) -> bool {
        (0.0..=1.0).contains(&self.submerged_fraction())
    }

    pub fn reset(&mut self) {
        self.pos = [0.0; 3];
        self.vel = [0.0; 3];
        self.time = 0.0;
        self.steps = 0;
    }
}

impl Default for BuoyantBody {
    fn default() -> Self {
        Self::new(1.0, 0.001)
    }
}

pub fn new_buoyant_body(mass: f32, volume: f32) -> BuoyantBody {
    BuoyantBody::new(mass, volume)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn above_surface_not_submerged() {
        let b = new_buoyant_body(1.0, 0.01).with_pos([0.0, 1.0, 0.0]);
        assert!(b.submerged_fraction() < 1e-5);
    }

    #[test]
    fn below_surface_submerged() {
        let b = new_buoyant_body(1.0, 0.01).with_pos([0.0, -1.0, 0.0]);
        assert!(b.submerged_fraction() > 0.0);
    }

    #[test]
    fn buoyancy_force_upward() {
        let b = new_buoyant_body(1.0, 0.01).with_pos([0.0, -1.0, 0.0]);
        assert!(b.buoyancy_force() > 0.0);
    }

    #[test]
    fn step_increments_steps() {
        let mut b = new_buoyant_body(1.0, 0.01);
        b.step(0.01);
        b.step(0.01);
        assert_eq!(b.steps, 2);
    }

    #[test]
    fn time_advances() {
        let mut b = new_buoyant_body(1.0, 0.01);
        b.step(0.1);
        assert!((b.time - 0.1).abs() < 1e-5);
    }

    #[test]
    fn heavy_body_sinks() {
        let mut b = new_buoyant_body(10.0, 0.001).with_pos([0.0, 0.0, 0.0]);
        for _ in 0..100 {
            b.step(0.01);
        }
        assert!(b.pos[1] < 0.0);
    }

    #[test]
    fn light_body_floats() {
        let mut b = new_buoyant_body(0.001, 0.1).with_pos([0.0, -0.05, 0.0]);
        for _ in 0..500 {
            b.step(0.01);
        }
        // Should not sink much below surface.
        assert!(b.pos[1] > -0.5);
    }

    #[test]
    fn reset_zeroes() {
        let mut b = new_buoyant_body(1.0, 0.01);
        b.step(0.5);
        b.reset();
        assert_eq!(b.steps, 0);
    }

    #[test]
    fn weight_matches_gravity() {
        let b = new_buoyant_body(2.0, 0.01);
        assert!((b.weight() - 2.0 * 9.81).abs() < 1e-4);
    }

    #[test]
    fn equilibrium_depth_reasonable() {
        let b = new_buoyant_body(1.0, 0.001);
        let d = b.equilibrium_depth();
        assert!(d >= 0.0);
    }
}
