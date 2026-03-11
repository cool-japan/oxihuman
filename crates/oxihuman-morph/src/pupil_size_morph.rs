// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// 0 = max constricted, 1 = max dilated.
pub struct PupilSizeMorph {
    pub dilation: f32,
}

pub fn new_pupil_size_morph() -> PupilSizeMorph {
    PupilSizeMorph { dilation: 0.3 }
}

pub fn pupil_set_dilation(m: &mut PupilSizeMorph, v: f32) {
    m.dilation = v.clamp(0.0, 1.0);
}

pub fn pupil_is_dilated(m: &PupilSizeMorph) -> bool {
    m.dilation > 0.6
}

pub fn pupil_overall_weight(m: &PupilSizeMorph) -> f32 {
    m.dilation.abs()
}

pub fn pupil_blend(a: &PupilSizeMorph, b: &PupilSizeMorph, t: f32) -> PupilSizeMorph {
    let t = t.clamp(0.0, 1.0);
    PupilSizeMorph {
        dilation: a.dilation + (b.dilation - a.dilation) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pupil_size_morph() {
        /* default dilation is 0.3 */
        let m = new_pupil_size_morph();
        assert!((m.dilation - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_pupil_set_dilation() {
        /* dilation is set */
        let mut m = new_pupil_size_morph();
        pupil_set_dilation(&mut m, 0.8);
        assert!((m.dilation - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_pupil_is_dilated_false() {
        /* default is not dilated */
        let m = new_pupil_size_morph();
        assert!(!pupil_is_dilated(&m));
    }

    #[test]
    fn test_pupil_is_dilated_true() {
        /* above 0.6 is dilated */
        let mut m = new_pupil_size_morph();
        m.dilation = 0.7;
        assert!(pupil_is_dilated(&m));
    }

    #[test]
    fn test_pupil_blend() {
        /* blend at 0.5 */
        let a = PupilSizeMorph { dilation: 0.0 };
        let b = PupilSizeMorph { dilation: 1.0 };
        let r = pupil_blend(&a, &b, 0.5);
        assert!((r.dilation - 0.5).abs() < 1e-6);
    }
}
