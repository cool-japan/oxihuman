// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Point constraint: pins a particle to a fixed target position.

/// A constraint that pulls a particle toward a target position.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintPointDef {
    pub particle: usize,
    pub target: [f32; 3],
    pub stiffness: f32,
    pub max_force: f32,
}

#[allow(dead_code)]
fn v3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn v3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[allow(dead_code)]
fn v3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
impl ConstraintPointDef {
    pub fn new(particle: usize, target: [f32; 3], stiffness: f32) -> Self {
        Self { particle, target, stiffness, max_force: f32::MAX }
    }

    pub fn with_max_force(mut self, max_force: f32) -> Self {
        self.max_force = max_force;
        self
    }

    pub fn error(&self, positions: &[[f32; 3]]) -> f32 {
        v3_len(v3_sub(positions[self.particle], self.target))
    }

    pub fn error_vec(&self, positions: &[[f32; 3]]) -> [f32; 3] {
        v3_sub(positions[self.particle], self.target)
    }

    pub fn solve(&self, positions: &mut [[f32; 3]], inv_masses: &[f32]) {
        let w = inv_masses[self.particle];
        if w < 1e-10 { return; }
        let diff = v3_sub(self.target, positions[self.particle]);
        let dist = v3_len(diff);
        if dist < 1e-10 { return; }
        let force = (self.stiffness * dist).min(self.max_force);
        let correction = force / dist;
        positions[self.particle][0] += diff[0] * correction * w;
        positions[self.particle][1] += diff[1] * correction * w;
        positions[self.particle][2] += diff[2] * correction * w;
    }

    pub fn energy(&self, positions: &[[f32; 3]]) -> f32 {
        let e = self.error(positions);
        0.5 * self.stiffness * e * e
    }

    pub fn force_magnitude(&self, positions: &[[f32; 3]]) -> f32 {
        (self.stiffness * self.error(positions)).min(self.max_force)
    }

    pub fn set_target(&mut self, target: [f32; 3]) {
        self.target = target;
    }

    pub fn is_satisfied(&self, positions: &[[f32; 3]], tolerance: f32) -> bool {
        self.error(positions) < tolerance
    }

    /// Direction from particle to target, normalized.
    pub fn direction(&self, positions: &[[f32; 3]]) -> [f32; 3] {
        let diff = v3_sub(self.target, positions[self.particle]);
        let d = v3_len(diff);
        if d < 1e-10 { return [0.0; 3]; }
        v3_scale(diff, 1.0 / d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error() {
        let c = ConstraintPointDef::new(0, [1.0, 0.0, 0.0], 1.0);
        let pos = [[0.0, 0.0, 0.0]];
        assert!((c.error(&pos) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_at_target() {
        let c = ConstraintPointDef::new(0, [1.0, 0.0, 0.0], 1.0);
        let pos = [[1.0, 0.0, 0.0]];
        assert!(c.error(&pos) < 0.001);
    }

    #[test]
    fn test_solve_moves_toward() {
        let c = ConstraintPointDef::new(0, [2.0, 0.0, 0.0], 1.0);
        let mut pos = [[0.0, 0.0, 0.0]];
        let inv_m = [1.0];
        let err_before = c.error(&pos);
        c.solve(&mut pos, &inv_m);
        let err_after = c.error(&pos);
        assert!(err_after < err_before);
    }

    #[test]
    fn test_fixed_mass() {
        let c = ConstraintPointDef::new(0, [5.0, 0.0, 0.0], 1.0);
        let mut pos = [[0.0, 0.0, 0.0]];
        let inv_m = [0.0]; // infinite mass
        c.solve(&mut pos, &inv_m);
        assert!((pos[0][0]).abs() < 0.001);
    }

    #[test]
    fn test_energy() {
        let c = ConstraintPointDef::new(0, [0.0, 0.0, 0.0], 10.0);
        let pos = [[1.0, 0.0, 0.0]];
        assert!((c.energy(&pos) - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_max_force() {
        let c = ConstraintPointDef::new(0, [100.0, 0.0, 0.0], 10.0).with_max_force(5.0);
        let pos = [[0.0, 0.0, 0.0]];
        assert!((c.force_magnitude(&pos) - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_direction() {
        let c = ConstraintPointDef::new(0, [1.0, 0.0, 0.0], 1.0);
        let pos = [[0.0, 0.0, 0.0]];
        let d = c.direction(&pos);
        assert!((d[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_is_satisfied() {
        let c = ConstraintPointDef::new(0, [0.0, 0.0, 0.0], 1.0);
        let pos = [[0.001, 0.0, 0.0]];
        assert!(c.is_satisfied(&pos, 0.01));
        assert!(!c.is_satisfied(&pos, 0.0001));
    }

    #[test]
    fn test_set_target() {
        let mut c = ConstraintPointDef::new(0, [0.0; 3], 1.0);
        c.set_target([5.0, 0.0, 0.0]);
        assert!((c.target[0] - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_error_vec() {
        let c = ConstraintPointDef::new(0, [0.0; 3], 1.0);
        let pos = [[3.0, 4.0, 0.0]];
        let ev = c.error_vec(&pos);
        assert!((ev[0] - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_3d_distance() {
        let c = ConstraintPointDef::new(0, [0.0; 3], 1.0);
        let pos = [[3.0, 4.0, 0.0]];
        assert!((c.error(&pos) - 5.0).abs() < 0.01);
    }
}
