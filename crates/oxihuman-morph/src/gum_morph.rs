// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct GumMorph {
    pub exposure: f32,
    pub recession: f32,
    pub color_saturation: f32,
}

pub fn new_gum_morph() -> GumMorph {
    GumMorph {
        exposure: 0.0,
        recession: 0.0,
        color_saturation: 0.5,
    }
}

pub fn gum_set_exposure(m: &mut GumMorph, v: f32) {
    m.exposure = v.clamp(0.0, 1.0);
}

pub fn gum_is_gummy_smile(m: &GumMorph) -> bool {
    m.exposure > 0.5
}

pub fn gum_overall_weight(m: &GumMorph) -> f32 {
    (m.exposure.abs() + m.recession.abs() + m.color_saturation.abs()) / 3.0
}

pub fn gum_blend(a: &GumMorph, b: &GumMorph, t: f32) -> GumMorph {
    let t = t.clamp(0.0, 1.0);
    GumMorph {
        exposure: a.exposure + (b.exposure - a.exposure) * t,
        recession: a.recession + (b.recession - a.recession) * t,
        color_saturation: a.color_saturation + (b.color_saturation - a.color_saturation) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_gum_morph() {
        /* exposure defaults to 0 */
        let m = new_gum_morph();
        assert_eq!(m.exposure, 0.0);
    }

    #[test]
    fn test_gum_set_exposure() {
        /* exposure is set and clamped */
        let mut m = new_gum_morph();
        gum_set_exposure(&mut m, 0.8);
        assert!((m.exposure - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_gum_is_gummy_smile_true() {
        /* exposure above 0.5 is gummy */
        let mut m = new_gum_morph();
        m.exposure = 0.7;
        assert!(gum_is_gummy_smile(&m));
    }

    #[test]
    fn test_gum_is_gummy_smile_false() {
        /* exposure at 0.5 or below is not gummy */
        let m = new_gum_morph();
        assert!(!gum_is_gummy_smile(&m));
    }

    #[test]
    fn test_gum_blend() {
        /* blend at t=0.5 is midpoint */
        let a = new_gum_morph();
        let b = GumMorph {
            exposure: 1.0,
            recession: 1.0,
            color_saturation: 1.0,
        };
        let r = gum_blend(&a, &b, 0.5);
        assert!((r.exposure - 0.5).abs() < 1e-6);
    }
}
