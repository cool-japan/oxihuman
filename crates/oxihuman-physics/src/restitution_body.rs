// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Coefficient-of-restitution body for bounce simulation.

/// A body with restitution properties.
#[derive(Debug, Clone)]
pub struct RestitutionBody {
    pub mass: f32,
    pub velocity: [f32; 3],
    pub restitution: f32,
    pub bounce_count: u32,
}

#[allow(dead_code)]
impl RestitutionBody {
    pub fn new(mass: f32, restitution: f32) -> Self {
        RestitutionBody {
            mass: mass.max(1e-6),
            velocity: [0.0; 3],
            restitution: restitution.clamp(0.0, 1.0),
            bounce_count: 0,
        }
    }

    /// Apply a surface normal bounce (1-D along normal axis).
    pub fn bounce_normal(&mut self, normal: [f32; 3]) {
        let dot = self.velocity[0] * normal[0]
            + self.velocity[1] * normal[1]
            + self.velocity[2] * normal[2];
        if dot >= 0.0 {
            return;
        }
        let factor = (1.0 + self.restitution) * dot;
        self.velocity[0] -= factor * normal[0];
        self.velocity[1] -= factor * normal[1];
        self.velocity[2] -= factor * normal[2];
        self.bounce_count += 1;
    }

    pub fn kinetic_energy(&self) -> f32 {
        let v2 = self.velocity[0].powi(2) + self.velocity[1].powi(2) + self.velocity[2].powi(2);
        0.5 * self.mass * v2
    }

    pub fn speed(&self) -> f32 {
        (self.velocity[0].powi(2) + self.velocity[1].powi(2) + self.velocity[2].powi(2)).sqrt()
    }

    pub fn set_velocity(&mut self, v: [f32; 3]) {
        self.velocity = v;
    }

    pub fn apply_impulse(&mut self, j: [f32; 3]) {
        let inv_m = 1.0 / self.mass;
        self.velocity[0] += j[0] * inv_m;
        self.velocity[1] += j[1] * inv_m;
        self.velocity[2] += j[2] * inv_m;
    }

    pub fn energy_after_n_bounces(&self, n: u32) -> f32 {
        let r2 = self.restitution.powi(2);
        self.kinetic_energy() * r2.powi(n as i32)
    }

    pub fn stop(&mut self) {
        self.velocity = [0.0; 3];
    }

    pub fn is_at_rest(&self) -> bool {
        self.speed() < 1e-6
    }
}

pub fn new_restitution_body(mass: f32, restitution: f32) -> RestitutionBody {
    RestitutionBody::new(mass, restitution)
}

pub fn collision_impulse(mass_a: f32, vel_a: f32, mass_b: f32, vel_b: f32, e: f32) -> (f32, f32) {
    let j = (1.0 + e) * (vel_b - vel_a) / (1.0 / mass_a + 1.0 / mass_b);
    (j / mass_a, -j / mass_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounce_reverses_component() {
        let mut b = new_restitution_body(1.0, 1.0);
        b.set_velocity([0.0, -5.0, 0.0]);
        b.bounce_normal([0.0, 1.0, 0.0]);
        assert!(b.velocity[1] > 0.0);
        assert_eq!(b.bounce_count, 1);
    }

    #[test]
    fn elastic_preserves_speed() {
        let mut b = new_restitution_body(1.0, 1.0);
        b.set_velocity([0.0, -3.0, 0.0]);
        let speed_before = b.speed();
        b.bounce_normal([0.0, 1.0, 0.0]);
        assert!((b.speed() - speed_before).abs() < 1e-5);
    }

    #[test]
    fn inelastic_loses_speed() {
        let mut b = new_restitution_body(1.0, 0.5);
        b.set_velocity([0.0, -4.0, 0.0]);
        let before = b.speed();
        b.bounce_normal([0.0, 1.0, 0.0]);
        assert!(b.speed() < before);
    }

    #[test]
    fn kinetic_energy() {
        let mut b = new_restitution_body(2.0, 1.0);
        b.set_velocity([3.0, 4.0, 0.0]);
        assert!((b.kinetic_energy() - 25.0).abs() < 1e-5);
    }

    #[test]
    fn apply_impulse() {
        let mut b = new_restitution_body(1.0, 1.0);
        b.apply_impulse([0.0, 10.0, 0.0]);
        assert_eq!(b.velocity[1], 10.0);
    }

    #[test]
    fn at_rest_check() {
        let mut b = new_restitution_body(1.0, 1.0);
        assert!(b.is_at_rest());
        b.set_velocity([1.0, 0.0, 0.0]);
        assert!(!b.is_at_rest());
    }

    #[test]
    fn energy_after_bounces() {
        let mut b = new_restitution_body(2.0, 0.5);
        b.set_velocity([1.0, 0.0, 0.0]);
        let e0 = b.kinetic_energy(); // 0.5 * 2 * 1 = 1.0
        let r2 = 0.5f32.powi(2);
        let e2 = e0 * r2.powi(2);
        assert!((b.energy_after_n_bounces(2) - e2).abs() < 1e-6);
    }

    #[test]
    fn collision_impulse_momentum() {
        let (da, db) = collision_impulse(1.0, 5.0, 1.0, -5.0, 1.0);
        assert!((da + db).abs() < 1e-5);
    }

    #[test]
    fn stop_zeroes_velocity() {
        let mut b = new_restitution_body(1.0, 1.0);
        b.set_velocity([3.0, 4.0, 5.0]);
        b.stop();
        assert!(b.is_at_rest());
    }
}
