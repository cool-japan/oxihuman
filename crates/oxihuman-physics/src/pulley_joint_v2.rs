// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Pulley/block-and-tackle joint — constrains combined rope length.

/// A pulley joint connecting two bodies through a fixed pulley anchor.
#[derive(Debug, Clone)]
pub struct PulleyJointV2 {
    /// Position of the pulley anchor.
    pub anchor: [f64; 3],
    /// Total rope length.
    pub total_length: f64,
    /// Mechanical advantage ratio (> 1 = block-and-tackle).
    pub ratio: f64,
}

impl PulleyJointV2 {
    /// Create a new pulley joint.
    pub fn new(anchor: [f64; 3], total_length: f64, ratio: f64) -> Self {
        PulleyJointV2 { anchor, total_length, ratio: ratio.max(1.0) }
    }

    /// Compute the constraint violation given two body positions.
    ///
    /// Violation = dist(anchor, pa) + ratio * dist(anchor, pb) - total_length
    pub fn violation(&self, pa: [f64; 3], pb: [f64; 3]) -> f64 {
        let da = dist3(self.anchor, pa);
        let db = dist3(self.anchor, pb);
        da + self.ratio * db - self.total_length
    }

    /// True if the constraint is satisfied (within tolerance).
    pub fn is_satisfied(&self, pa: [f64; 3], pb: [f64; 3], tol: f64) -> bool {
        self.violation(pa, pb).abs() <= tol
    }

    /// Distance from anchor to point `pa`.
    pub fn rope_length_a(&self, pa: [f64; 3]) -> f64 {
        dist3(self.anchor, pa)
    }

    /// Distance from anchor to point `pb` (scaled by ratio).
    pub fn rope_length_b(&self, pb: [f64; 3]) -> f64 {
        dist3(self.anchor, pb) * self.ratio
    }

    /// Force transmitted to body A when body B has force `fb`.
    pub fn force_ratio(&self, fb: f64) -> f64 {
        fb / self.ratio
    }
}

fn dist3(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d: f64 = (0..3).map(|k| (b[k] - a[k]).powi(2)).sum();
    d.sqrt()
}

/// Create a new pulley joint.
pub fn new_pulley_joint_v2(anchor: [f64; 3], total_len: f64, ratio: f64) -> PulleyJointV2 {
    PulleyJointV2::new(anchor, total_len, ratio)
}

/// Constraint violation.
pub fn pj2_violation(pj: &PulleyJointV2, pa: [f64; 3], pb: [f64; 3]) -> f64 {
    pj.violation(pa, pb)
}

/// Is the constraint satisfied?
pub fn pj2_is_satisfied(pj: &PulleyJointV2, pa: [f64; 3], pb: [f64; 3], tol: f64) -> bool {
    pj.is_satisfied(pa, pb, tol)
}

/// Rope A length.
pub fn pj2_rope_length_a(pj: &PulleyJointV2, pa: [f64; 3]) -> f64 {
    pj.rope_length_a(pa)
}

/// Rope B length.
pub fn pj2_rope_length_b(pj: &PulleyJointV2, pb: [f64; 3]) -> f64 {
    pj.rope_length_b(pb)
}

/// Force ratio.
pub fn pj2_force_ratio(pj: &PulleyJointV2, fb: f64) -> f64 {
    pj.force_ratio(fb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_violation_at_rest() {
        /* anchor at origin, pa at (1,0,0), pb at (1,0,0), ratio=1, total=2 */
        let pj = new_pulley_joint_v2([0.0; 3], 2.0, 1.0);
        let v = pj2_violation(&pj, [1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(v.abs() < 1e-9 /* constraint satisfied */);
    }

    #[test]
    fn test_positive_violation() {
        let pj = new_pulley_joint_v2([0.0; 3], 1.0, 1.0);
        let v = pj2_violation(&pj, [2.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        assert!(v > 0.0 /* over-extended */);
    }

    #[test]
    fn test_is_satisfied() {
        let pj = new_pulley_joint_v2([0.0; 3], 2.0, 1.0);
        assert!(pj2_is_satisfied(&pj, [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1e-6) /* satisfied */);
    }

    #[test]
    fn test_rope_length_a() {
        let pj = new_pulley_joint_v2([0.0; 3], 5.0, 1.0);
        assert!((pj2_rope_length_a(&pj, [3.0, 4.0, 0.0]) - 5.0).abs() < 1e-9 /* 3-4-5 */);
    }

    #[test]
    fn test_rope_length_b_scaled() {
        let pj = new_pulley_joint_v2([0.0; 3], 10.0, 2.0);
        /* dist = 1.0, ratio = 2 → effective = 2.0 */
        assert!((pj2_rope_length_b(&pj, [1.0, 0.0, 0.0]) - 2.0).abs() < 1e-9 /* scaled */);
    }

    #[test]
    fn test_force_ratio() {
        let pj = new_pulley_joint_v2([0.0; 3], 5.0, 2.0);
        assert!((pj2_force_ratio(&pj, 10.0) - 5.0).abs() < 1e-9 /* mechanical advantage */);
    }

    #[test]
    fn test_ratio_clamped_to_one() {
        let pj = PulleyJointV2::new([0.0; 3], 5.0, 0.5);
        assert!(pj.ratio >= 1.0 /* ratio clamped */);
    }

    #[test]
    fn test_ratio_2_block_tackle() {
        let pj = new_pulley_joint_v2([0.0; 3], 10.0, 2.0);
        /* pa at (3,0,0) → da=3, pb at (1,0,0) → db*ratio=2, total=5 < 10 → negative violation */
        let v = pj2_violation(&pj, [3.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(v < 0.0 /* slack */);
    }

    #[test]
    fn test_total_length_stored() {
        let pj = new_pulley_joint_v2([0.0; 3], 7.5, 1.0);
        assert!((pj.total_length - 7.5).abs() < 1e-12 /* stored correctly */);
    }

    #[test]
    fn test_anchor_stored() {
        let pj = new_pulley_joint_v2([1.0, 2.0, 3.0], 5.0, 1.0);
        assert_eq!(pj.anchor, [1.0, 2.0, 3.0] /* anchor stored */);
    }
}
