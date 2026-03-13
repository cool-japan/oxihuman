// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A linear actuator with position, velocity, and force limits.
#[allow(dead_code)]
pub struct LinearActuator {
    pub position: f32,
    pub velocity: f32,
    pub target: f32,
    pub min_pos: f32,
    pub max_pos: f32,
    pub max_force: f32,
    pub stiffness: f32,
    pub damping: f32,
}

#[allow(dead_code)]
impl LinearActuator {
    pub fn new(min_pos: f32, max_pos: f32, max_force: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            position: 0.0,
            velocity: 0.0,
            target: 0.0,
            min_pos,
            max_pos,
            max_force,
            stiffness,
            damping,
        }
    }
    pub fn set_target(&mut self, t: f32) {
        self.target = t.clamp(self.min_pos, self.max_pos);
    }
    pub fn step(&mut self, dt: f32) -> f32 {
        let error = self.target - self.position;
        let force = (self.stiffness * error - self.damping * self.velocity)
            .clamp(-self.max_force, self.max_force);
        self.velocity += force * dt;
        self.position = (self.position + self.velocity * dt).clamp(self.min_pos, self.max_pos);
        force
    }
    pub fn at_target(&self, tol: f32) -> bool {
        (self.position - self.target).abs() <= tol
    }
    pub fn range(&self) -> f32 {
        self.max_pos - self.min_pos
    }
    pub fn normalized_pos(&self) -> f32 {
        let r = self.range();
        if r < 1e-8 {
            0.0
        } else {
            (self.position - self.min_pos) / r
        }
    }
    pub fn kinetic_energy(&self, mass: f32) -> f32 {
        0.5 * mass * self.velocity * self.velocity
    }
    pub fn potential_energy(&self) -> f32 {
        let err = self.target - self.position;
        0.5 * self.stiffness * err * err
    }
    pub fn reset(&mut self) {
        self.position = 0.0;
        self.velocity = 0.0;
        self.target = 0.0;
    }
}

#[allow(dead_code)]
pub fn new_linear_actuator(
    min_pos: f32,
    max_pos: f32,
    max_force: f32,
    stiffness: f32,
    damping: f32,
) -> LinearActuator {
    LinearActuator::new(min_pos, max_pos, max_force, stiffness, damping)
}
#[allow(dead_code)]
pub fn la_set_target(a: &mut LinearActuator, t: f32) {
    a.set_target(t);
}
#[allow(dead_code)]
pub fn la_step(a: &mut LinearActuator, dt: f32) -> f32 {
    a.step(dt)
}
#[allow(dead_code)]
pub fn la_at_target(a: &LinearActuator, tol: f32) -> bool {
    a.at_target(tol)
}
#[allow(dead_code)]
pub fn la_normalized_pos(a: &LinearActuator) -> f32 {
    a.normalized_pos()
}
#[allow(dead_code)]
pub fn la_kinetic_energy(a: &LinearActuator, mass: f32) -> f32 {
    a.kinetic_energy(mass)
}
#[allow(dead_code)]
pub fn la_potential_energy(a: &LinearActuator) -> f32 {
    a.potential_energy()
}
#[allow(dead_code)]
pub fn la_reset(a: &mut LinearActuator) {
    a.reset();
}
#[allow(dead_code)]
pub fn la_range(a: &LinearActuator) -> f32 {
    a.range()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_moves_toward_target() {
        let mut a = new_linear_actuator(-1.0, 1.0, 100.0, 50.0, 5.0);
        la_set_target(&mut a, 1.0);
        for _ in 0..50 {
            la_step(&mut a, 0.02);
        }
        assert!(a.position > 0.5);
    }
    #[test]
    fn test_clamped_to_range() {
        let mut a = new_linear_actuator(0.0, 1.0, 100.0, 100.0, 1.0);
        la_set_target(&mut a, 5.0);
        for _ in 0..100 {
            la_step(&mut a, 0.02);
        }
        assert!(a.position <= 1.0 + 1e-5);
    }
    #[test]
    fn test_at_target() {
        let mut a = new_linear_actuator(-1.0, 1.0, 100.0, 200.0, 20.0);
        la_set_target(&mut a, 0.0);
        for _ in 0..200 {
            la_step(&mut a, 0.01);
        }
        assert!(la_at_target(&a, 0.05));
    }
    #[test]
    fn test_normalized_pos() {
        let mut a = new_linear_actuator(0.0, 2.0, 100.0, 100.0, 10.0);
        a.position = 1.0;
        let n = la_normalized_pos(&a);
        assert!((n - 0.5).abs() < 1e-5);
    }
    #[test]
    fn test_kinetic_energy_zero_at_rest() {
        let a = new_linear_actuator(-1.0, 1.0, 100.0, 10.0, 5.0);
        assert_eq!(la_kinetic_energy(&a, 1.0), 0.0);
    }
    #[test]
    fn test_potential_energy_at_target() {
        let mut a = new_linear_actuator(-1.0, 1.0, 100.0, 10.0, 5.0);
        a.position = 0.0;
        a.target = 0.0;
        assert_eq!(la_potential_energy(&a), 0.0);
    }
    #[test]
    fn test_range() {
        let a = new_linear_actuator(-2.0, 3.0, 10.0, 1.0, 0.1);
        assert!((la_range(&a) - 5.0).abs() < 1e-5);
    }
    #[test]
    fn test_reset() {
        let mut a = new_linear_actuator(-1.0, 1.0, 100.0, 10.0, 5.0);
        la_set_target(&mut a, 1.0);
        la_step(&mut a, 0.1);
        la_reset(&mut a);
        assert_eq!(a.position, 0.0);
        assert_eq!(a.velocity, 0.0);
    }
    #[test]
    fn test_target_clamped_to_range() {
        let mut a = new_linear_actuator(0.0, 1.0, 100.0, 10.0, 1.0);
        la_set_target(&mut a, 100.0);
        assert_eq!(a.target, 1.0);
    }
    #[test]
    fn test_force_limited() {
        let mut a = new_linear_actuator(-1.0, 1.0, 1.0, 1000.0, 0.0);
        let f = la_step(&mut a, 0.01);
        assert!(f.abs() <= 1.0 + 1e-5);
    }
}
