// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// A rigid body with full 3x3 inertia tensor (stored as diagonal for simplicity).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct InertiaBody {
    mass: f32,
    inv_mass: f32,
    inertia: [f32; 3],
    inv_inertia: [f32; 3],
    position: [f32; 3],
    velocity: [f32; 3],
    angular_velocity: [f32; 3],
    force_accum: [f32; 3],
    torque_accum: [f32; 3],
}

#[allow(dead_code)]
impl InertiaBody {
    pub fn new(mass: f32, inertia: [f32; 3]) -> Self {
        let inv_m = if mass > f32::EPSILON { 1.0 / mass } else { 0.0 };
        let inv_i = [
            if inertia[0] > f32::EPSILON {
                1.0 / inertia[0]
            } else {
                0.0
            },
            if inertia[1] > f32::EPSILON {
                1.0 / inertia[1]
            } else {
                0.0
            },
            if inertia[2] > f32::EPSILON {
                1.0 / inertia[2]
            } else {
                0.0
            },
        ];
        Self {
            mass: mass.max(0.0),
            inv_mass: inv_m,
            inertia,
            inv_inertia: inv_i,
            position: [0.0; 3],
            velocity: [0.0; 3],
            angular_velocity: [0.0; 3],
            force_accum: [0.0; 3],
            torque_accum: [0.0; 3],
        }
    }

    pub fn sphere(mass: f32, radius: f32) -> Self {
        let _ = PI;
        let i = 0.4 * mass * radius * radius;
        Self::new(mass, [i, i, i])
    }

    pub fn cuboid(mass: f32, half_extents: [f32; 3]) -> Self {
        let hx = half_extents[0];
        let hy = half_extents[1];
        let hz = half_extents[2];
        let factor = mass / 3.0;
        Self::new(
            mass,
            [
                factor * (hy * hy + hz * hz),
                factor * (hx * hx + hz * hz),
                factor * (hx * hx + hy * hy),
            ],
        )
    }

    #[allow(clippy::needless_range_loop)]
    pub fn apply_force(&mut self, force: [f32; 3]) {
        for i in 0..3 {
            self.force_accum[i] += force[i];
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn apply_torque(&mut self, torque: [f32; 3]) {
        for i in 0..3 {
            self.torque_accum[i] += torque[i];
        }
    }

    pub fn apply_force_at_point(&mut self, force: [f32; 3], point: [f32; 3]) {
        self.apply_force(force);
        // torque = r x F
        let r = [
            point[0] - self.position[0],
            point[1] - self.position[1],
            point[2] - self.position[2],
        ];
        let cross = [
            r[1] * force[2] - r[2] * force[1],
            r[2] * force[0] - r[0] * force[2],
            r[0] * force[1] - r[1] * force[0],
        ];
        self.apply_torque(cross);
    }

    pub fn integrate(&mut self, dt: f32) {
        // linear
        for i in 0..3 {
            self.velocity[i] += self.force_accum[i] * self.inv_mass * dt;
            self.position[i] += self.velocity[i] * dt;
        }
        // angular
        for i in 0..3 {
            self.angular_velocity[i] += self.torque_accum[i] * self.inv_inertia[i] * dt;
        }
        self.force_accum = [0.0; 3];
        self.torque_accum = [0.0; 3];
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    pub fn set_position(&mut self, p: [f32; 3]) {
        self.position = p;
    }

    pub fn velocity(&self) -> [f32; 3] {
        self.velocity
    }

    pub fn set_velocity(&mut self, v: [f32; 3]) {
        self.velocity = v;
    }

    pub fn angular_velocity(&self) -> [f32; 3] {
        self.angular_velocity
    }

    pub fn set_angular_velocity(&mut self, w: [f32; 3]) {
        self.angular_velocity = w;
    }

    pub fn mass(&self) -> f32 {
        self.mass
    }

    pub fn inertia(&self) -> [f32; 3] {
        self.inertia
    }

    pub fn kinetic_energy(&self) -> f32 {
        let v2 = self.velocity[0] * self.velocity[0]
            + self.velocity[1] * self.velocity[1]
            + self.velocity[2] * self.velocity[2];
        let w2 = self.inertia[0] * self.angular_velocity[0] * self.angular_velocity[0]
            + self.inertia[1] * self.angular_velocity[1] * self.angular_velocity[1]
            + self.inertia[2] * self.angular_velocity[2] * self.angular_velocity[2];
        0.5 * self.mass * v2 + 0.5 * w2
    }

    pub fn angular_momentum(&self) -> [f32; 3] {
        [
            self.inertia[0] * self.angular_velocity[0],
            self.inertia[1] * self.angular_velocity[1],
            self.inertia[2] * self.angular_velocity[2],
        ]
    }

    pub fn is_static(&self) -> bool {
        self.inv_mass == 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let b = InertiaBody::new(2.0, [1.0, 1.0, 1.0]);
        assert!((b.mass() - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sphere() {
        let b = InertiaBody::sphere(10.0, 1.0);
        assert!((b.inertia()[0] - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply_force_and_integrate() {
        let mut b = InertiaBody::new(1.0, [1.0; 3]);
        b.apply_force([10.0, 0.0, 0.0]);
        b.integrate(1.0);
        assert!(b.velocity()[0] > 0.0);
    }

    #[test]
    fn test_apply_torque() {
        let mut b = InertiaBody::new(1.0, [1.0; 3]);
        b.apply_torque([0.0, 5.0, 0.0]);
        b.integrate(1.0);
        assert!(b.angular_velocity()[1] > 0.0);
    }

    #[test]
    fn test_force_at_point() {
        let mut b = InertiaBody::new(1.0, [1.0; 3]);
        b.set_position([0.0; 3]);
        b.apply_force_at_point([0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        b.integrate(1.0);
        // should produce angular velocity
        let w = b.angular_velocity();
        let mag = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();
        assert!(mag > 0.0);
    }

    #[test]
    fn test_kinetic_energy_at_rest() {
        let b = InertiaBody::new(1.0, [1.0; 3]);
        assert!((b.kinetic_energy() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_kinetic_energy() {
        let mut b = InertiaBody::new(2.0, [1.0; 3]);
        b.set_velocity([1.0, 0.0, 0.0]);
        assert!((b.kinetic_energy() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_angular_momentum() {
        let mut b = InertiaBody::new(1.0, [2.0, 2.0, 2.0]);
        b.set_angular_velocity([3.0, 0.0, 0.0]);
        let am = b.angular_momentum();
        assert!((am[0] - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_static() {
        let b = InertiaBody::new(0.0, [0.0; 3]);
        assert!(b.is_static());
    }

    #[test]
    fn test_cuboid() {
        let b = InertiaBody::cuboid(12.0, [1.0, 1.0, 1.0]);
        // Ix = m/3 * (hy^2 + hz^2) = 12/3 * 2 = 8
        assert!((b.inertia()[0] - 8.0).abs() < f32::EPSILON);
    }
}
