// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! LimbScale — per-segment limb scale control.

#![allow(dead_code)]

/// Identifies a limb segment.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimbSegment {
    UpperArm,
    Forearm,
    Thigh,
    Shin,
    Hand,
    Foot,
}

/// Per-segment scale factors.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LimbScale {
    pub upper_arm: f32,
    pub forearm: f32,
    pub thigh: f32,
    pub shin: f32,
    pub hand: f32,
    pub foot: f32,
}

impl Default for LimbScale {
    fn default() -> Self {
        LimbScale { upper_arm: 1.0, forearm: 1.0, thigh: 1.0, shin: 1.0, hand: 1.0, foot: 1.0 }
    }
}

/// Create a `LimbScale` with all segments at 1.0.
#[allow(dead_code)]
pub fn new_limb_scale() -> LimbScale {
    LimbScale::default()
}

/// Set the upper-arm scale.
#[allow(dead_code)]
pub fn scale_upper_arm(ls: &mut LimbScale, s: f32) {
    ls.upper_arm = s;
}

/// Set the forearm scale.
#[allow(dead_code)]
pub fn scale_forearm(ls: &mut LimbScale, s: f32) {
    ls.forearm = s;
}

/// Set the thigh scale.
#[allow(dead_code)]
pub fn scale_thigh(ls: &mut LimbScale, s: f32) {
    ls.thigh = s;
}

/// Set the shin scale.
#[allow(dead_code)]
pub fn scale_shin(ls: &mut LimbScale, s: f32) {
    ls.shin = s;
}

/// Set the hand scale.
#[allow(dead_code)]
pub fn scale_hand(ls: &mut LimbScale, s: f32) {
    ls.hand = s;
}

/// Set the foot scale.
#[allow(dead_code)]
pub fn scale_foot(ls: &mut LimbScale, s: f32) {
    ls.foot = s;
}

/// Set all segments to the same scale factor.
#[allow(dead_code)]
pub fn limb_scale_uniform(ls: &mut LimbScale, s: f32) {
    ls.upper_arm = s;
    ls.forearm = s;
    ls.thigh = s;
    ls.shin = s;
    ls.hand = s;
    ls.foot = s;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_all_ones() {
        let ls = new_limb_scale();
        assert_eq!(ls.upper_arm, 1.0);
        assert_eq!(ls.foot, 1.0);
    }

    #[test]
    fn test_scale_upper_arm() {
        let mut ls = new_limb_scale();
        scale_upper_arm(&mut ls, 1.2);
        assert!((ls.upper_arm - 1.2).abs() < 1e-6);
    }

    #[test]
    fn test_scale_forearm() {
        let mut ls = new_limb_scale();
        scale_forearm(&mut ls, 0.9);
        assert!((ls.forearm - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_scale_thigh() {
        let mut ls = new_limb_scale();
        scale_thigh(&mut ls, 1.1);
        assert!((ls.thigh - 1.1).abs() < 1e-6);
    }

    #[test]
    fn test_scale_shin() {
        let mut ls = new_limb_scale();
        scale_shin(&mut ls, 1.3);
        assert!((ls.shin - 1.3).abs() < 1e-6);
    }

    #[test]
    fn test_scale_hand() {
        let mut ls = new_limb_scale();
        scale_hand(&mut ls, 0.8);
        assert!((ls.hand - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_scale_foot() {
        let mut ls = new_limb_scale();
        scale_foot(&mut ls, 1.05);
        assert!((ls.foot - 1.05).abs() < 1e-5);
    }

    #[test]
    fn test_limb_scale_uniform() {
        let mut ls = new_limb_scale();
        limb_scale_uniform(&mut ls, 2.0);
        assert_eq!(ls.upper_arm, 2.0);
        assert_eq!(ls.forearm, 2.0);
        assert_eq!(ls.thigh, 2.0);
        assert_eq!(ls.shin, 2.0);
        assert_eq!(ls.hand, 2.0);
        assert_eq!(ls.foot, 2.0);
    }

    #[test]
    fn test_limb_segment_eq() {
        assert_eq!(LimbSegment::UpperArm, LimbSegment::UpperArm);
        assert_ne!(LimbSegment::Hand, LimbSegment::Foot);
    }
}
