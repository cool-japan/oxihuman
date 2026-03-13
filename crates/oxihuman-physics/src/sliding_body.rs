// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Body constrained to slide along a 1-D axis with friction and limits.

/// A body that slides along a single axis.
#[derive(Debug, Clone)]
pub struct SlidingBody {
    pub mass: f32,
    pub position: f32,
    pub velocity: f32,
    pub min_position: f32,
    pub max_position: f32,
    pub friction_coeff: f32,
    pub normal_force: f32,
    pub accumulated_force: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
impl SlidingBody {
    pub fn new(mass: f32, min_pos: f32, max_pos: f32) -> Self {
        SlidingBody {
            mass: mass.max(1e-9),
            position: 0.0,
            velocity: 0.0,
            min_position: min_pos,
            max_position: max_pos,
            friction_coeff: 0.0,
            normal_force: 0.0,
            accumulated_force: 0.0,
            enabled: true,
        }
    }

    pub fn apply_force(&mut self, f: f32) {
        self.accumulated_force += f;
    }

    pub fn step(&mut self, dt: f32) {
        if !self.enabled {
            self.accumulated_force = 0.0;
            return;
        }
        let friction = if self.velocity.abs() > 1e-9 {
            -self.velocity.signum() * self.friction_coeff * self.normal_force
        } else {
            0.0
        };
        let total = self.accumulated_force + friction;
        self.velocity += total / self.mass * dt;
        self.position += self.velocity * dt;
        self.accumulated_force = 0.0;
        self.clamp();
    }

    fn clamp(&mut self) {
        if self.position < self.min_position {
            self.position = self.min_position;
            if self.velocity < 0.0 {
                self.velocity = 0.0;
            }
        } else if self.position > self.max_position {
            self.position = self.max_position;
            if self.velocity > 0.0 {
                self.velocity = 0.0;
            }
        }
    }

    pub fn is_at_min(&self) -> bool {
        self.position <= self.min_position + 1e-6
    }

    pub fn is_at_max(&self) -> bool {
        self.position >= self.max_position - 1e-6
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.velocity.powi(2)
    }

    pub fn momentum(&self) -> f32 {
        self.mass * self.velocity
    }

    pub fn normalized_position(&self) -> f32 {
        let range = self.max_position - self.min_position;
        if range < 1e-9 {
            return 0.0;
        }
        (self.position - self.min_position) / range
    }

    pub fn stop(&mut self) {
        self.velocity = 0.0;
        self.accumulated_force = 0.0;
    }

    pub fn reset(&mut self) {
        self.position = 0.0;
        self.velocity = 0.0;
        self.accumulated_force = 0.0;
    }

    pub fn travel_range(&self) -> f32 {
        self.max_position - self.min_position
    }
}

pub fn new_sliding_body(mass: f32, min_pos: f32, max_pos: f32) -> SlidingBody {
    SlidingBody::new(mass, min_pos, max_pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn force_moves_body() {
        let mut b = new_sliding_body(1.0, -10.0, 10.0);
        b.apply_force(10.0);
        b.step(1.0);
        assert!(b.position > 0.0);
    }

    #[test]
    fn kinetic_energy() {
        let mut b = new_sliding_body(2.0, -10.0, 10.0);
        b.velocity = 3.0;
        assert!((b.kinetic_energy() - 9.0).abs() < 1e-6);
    }

    #[test]
    fn clamp_at_max() {
        let mut b = new_sliding_body(1.0, 0.0, 1.0);
        b.velocity = 100.0;
        b.step(1.0);
        assert!(b.is_at_max());
        assert!(b.velocity <= 0.0);
    }

    #[test]
    fn clamp_at_min() {
        let mut b = new_sliding_body(1.0, 0.0, 1.0);
        b.position = 0.5;
        b.velocity = -100.0;
        b.step(1.0);
        assert!(b.is_at_min());
    }

    #[test]
    fn friction_decelerates() {
        let mut b = new_sliding_body(1.0, -10.0, 10.0);
        b.velocity = 5.0;
        b.friction_coeff = 0.5;
        b.normal_force = 10.0;
        b.step(0.1);
        assert!(b.velocity < 5.0);
    }

    #[test]
    fn disabled_no_step() {
        let mut b = new_sliding_body(1.0, -10.0, 10.0);
        b.apply_force(100.0);
        b.enabled = false;
        b.step(1.0);
        assert_eq!(b.position, 0.0);
    }

    #[test]
    fn normalized_position() {
        let mut b = new_sliding_body(1.0, 0.0, 4.0);
        b.position = 2.0;
        assert!((b.normalized_position() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn reset() {
        let mut b = new_sliding_body(1.0, 0.0, 10.0);
        b.position = 5.0;
        b.velocity = 3.0;
        b.reset();
        assert_eq!(b.position, 0.0);
        assert_eq!(b.velocity, 0.0);
    }

    #[test]
    fn momentum() {
        let mut b = new_sliding_body(3.0, -10.0, 10.0);
        b.velocity = 4.0;
        assert!((b.momentum() - 12.0).abs() < 1e-6);
    }
}
