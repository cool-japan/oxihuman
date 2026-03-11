// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A pivot (point-to-point) joint constraining two bodies to share a point.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct PivotJoint {
    local_anchor_a: [f32; 3],
    local_anchor_b: [f32; 3],
    stiffness: f32,
    damping: f32,
    max_force: f32,
    breaking_force: f32,
    broken: bool,
}

#[allow(dead_code)]
impl PivotJoint {
    pub fn new(local_anchor_a: [f32; 3], local_anchor_b: [f32; 3]) -> Self {
        Self {
            local_anchor_a,
            local_anchor_b,
            stiffness: 1000.0,
            damping: 50.0,
            max_force: f32::MAX,
            breaking_force: f32::MAX,
            broken: false,
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

    pub fn with_breaking_force(mut self, force: f32) -> Self {
        self.breaking_force = force;
        self
    }

    pub fn is_broken(&self) -> bool {
        self.broken
    }

    pub fn local_anchor_a(&self) -> [f32; 3] {
        self.local_anchor_a
    }

    pub fn local_anchor_b(&self) -> [f32; 3] {
        self.local_anchor_b
    }

    pub fn compute_correction(
        &mut self,
        world_anchor_a: [f32; 3],
        world_anchor_b: [f32; 3],
        velocity_a: [f32; 3],
        velocity_b: [f32; 3],
    ) -> ([f32; 3], [f32; 3]) {
        if self.broken {
            return ([0.0; 3], [0.0; 3]);
        }
        let mut force_a = [0.0f32; 3];
        let mut force_b = [0.0f32; 3];
        let mut force_mag_sq = 0.0f32;
        for i in 0..3 {
            let diff = world_anchor_b[i] - world_anchor_a[i];
            let rel_vel = velocity_b[i] - velocity_a[i];
            let f = diff * self.stiffness + rel_vel * self.damping;
            let f = f.clamp(-self.max_force, self.max_force);
            force_a[i] = f;
            force_b[i] = -f;
            force_mag_sq += f * f;
        }
        if force_mag_sq > self.breaking_force * self.breaking_force {
            self.broken = true;
            return ([0.0; 3], [0.0; 3]);
        }
        (force_a, force_b)
    }

    pub fn error(&self, world_anchor_a: [f32; 3], world_anchor_b: [f32; 3]) -> f32 {
        let dx = world_anchor_b[0] - world_anchor_a[0];
        let dy = world_anchor_b[1] - world_anchor_a[1];
        let dz = world_anchor_b[2] - world_anchor_a[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn is_satisfied(&self, world_anchor_a: [f32; 3], world_anchor_b: [f32; 3], tolerance: f32) -> bool {
        self.error(world_anchor_a, world_anchor_b) <= tolerance
    }

    pub fn reset(&mut self) {
        self.broken = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let j = PivotJoint::new([0.0; 3], [0.0; 3]);
        assert!(!j.is_broken());
    }

    #[test]
    fn test_zero_error_no_force() {
        let mut j = PivotJoint::new([0.0; 3], [0.0; 3]);
        let (fa, fb) = j.compute_correction([1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        for i in 0..3 {
            assert!(fa[i].abs() < 1e-5);
            assert!(fb[i].abs() < 1e-5);
        }
    }

    #[test]
    fn test_error_produces_force() {
        let mut j = PivotJoint::new([0.0; 3], [0.0; 3]);
        let (fa, _fb) = j.compute_correction([0.0; 3], [1.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        assert!(fa[0] > 0.0);
    }

    #[test]
    fn test_forces_opposite() {
        let mut j = PivotJoint::new([0.0; 3], [0.0; 3]);
        let (fa, fb) = j.compute_correction([0.0; 3], [1.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        assert!((fa[0] + fb[0]).abs() < 1e-5);
    }

    #[test]
    fn test_breaking_force() {
        let mut j = PivotJoint::new([0.0; 3], [0.0; 3]).with_breaking_force(1.0);
        let _ = j.compute_correction([0.0; 3], [100.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        assert!(j.is_broken());
    }

    #[test]
    fn test_broken_no_force() {
        let mut j = PivotJoint::new([0.0; 3], [0.0; 3]).with_breaking_force(1.0);
        let _ = j.compute_correction([0.0; 3], [100.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        let (fa, _) = j.compute_correction([0.0; 3], [100.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        assert_eq!(fa, [0.0; 3]);
    }

    #[test]
    fn test_error() {
        let j = PivotJoint::new([0.0; 3], [0.0; 3]);
        let e = j.error([0.0; 3], [3.0, 4.0, 0.0]);
        assert!((e - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_satisfied() {
        let j = PivotJoint::new([0.0; 3], [0.0; 3]);
        assert!(j.is_satisfied([0.0; 3], [0.01, 0.0, 0.0], 0.1));
        assert!(!j.is_satisfied([0.0; 3], [1.0, 0.0, 0.0], 0.1));
    }

    #[test]
    fn test_reset() {
        let mut j = PivotJoint::new([0.0; 3], [0.0; 3]).with_breaking_force(1.0);
        let _ = j.compute_correction([0.0; 3], [100.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        j.reset();
        assert!(!j.is_broken());
    }

    #[test]
    fn test_local_anchors() {
        let j = PivotJoint::new([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        assert_eq!(j.local_anchor_a(), [1.0, 2.0, 3.0]);
        assert_eq!(j.local_anchor_b(), [4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_stiffness_damping() {
        let j = PivotJoint::new([0.0; 3], [0.0; 3])
            .with_stiffness(500.0)
            .with_damping(25.0);
        assert!((j.stiffness - 500.0).abs() < 1e-9);
        assert!((j.damping - 25.0).abs() < 1e-9);
    }
}
