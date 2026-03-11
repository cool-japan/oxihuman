// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Deformable spring with permanent set (plastic deformation) and fatigue tracking.

/// A spring that can deform plastically and accumulate fatigue.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeformableSpring {
    pub rest_length: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub plastic_threshold: f32, // strain fraction above which plastic deformation occurs
    pub plastic_rate: f32,      // rate of permanent set per unit excess strain
    pub fatigue: f32,           // accumulated fatigue [0..1]
    pub max_fatigue: f32,       // fatigue at which spring breaks
}

#[allow(dead_code)]
impl DeformableSpring {
    pub fn new(rest_length: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            rest_length,
            stiffness,
            damping,
            plastic_threshold: 0.1,
            plastic_rate: 0.01,
            fatigue: 0.0,
            max_fatigue: 1.0,
        }
    }

    /// Current spring length from two point positions.
    pub fn current_length(p0: [f32; 3], p1: [f32; 3]) -> f32 {
        let dx = p1[0] - p0[0];
        let dy = p1[1] - p0[1];
        let dz = p1[2] - p0[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Strain = (current - rest) / rest.
    pub fn strain(&self, current: f32) -> f32 {
        if self.rest_length > 1e-9 {
            (current - self.rest_length) / self.rest_length
        } else {
            0.0
        }
    }

    /// Spring force magnitude (positive = tension, negative = compression).
    pub fn force_magnitude(&self, current: f32, velocity: f32) -> f32 {
        let extension = current - self.rest_length;
        -self.stiffness * extension - self.damping * velocity
    }

    /// Update plastic deformation and fatigue given current length.
    pub fn update(&mut self, current: f32, dt: f32) {
        let strain = self.strain(current).abs();
        if strain > self.plastic_threshold {
            let excess = strain - self.plastic_threshold;
            // Adjust rest length toward current (permanent set)
            self.rest_length += (current - self.rest_length) * excess * self.plastic_rate * dt;
            // Accumulate fatigue
            self.fatigue += excess * dt;
            self.fatigue = self.fatigue.min(self.max_fatigue);
        }
    }

    pub fn is_broken(&self) -> bool {
        self.fatigue >= self.max_fatigue
    }

    pub fn effective_stiffness(&self) -> f32 {
        self.stiffness * (1.0 - self.fatigue / self.max_fatigue).max(0.0)
    }

    pub fn reset_fatigue(&mut self) {
        self.fatigue = 0.0;
    }
}

/// Compute spring force vector between two points.
#[allow(dead_code)]
pub fn spring_force_vector(
    p0: [f32; 3],
    p1: [f32; 3],
    rest_length: f32,
    stiffness: f32,
) -> [f32; 3] {
    let dx = p1[0] - p0[0];
    let dy = p1[1] - p0[1];
    let dz = p1[2] - p0[2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt();
    if len < 1e-9 {
        return [0.0; 3];
    }
    let force_mag = stiffness * (len - rest_length);
    [
        dx / len * force_mag,
        dy / len * force_mag,
        dz / len * force_mag,
    ]
}

/// Natural frequency of a spring-mass system.
#[allow(dead_code)]
pub fn natural_frequency(stiffness: f32, mass: f32) -> f32 {
    if mass > 1e-9 {
        (stiffness / mass).sqrt()
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn new_spring_no_fatigue() {
        let s = DeformableSpring::new(1.0, 100.0, 1.0);
        assert_eq!(s.fatigue, 0.0);
        assert!(!s.is_broken());
    }

    #[test]
    fn strain_zero_at_rest() {
        let s = DeformableSpring::new(1.0, 100.0, 0.0);
        assert!(s.strain(1.0).abs() < 1e-6);
    }

    #[test]
    fn strain_positive_when_extended() {
        let s = DeformableSpring::new(1.0, 100.0, 0.0);
        assert!(s.strain(1.5) > 0.0);
    }

    #[test]
    fn force_magnitude_restoring() {
        let s = DeformableSpring::new(1.0, 100.0, 0.0);
        // Extended to 1.1: force should be negative (restoring toward shorter)
        assert!(s.force_magnitude(1.1, 0.0) < 0.0);
    }

    #[test]
    fn update_accumulates_fatigue_at_high_strain() {
        let mut s = DeformableSpring::new(1.0, 100.0, 0.0);
        s.plastic_threshold = 0.05;
        s.plastic_rate = 1.0;
        s.update(1.2, 0.1); // 20% strain >> 5% threshold
        assert!(s.fatigue > 0.0);
    }

    #[test]
    fn effective_stiffness_reduces_with_fatigue() {
        let mut s = DeformableSpring::new(1.0, 100.0, 0.0);
        s.fatigue = 0.5;
        assert!(s.effective_stiffness() < 100.0);
    }

    #[test]
    fn spring_force_vector_direction() {
        let f = spring_force_vector([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], 1.0, 10.0);
        // Spring is extended 1 unit; force pulls p0 toward p1 (+x direction)
        assert!(f[0] > 0.0);
    }

    #[test]
    fn natural_frequency_formula() {
        // k=4, m=1 → omega = 2 rad/s → period = PI
        let omega = natural_frequency(4.0, 1.0);
        assert!((omega - 2.0).abs() < 1e-5);
        let period = 2.0 * PI / omega;
        assert!((period - PI).abs() < 1e-4);
    }

    #[test]
    fn reset_fatigue() {
        let mut s = DeformableSpring::new(1.0, 100.0, 0.0);
        s.fatigue = 0.8;
        s.reset_fatigue();
        assert_eq!(s.fatigue, 0.0);
    }

    #[test]
    fn broken_when_max_fatigue() {
        let mut s = DeformableSpring::new(1.0, 100.0, 0.0);
        s.fatigue = s.max_fatigue;
        assert!(s.is_broken());
    }
}
