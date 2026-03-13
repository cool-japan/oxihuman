// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Moment Shadow Maps (MSM) stub.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MomentShadow {
    pub moments: [f32; 4],
}

#[allow(dead_code)]
pub fn new_moment_shadow() -> MomentShadow {
    MomentShadow { moments: [0.0; 4] }
}

#[allow(dead_code)]
pub fn msm_accumulate(shadow: &mut MomentShadow, depth: f32, weight: f32) {
    shadow.moments[0] += weight;
    shadow.moments[1] += depth * weight;
    shadow.moments[2] += depth * depth * weight;
    shadow.moments[3] += depth * depth * depth * weight;
}

#[allow(dead_code)]
pub fn msm_evaluate(shadow: &MomentShadow, depth: f32) -> f32 {
    if shadow.moments[0] == 0.0 {
        return 1.0;
    }
    let mean = shadow.moments[1] / shadow.moments[0];
    if depth <= mean { 1.0 } else { 0.0 }
}

#[allow(dead_code)]
pub fn msm_moments(shadow: &MomentShadow) -> [f32; 4] {
    shadow.moments
}

#[allow(dead_code)]
pub fn msm_reset(shadow: &mut MomentShadow) {
    shadow.moments = [0.0; 4];
}

#[allow(dead_code)]
pub fn msm_moment_count() -> usize {
    4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accumulate_updates_moments() {
        let mut s = new_moment_shadow();
        msm_accumulate(&mut s, 0.5, 1.0);
        assert!(s.moments[0] > 0.0);
        assert!(s.moments[1] > 0.0);
    }

    #[test]
    fn test_evaluate_depth_at_mean() {
        let mut s = new_moment_shadow();
        msm_accumulate(&mut s, 0.5, 1.0);
        /* depth <= mean → lit */
        let r = msm_evaluate(&s, 0.5);
        assert!((r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_depth_above_mean() {
        let mut s = new_moment_shadow();
        msm_accumulate(&mut s, 0.5, 1.0);
        /* depth > mean → shadow */
        let r = msm_evaluate(&s, 0.9);
        assert!((r - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_moments_getter() {
        let mut s = new_moment_shadow();
        msm_accumulate(&mut s, 0.5, 1.0);
        let m = msm_moments(&s);
        assert_eq!(m.len(), 4);
    }

    #[test]
    fn test_reset_zeros_all() {
        let mut s = new_moment_shadow();
        msm_accumulate(&mut s, 0.5, 1.0);
        msm_reset(&mut s);
        assert_eq!(s.moments, [0.0; 4]);
    }

    #[test]
    fn test_moment_count() {
        assert_eq!(msm_moment_count(), 4);
    }

    #[test]
    fn test_evaluate_empty_returns_one() {
        let s = new_moment_shadow();
        assert!((msm_evaluate(&s, 0.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_accumulate_multiple() {
        let mut s = new_moment_shadow();
        msm_accumulate(&mut s, 0.3, 0.5);
        msm_accumulate(&mut s, 0.7, 0.5);
        assert!((s.moments[0] - 1.0).abs() < 1e-6);
    }
}
