// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A yoke joint connecting two rigid bodies with shared positional anchor.

/// A yoke joint that constrains two bodies to share an anchor point.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct YokeJoint {
    pub body_a: usize,
    pub body_b: usize,
    /// Anchor position in world space.
    pub anchor: [f32; 3],
    /// Compliance (softness).
    pub compliance: f32,
    pub enabled: bool,
    pub constraint_force: [f32; 3],
    pub iterations: u64,
}

#[allow(dead_code)]
impl YokeJoint {
    pub fn new(body_a: usize, body_b: usize, anchor: [f32; 3], compliance: f32) -> Self {
        Self {
            body_a,
            body_b,
            anchor,
            compliance: compliance.max(0.0),
            enabled: true,
            constraint_force: [0.0; 3],
            iterations: 0,
        }
    }

    /// Solve the joint: returns correction vectors for body_a and body_b.
    #[allow(clippy::too_many_arguments)]
    pub fn solve(
        &mut self,
        pos_a: [f32; 3],
        pos_b: [f32; 3],
        inv_mass_a: f32,
        inv_mass_b: f32,
        dt: f32,
    ) -> ([f32; 3], [f32; 3]) {
        if !self.enabled {
            return ([0.0; 3], [0.0; 3]);
        }
        // Error: distance of each body from anchor.
        let err_a = [
            pos_a[0] - self.anchor[0],
            pos_a[1] - self.anchor[1],
            pos_a[2] - self.anchor[2],
        ];
        let err_b = [
            pos_b[0] - self.anchor[0],
            pos_b[1] - self.anchor[1],
            pos_b[2] - self.anchor[2],
        ];
        let alpha = self.compliance / (dt * dt);
        let w_sum = inv_mass_a + inv_mass_b + alpha;
        let correction_scale = if w_sum > 1e-9 { 1.0 / w_sum } else { 0.0 };

        let corr_a = [
            -err_a[0] * inv_mass_a * correction_scale,
            -err_a[1] * inv_mass_a * correction_scale,
            -err_a[2] * inv_mass_a * correction_scale,
        ];
        let corr_b = [
            -err_b[0] * inv_mass_b * correction_scale,
            -err_b[1] * inv_mass_b * correction_scale,
            -err_b[2] * inv_mass_b * correction_scale,
        ];
        self.constraint_force = [
            (err_a[0] + err_b[0]) * 0.5,
            (err_a[1] + err_b[1]) * 0.5,
            (err_a[2] + err_b[2]) * 0.5,
        ];
        self.iterations += 1;
        (corr_a, corr_b)
    }

    pub fn set_anchor(&mut self, anchor: [f32; 3]) {
        self.anchor = anchor;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn constraint_error(&self, pos_a: [f32; 3], pos_b: [f32; 3]) -> f32 {
        let ea = [
            pos_a[0] - self.anchor[0],
            pos_a[1] - self.anchor[1],
            pos_a[2] - self.anchor[2],
        ];
        let eb = [
            pos_b[0] - self.anchor[0],
            pos_b[1] - self.anchor[1],
            pos_b[2] - self.anchor[2],
        ];
        let ea_mag = (ea[0] * ea[0] + ea[1] * ea[1] + ea[2] * ea[2]).sqrt();
        let eb_mag = (eb[0] * eb[0] + eb[1] * eb[1] + eb[2] * eb[2]).sqrt();
        ea_mag + eb_mag
    }

    pub fn reset(&mut self) {
        self.constraint_force = [0.0; 3];
        self.iterations = 0;
    }
}

impl Default for YokeJoint {
    fn default() -> Self {
        Self::new(0, 1, [0.0; 3], 0.0)
    }
}

pub fn new_yoke_joint(body_a: usize, body_b: usize, anchor: [f32; 3]) -> YokeJoint {
    YokeJoint::new(body_a, body_b, anchor, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_error_at_anchor() {
        let j = new_yoke_joint(0, 1, [1.0, 2.0, 3.0]);
        let err = j.constraint_error([1.0, 2.0, 3.0], [1.0, 2.0, 3.0]);
        assert!(err < 1e-5);
    }

    #[test]
    fn nonzero_error_when_displaced() {
        let j = new_yoke_joint(0, 1, [0.0; 3]);
        let err = j.constraint_error([1.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert!(err > 0.9);
    }

    #[test]
    fn solve_produces_correction() {
        let mut j = new_yoke_joint(0, 1, [0.0; 3]);
        let (ca, _) = j.solve([1.0, 0.0, 0.0], [0.0; 3], 1.0, 1.0, 0.016);
        assert!(ca.iter().any(|&x| x.abs() > 1e-5));
    }

    #[test]
    fn disabled_joint_no_correction() {
        let mut j = new_yoke_joint(0, 1, [0.0; 3]);
        j.set_enabled(false);
        let (ca, cb) = j.solve([5.0, 0.0, 0.0], [3.0, 0.0, 0.0], 1.0, 1.0, 0.016);
        assert!(ca.iter().all(|&x| x.abs() < 1e-5));
        assert!(cb.iter().all(|&x| x.abs() < 1e-5));
    }

    #[test]
    fn iterations_increment() {
        let mut j = new_yoke_joint(0, 1, [0.0; 3]);
        j.solve([0.0; 3], [0.0; 3], 1.0, 1.0, 0.016);
        j.solve([0.0; 3], [0.0; 3], 1.0, 1.0, 0.016);
        assert_eq!(j.iterations, 2);
    }

    #[test]
    fn reset_clears_iterations() {
        let mut j = new_yoke_joint(0, 1, [0.0; 3]);
        j.solve([0.0; 3], [0.0; 3], 1.0, 1.0, 0.016);
        j.reset();
        assert_eq!(j.iterations, 0);
    }

    #[test]
    fn set_anchor() {
        let mut j = new_yoke_joint(0, 1, [0.0; 3]);
        j.set_anchor([5.0, 5.0, 5.0]);
        assert_eq!(j.anchor, [5.0, 5.0, 5.0]);
    }

    #[test]
    fn compliance_softens() {
        let mut j_hard = YokeJoint::new(0, 1, [0.0; 3], 0.0);
        let mut j_soft = YokeJoint::new(0, 1, [0.0; 3], 10.0);
        let (ca_hard, _) = j_hard.solve([1.0, 0.0, 0.0], [0.0; 3], 1.0, 1.0, 0.016);
        let (ca_soft, _) = j_soft.solve([1.0, 0.0, 0.0], [0.0; 3], 1.0, 1.0, 0.016);
        assert!(ca_hard[0].abs() >= ca_soft[0].abs());
    }

    #[test]
    fn default_enabled() {
        let j = YokeJoint::default();
        assert!(j.enabled);
    }

    #[test]
    fn body_ids_set() {
        let j = new_yoke_joint(3, 7, [0.0; 3]);
        assert_eq!(j.body_a, 3);
        assert_eq!(j.body_b, 7);
    }
}
