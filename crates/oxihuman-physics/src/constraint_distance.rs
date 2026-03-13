// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Distance constraint: maintains a target distance between two particles.

/// A distance constraint between two particles.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintDistanceDef {
    pub particle_a: usize,
    pub particle_b: usize,
    pub rest_length: f32,
    pub stiffness: f32,
    pub damping: f32,
}

#[allow(dead_code)]
fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
impl ConstraintDistanceDef {
    pub fn new(a: usize, b: usize, rest_length: f32, stiffness: f32) -> Self {
        Self { particle_a: a, particle_b: b, rest_length, stiffness, damping: 0.0 }
    }

    pub fn with_damping(mut self, damping: f32) -> Self {
        self.damping = damping;
        self
    }

    /// Compute the constraint error (current length - rest length).
    pub fn error(&self, positions: &[[f32; 3]]) -> f32 {
        let d = dist3(positions[self.particle_a], positions[self.particle_b]);
        d - self.rest_length
    }

    /// Apply position-based constraint correction.
    pub fn solve(&self, positions: &mut [[f32; 3]], inv_masses: &[f32]) {
        let pa = positions[self.particle_a];
        let pb = positions[self.particle_b];
        let dx = pb[0] - pa[0];
        let dy = pb[1] - pa[1];
        let dz = pb[2] - pa[2];
        let d = (dx * dx + dy * dy + dz * dz).sqrt();
        if d < 1e-10 { return; }
        let err = d - self.rest_length;
        let wa = inv_masses[self.particle_a];
        let wb = inv_masses[self.particle_b];
        let w_sum = wa + wb;
        if w_sum < 1e-10 { return; }
        let correction = self.stiffness * err / (d * w_sum);
        let cx = dx * correction;
        let cy = dy * correction;
        let cz = dz * correction;
        positions[self.particle_a][0] += wa * cx;
        positions[self.particle_a][1] += wa * cy;
        positions[self.particle_a][2] += wa * cz;
        positions[self.particle_b][0] -= wb * cx;
        positions[self.particle_b][1] -= wb * cy;
        positions[self.particle_b][2] -= wb * cz;
    }

    /// Compute spring-like force magnitude.
    pub fn force_magnitude(&self, positions: &[[f32; 3]]) -> f32 {
        let err = self.error(positions);
        (self.stiffness * err).abs()
    }

    /// Energy stored in this constraint.
    pub fn energy(&self, positions: &[[f32; 3]]) -> f32 {
        let err = self.error(positions);
        0.5 * self.stiffness * err * err
    }

    pub fn current_length(&self, positions: &[[f32; 3]]) -> f32 {
        dist3(positions[self.particle_a], positions[self.particle_b])
    }

    pub fn is_stretched(&self, positions: &[[f32; 3]]) -> bool {
        self.current_length(positions) > self.rest_length
    }

    pub fn is_compressed(&self, positions: &[[f32; 3]]) -> bool {
        self.current_length(positions) < self.rest_length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error() {
        let c = ConstraintDistanceDef::new(0, 1, 1.0, 1.0);
        let pos = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        assert!((c.error(&pos) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_solve_reduces_error() {
        let c = ConstraintDistanceDef::new(0, 1, 1.0, 1.0);
        let mut pos = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let inv_m = [1.0, 1.0];
        let err_before = c.error(&pos).abs();
        c.solve(&mut pos, &inv_m);
        let err_after = c.error(&pos).abs();
        assert!(err_after < err_before);
    }

    #[test]
    fn test_at_rest() {
        let c = ConstraintDistanceDef::new(0, 1, 1.0, 1.0);
        let pos = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        assert!(c.error(&pos).abs() < 0.01);
    }

    #[test]
    fn test_energy_at_rest() {
        let c = ConstraintDistanceDef::new(0, 1, 1.0, 1.0);
        let pos = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        assert!(c.energy(&pos) < 0.001);
    }

    #[test]
    fn test_energy_stretched() {
        let c = ConstraintDistanceDef::new(0, 1, 1.0, 1.0);
        let pos = [[0.0, 0.0, 0.0], [3.0, 0.0, 0.0]];
        assert!(c.energy(&pos) > 0.0);
    }

    #[test]
    fn test_is_stretched() {
        let c = ConstraintDistanceDef::new(0, 1, 1.0, 1.0);
        let pos = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        assert!(c.is_stretched(&pos));
    }

    #[test]
    fn test_is_compressed() {
        let c = ConstraintDistanceDef::new(0, 1, 2.0, 1.0);
        let pos = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        assert!(c.is_compressed(&pos));
    }

    #[test]
    fn test_fixed_particle() {
        let c = ConstraintDistanceDef::new(0, 1, 1.0, 1.0);
        let mut pos = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let inv_m = [0.0, 1.0]; // particle 0 is fixed
        c.solve(&mut pos, &inv_m);
        assert!((pos[0][0]).abs() < 0.001);
    }

    #[test]
    fn test_force_magnitude() {
        let c = ConstraintDistanceDef::new(0, 1, 1.0, 10.0);
        let pos = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        assert!((c.force_magnitude(&pos) - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_current_length() {
        let c = ConstraintDistanceDef::new(0, 1, 1.0, 1.0);
        let pos = [[0.0, 0.0, 0.0], [3.0, 4.0, 0.0]];
        assert!((c.current_length(&pos) - 5.0).abs() < 0.01);
    }
}
