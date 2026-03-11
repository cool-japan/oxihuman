// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Symmetric morph: mirror deltas across a plane.

#[allow(dead_code)]
pub struct SymmetricMorph {
    pub deltas_left: Vec<[f32; 3]>,
    pub deltas_right: Vec<[f32; 3]>,
    pub axis: u8, /* 0=X, 1=Y, 2=Z */
}

#[allow(dead_code)]
pub fn new_symmetric_morph(axis: u8) -> SymmetricMorph {
    SymmetricMorph { deltas_left: Vec::new(), deltas_right: Vec::new(), axis }
}

#[allow(dead_code)]
pub fn sm_mirror_delta(axis: u8, d: [f32; 3]) -> [f32; 3] {
    let mut r = d;
    let a = (axis as usize).min(2);
    r[a] = -r[a];
    r
}

#[allow(dead_code)]
pub fn sm_add_delta(m: &mut SymmetricMorph, left: [f32; 3]) {
    let right = sm_mirror_delta(m.axis, left);
    m.deltas_left.push(left);
    m.deltas_right.push(right);
}

#[allow(dead_code)]
pub fn sm_apply(m: &SymmetricMorph, weight: f32) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    let left = m.deltas_left.iter().map(|d| [d[0] * weight, d[1] * weight, d[2] * weight]).collect();
    let right = m.deltas_right.iter().map(|d| [d[0] * weight, d[1] * weight, d[2] * weight]).collect();
    (left, right)
}

#[allow(dead_code)]
pub fn sm_delta_count(m: &SymmetricMorph) -> usize {
    m.deltas_left.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_delta() {
        let mut m = new_symmetric_morph(0);
        sm_add_delta(&mut m, [1.0, 0.0, 0.0]);
        assert_eq!(sm_delta_count(&m), 1);
    }

    #[test]
    fn test_mirror_delta_x() {
        let r = sm_mirror_delta(0, [1.0, 2.0, 3.0]);
        assert!((r[0] - (-1.0)).abs() < 1e-5);
        assert!((r[1] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_mirror_delta_y() {
        let r = sm_mirror_delta(1, [1.0, 2.0, 3.0]);
        assert!((r[1] - (-2.0)).abs() < 1e-5);
        assert!((r[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_mirror_delta_z() {
        let r = sm_mirror_delta(2, [1.0, 2.0, 3.0]);
        assert!((r[2] - (-3.0)).abs() < 1e-5);
    }

    #[test]
    fn test_apply_weight() {
        let mut m = new_symmetric_morph(0);
        sm_add_delta(&mut m, [1.0, 1.0, 1.0]);
        let (l, _r) = sm_apply(&m, 0.5);
        assert!((l[0][0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_apply_right_mirrored() {
        let mut m = new_symmetric_morph(0);
        sm_add_delta(&mut m, [1.0, 0.0, 0.0]);
        let (_l, r) = sm_apply(&m, 1.0);
        assert!((r[0][0] - (-1.0)).abs() < 1e-5);
    }

    #[test]
    fn test_delta_count() {
        let mut m = new_symmetric_morph(0);
        sm_add_delta(&mut m, [0.0, 0.0, 0.0]);
        sm_add_delta(&mut m, [1.0, 0.0, 0.0]);
        assert_eq!(sm_delta_count(&m), 2);
    }

    #[test]
    fn test_apply_zero_weight() {
        let mut m = new_symmetric_morph(0);
        sm_add_delta(&mut m, [1.0, 1.0, 1.0]);
        let (l, _) = sm_apply(&m, 0.0);
        assert!(l[0][0].abs() < 1e-5);
    }
}
