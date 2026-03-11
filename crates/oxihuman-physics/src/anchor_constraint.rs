// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Constrains a body to a fixed anchor point in world space.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnchorConstraint {
    anchor: [f32; 3],
    body_local: [f32; 3],
    stiffness: f32,
    damping: f32,
    max_force: f32,
}

#[allow(dead_code)]
impl AnchorConstraint {
    pub fn new(anchor: [f32; 3], body_local: [f32; 3]) -> Self {
        Self {
            anchor,
            body_local,
            stiffness: 1.0,
            damping: 0.1,
            max_force: f32::MAX,
        }
    }

    pub fn with_stiffness(mut self, stiffness: f32) -> Self {
        self.stiffness = stiffness;
        self
    }

    pub fn with_damping(mut self, damping: f32) -> Self {
        self.damping = damping;
        self
    }

    pub fn with_max_force(mut self, max_force: f32) -> Self {
        self.max_force = max_force;
        self
    }

    pub fn anchor(&self) -> [f32; 3] {
        self.anchor
    }

    pub fn set_anchor(&mut self, pos: [f32; 3]) {
        self.anchor = pos;
    }

    pub fn stiffness(&self) -> f32 {
        self.stiffness
    }

    pub fn damping(&self) -> f32 {
        self.damping
    }

    pub fn compute_force(&self, body_world_pos: [f32; 3], velocity: [f32; 3]) -> [f32; 3] {
        let mut force = [0.0f32; 3];
        for i in 0..3 {
            let displacement = self.anchor[i] - body_world_pos[i];
            let spring = displacement * self.stiffness;
            let damp = -velocity[i] * self.damping;
            force[i] = spring + damp;
        }
        let mag_sq: f32 = force.iter().map(|&f| f * f).sum();
        let mag = mag_sq.sqrt();
        if mag > self.max_force && mag > 1e-9 {
            let scale = self.max_force / mag;
            for f in &mut force {
                *f *= scale;
            }
        }
        force
    }

    pub fn displacement(&self, body_world_pos: [f32; 3]) -> f32 {
        let dx = self.anchor[0] - body_world_pos[0];
        let dy = self.anchor[1] - body_world_pos[1];
        let dz = self.anchor[2] - body_world_pos[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn is_satisfied(&self, body_world_pos: [f32; 3], tolerance: f32) -> bool {
        self.displacement(body_world_pos) <= tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let c = AnchorConstraint::new([1.0, 2.0, 3.0], [0.0, 0.0, 0.0]);
        assert_eq!(c.anchor(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_stiffness() {
        let c = AnchorConstraint::new([0.0; 3], [0.0; 3]).with_stiffness(5.0);
        assert!((c.stiffness() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_damping() {
        let c = AnchorConstraint::new([0.0; 3], [0.0; 3]).with_damping(0.5);
        assert!((c.damping() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_zero_displacement_zero_force() {
        let c = AnchorConstraint::new([1.0, 0.0, 0.0], [0.0; 3]);
        let force = c.compute_force([1.0, 0.0, 0.0], [0.0; 3]);
        for &f in &force {
            assert!(f.abs() < 1e-9);
        }
    }

    #[test]
    fn test_force_toward_anchor() {
        let c = AnchorConstraint::new([10.0, 0.0, 0.0], [0.0; 3]).with_stiffness(2.0);
        let force = c.compute_force([0.0, 0.0, 0.0], [0.0; 3]);
        assert!(force[0] > 0.0);
    }

    #[test]
    fn test_max_force_clamping() {
        let c = AnchorConstraint::new([100.0, 0.0, 0.0], [0.0; 3])
            .with_stiffness(100.0)
            .with_max_force(5.0);
        let force = c.compute_force([0.0, 0.0, 0.0], [0.0; 3]);
        let mag = (force[0] * force[0] + force[1] * force[1] + force[2] * force[2]).sqrt();
        assert!((mag - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_displacement() {
        let c = AnchorConstraint::new([3.0, 4.0, 0.0], [0.0; 3]);
        let d = c.displacement([0.0, 0.0, 0.0]);
        assert!((d - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_satisfied() {
        let c = AnchorConstraint::new([0.0; 3], [0.0; 3]);
        assert!(c.is_satisfied([0.0, 0.0, 0.01], 0.1));
        assert!(!c.is_satisfied([10.0, 0.0, 0.0], 0.1));
    }

    #[test]
    fn test_set_anchor() {
        let mut c = AnchorConstraint::new([0.0; 3], [0.0; 3]);
        c.set_anchor([5.0, 5.0, 5.0]);
        assert_eq!(c.anchor(), [5.0, 5.0, 5.0]);
    }

    #[test]
    fn test_damping_reduces_velocity() {
        let c = AnchorConstraint::new([0.0; 3], [0.0; 3]).with_stiffness(0.0).with_damping(1.0);
        let force = c.compute_force([0.0; 3], [10.0, 0.0, 0.0]);
        assert!(force[0] < 0.0);
    }
}
