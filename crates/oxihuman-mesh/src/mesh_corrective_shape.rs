// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct CorrectiveShape {
    pub trigger_angle_rad: f32,
    pub peak_angle_rad: f32,
    pub deltas: Vec<[f32; 3]>,
}

pub fn new_corrective_shape(trigger: f32, peak: f32, deltas: Vec<[f32; 3]>) -> CorrectiveShape {
    CorrectiveShape {
        trigger_angle_rad: trigger,
        peak_angle_rad: peak,
        deltas,
    }
}

/// Ramp: 0 below trigger, 1 at peak, 0 beyond peak (triangle ramp).
pub fn corrective_weight(s: &CorrectiveShape, angle: f32) -> f32 {
    let t = s.trigger_angle_rad;
    let p = s.peak_angle_rad;
    if (p - t).abs() < 1e-8 {
        if (angle - t).abs() < 1e-6 {
            return 1.0;
        }
        return 0.0;
    }
    if angle < t || angle > 2.0 * p - t {
        return 0.0;
    }
    let half = p - t;
    if angle <= p {
        (angle - t) / half
    } else {
        (2.0 * p - t - angle) / half
    }
}

#[allow(clippy::needless_range_loop)]
pub fn corrective_apply(s: &CorrectiveShape, base: &[[f32; 3]], angle: f32) -> Vec<[f32; 3]> {
    let w = corrective_weight(s, angle);
    let n = base.len();
    let mut result = base.to_vec();
    for i in 0..n.min(s.deltas.len()) {
        result[i][0] += w * s.deltas[i][0];
        result[i][1] += w * s.deltas[i][1];
        result[i][2] += w * s.deltas[i][2];
    }
    result
}

pub fn corrective_is_active(s: &CorrectiveShape, angle: f32) -> bool {
    corrective_weight(s, angle) > 0.0
}

pub fn corrective_peak_weight(s: &CorrectiveShape) -> f32 {
    corrective_weight(s, s.peak_angle_rad)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    #[test]
    fn test_new_corrective_shape() {
        /* basic construction */
        let s = new_corrective_shape(0.0, FRAC_PI_2, vec![]);
        assert!((s.trigger_angle_rad).abs() < 1e-6);
    }

    #[test]
    fn test_weight_at_peak() {
        /* weight is 1 at peak */
        let s = new_corrective_shape(0.0, 1.0, vec![]);
        assert!((corrective_weight(&s, 1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_weight_below_trigger() {
        /* weight is 0 below trigger */
        let s = new_corrective_shape(0.5, 1.0, vec![]);
        assert!((corrective_weight(&s, 0.0)).abs() < 1e-6);
    }

    #[test]
    fn test_is_active() {
        /* active between trigger and 2*peak-trigger */
        let s = new_corrective_shape(0.0, 1.0, vec![]);
        assert!(corrective_is_active(&s, 0.5));
        assert!(!corrective_is_active(&s, 3.0));
    }

    #[test]
    fn test_peak_weight() {
        /* peak weight should be 1 */
        let s = new_corrective_shape(0.0, 1.0, vec![]);
        assert!((corrective_peak_weight(&s) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_at_trigger() {
        /* at trigger, no displacement */
        let s = new_corrective_shape(0.5, 1.0, vec![[1.0, 0.0, 0.0]]);
        let base = vec![[0.0, 0.0, 0.0_f32]];
        let result = corrective_apply(&s, &base, 0.5);
        assert!(result[0][0].abs() < 1e-6);
    }
}
