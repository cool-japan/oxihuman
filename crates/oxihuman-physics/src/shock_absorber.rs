// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Shock absorber: spring-damper with compression/rebound settings.

/// Shock absorber combining spring stiffness and viscous damping.
#[derive(Debug, Clone)]
pub struct ShockAbsorber {
    pub spring_constant: f32,
    pub compression_damping: f32,
    pub rebound_damping: f32,
    pub rest_length: f32,
    pub current_length: f32,
    pub velocity: f32,
    pub min_length: f32,
    pub max_length: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
impl ShockAbsorber {
    pub fn new(
        spring_constant: f32,
        compression_damping: f32,
        rebound_damping: f32,
        rest_length: f32,
    ) -> Self {
        let min = rest_length * 0.5;
        let max = rest_length * 1.5;
        ShockAbsorber {
            spring_constant,
            compression_damping,
            rebound_damping,
            rest_length,
            current_length: rest_length,
            velocity: 0.0,
            min_length: min,
            max_length: max,
            enabled: true,
        }
    }

    /// Compute the force produced (positive = extension force).
    pub fn force(&self) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        let displacement = self.current_length - self.rest_length;
        let spring_force = -self.spring_constant * displacement;
        let damping = if self.velocity < 0.0 {
            self.compression_damping
        } else {
            self.rebound_damping
        };
        spring_force - damping * self.velocity
    }

    pub fn step(&mut self, dt: f32, external_force: f32) {
        if !self.enabled {
            return;
        }
        let f = self.force() + external_force;
        self.velocity += f * dt;
        self.current_length += self.velocity * dt;
        self.current_length = self.current_length.clamp(self.min_length, self.max_length);
    }

    pub fn compression(&self) -> f32 {
        (self.rest_length - self.current_length).max(0.0)
    }

    pub fn extension(&self) -> f32 {
        (self.current_length - self.rest_length).max(0.0)
    }

    pub fn is_compressed(&self) -> bool {
        self.current_length < self.rest_length - 1e-6
    }

    pub fn is_extended(&self) -> bool {
        self.current_length > self.rest_length + 1e-6
    }

    pub fn energy_stored(&self) -> f32 {
        let disp = self.current_length - self.rest_length;
        0.5 * self.spring_constant * disp.powi(2)
    }

    pub fn travel(&self) -> f32 {
        self.max_length - self.min_length
    }

    pub fn set_rest_length(&mut self, l: f32) {
        self.rest_length = l.max(1e-6);
    }

    pub fn reset(&mut self) {
        self.current_length = self.rest_length;
        self.velocity = 0.0;
    }
}

pub fn new_shock_absorber(k: f32, cd: f32, rd: f32, rest: f32) -> ShockAbsorber {
    ShockAbsorber::new(k, cd, rd, rest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rest_no_force() {
        let s = new_shock_absorber(100.0, 10.0, 5.0, 0.3);
        assert!(s.force().abs() < 1e-6);
    }

    #[test]
    fn compression_produces_positive_force() {
        let mut s = new_shock_absorber(100.0, 10.0, 5.0, 0.3);
        s.current_length = 0.2;
        assert!(s.force() > 0.0);
    }

    #[test]
    fn extension_produces_negative_force() {
        let mut s = new_shock_absorber(100.0, 10.0, 5.0, 0.3);
        s.current_length = 0.4;
        assert!(s.force() < 0.0);
    }

    #[test]
    fn energy_stored_at_rest_zero() {
        let s = new_shock_absorber(100.0, 5.0, 5.0, 0.3);
        assert!(s.energy_stored() < 1e-10);
    }

    #[test]
    fn compression_measurement() {
        let mut s = new_shock_absorber(100.0, 5.0, 5.0, 0.3);
        s.current_length = 0.25;
        assert!((s.compression() - 0.05).abs() < 1e-6);
    }

    #[test]
    fn step_moves_toward_rest() {
        let mut s = new_shock_absorber(100.0, 20.0, 20.0, 0.3);
        s.current_length = 0.2;
        let before = s.current_length;
        s.step(0.01, 0.0);
        assert!(s.current_length > before);
    }

    #[test]
    fn disabled_zero_force() {
        let mut s = new_shock_absorber(100.0, 5.0, 5.0, 0.3);
        s.current_length = 0.1;
        s.enabled = false;
        assert_eq!(s.force(), 0.0);
    }

    #[test]
    fn reset_restores() {
        let mut s = new_shock_absorber(100.0, 5.0, 5.0, 0.3);
        s.step(1.0, -100.0);
        s.reset();
        assert!((s.current_length - s.rest_length).abs() < 1e-6);
    }

    #[test]
    fn travel_range() {
        let s = new_shock_absorber(100.0, 5.0, 5.0, 0.4);
        assert!((s.travel() - 0.4).abs() < 1e-6);
    }
}
