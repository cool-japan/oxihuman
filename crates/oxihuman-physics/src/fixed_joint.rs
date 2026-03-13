// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A fixed (weld) joint that constrains two bodies to maintain a rigid relative transform.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedJoint {
    pub body_a: u32,
    pub body_b: u32,
    pub local_anchor_a: [f32; 3],
    pub local_anchor_b: [f32; 3],
    pub compliance: f32,
    pub break_force: f32,
    pub broken: bool,
}

#[allow(dead_code)]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[allow(dead_code)]
fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
impl FixedJoint {
    pub fn new(body_a: u32, body_b: u32) -> Self {
        Self {
            body_a,
            body_b,
            local_anchor_a: [0.0; 3],
            local_anchor_b: [0.0; 3],
            compliance: 0.0,
            break_force: f32::MAX,
            broken: false,
        }
    }

    pub fn with_anchors(mut self, anchor_a: [f32; 3], anchor_b: [f32; 3]) -> Self {
        self.local_anchor_a = anchor_a;
        self.local_anchor_b = anchor_b;
        self
    }

    pub fn with_compliance(mut self, c: f32) -> Self {
        self.compliance = c.max(0.0);
        self
    }

    pub fn with_break_force(mut self, f: f32) -> Self {
        self.break_force = f;
        self
    }

    /// Compute correction given current world-space anchor positions.
    pub fn compute_correction(
        &self,
        world_a: [f32; 3],
        world_b: [f32; 3],
    ) -> [f32; 3] {
        if self.broken {
            return [0.0; 3];
        }
        vec3_sub(world_b, world_a)
    }

    /// Compute correction magnitude.
    pub fn correction_magnitude(&self, world_a: [f32; 3], world_b: [f32; 3]) -> f32 {
        if self.broken {
            return 0.0;
        }
        vec3_len(vec3_sub(world_b, world_a))
    }

    /// Check if the joint should break given a force magnitude.
    pub fn check_break(&mut self, force_magnitude: f32) -> bool {
        if force_magnitude > self.break_force {
            self.broken = true;
        }
        self.broken
    }

    pub fn is_broken(&self) -> bool {
        self.broken
    }

    pub fn repair(&mut self) {
        self.broken = false;
    }

    pub fn is_stiff(&self) -> bool {
        self.compliance <= 0.0
    }

    /// Effective stiffness = 1 / compliance (or infinity for zero compliance).
    pub fn effective_stiffness(&self) -> f32 {
        if self.compliance <= 0.0 {
            f32::MAX
        } else {
            1.0 / self.compliance
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_joint() {
        let j = FixedJoint::new(0, 1);
        assert!(!j.is_broken());
        assert!(j.is_stiff());
    }

    #[test]
    fn test_correction_zero() {
        let j = FixedJoint::new(0, 1);
        let c = j.compute_correction([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(vec3_len(c) < 1e-6);
    }

    #[test]
    fn test_correction_nonzero() {
        let j = FixedJoint::new(0, 1);
        let m = j.correction_magnitude([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
        assert!((m - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_break_force() {
        let mut j = FixedJoint::new(0, 1).with_break_force(10.0);
        assert!(!j.check_break(5.0));
        assert!(j.check_break(15.0));
        assert!(j.is_broken());
    }

    #[test]
    fn test_broken_returns_zero() {
        let mut j = FixedJoint::new(0, 1);
        j.broken = true;
        let c = j.compute_correction([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(vec3_len(c) < 1e-6);
    }

    #[test]
    fn test_repair() {
        let mut j = FixedJoint::new(0, 1);
        j.broken = true;
        j.repair();
        assert!(!j.is_broken());
    }

    #[test]
    fn test_with_compliance() {
        let j = FixedJoint::new(0, 1).with_compliance(0.01);
        assert!(!j.is_stiff());
    }

    #[test]
    fn test_effective_stiffness() {
        let j = FixedJoint::new(0, 1).with_compliance(0.1);
        assert!((j.effective_stiffness() - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_with_anchors() {
        let j = FixedJoint::new(0, 1).with_anchors([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((j.local_anchor_a[0] - 1.0).abs() < f32::EPSILON);
        assert!((j.local_anchor_b[1] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_stiff_has_max_stiffness() {
        let j = FixedJoint::new(0, 1);
        assert_eq!(j.effective_stiffness(), f32::MAX);
    }
}
