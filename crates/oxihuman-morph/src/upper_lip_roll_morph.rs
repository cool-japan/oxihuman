// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct UpperLipRollMorph {
    pub eversion: f32,
    pub tubercle: f32,
    pub width: f32,
}

pub fn new_upper_lip_roll_morph() -> UpperLipRollMorph {
    UpperLipRollMorph {
        eversion: 0.0,
        tubercle: 0.0,
        width: 0.0,
    }
}

pub fn ulr_set_eversion(m: &mut UpperLipRollMorph, v: f32) {
    m.eversion = v.clamp(0.0, 1.0);
}

pub fn ulr_set_tubercle(m: &mut UpperLipRollMorph, v: f32) {
    m.tubercle = v.clamp(0.0, 1.0);
}

pub fn ulr_overall_weight(m: &UpperLipRollMorph) -> f32 {
    (m.eversion + m.tubercle + m.width) / 3.0
}

pub fn ulr_blend(a: &UpperLipRollMorph, b: &UpperLipRollMorph, t: f32) -> UpperLipRollMorph {
    let t = t.clamp(0.0, 1.0);
    UpperLipRollMorph {
        eversion: a.eversion + (b.eversion - a.eversion) * t,
        tubercle: a.tubercle + (b.tubercle - a.tubercle) * t,
        width: a.width + (b.width - a.width) * t,
    }
}

pub fn ulr_is_everted(m: &UpperLipRollMorph) -> bool {
    m.eversion > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all zero */
        let m = new_upper_lip_roll_morph();
        assert_eq!(m.eversion, 0.0);
    }

    #[test]
    fn test_set_eversion() {
        /* valid value */
        let mut m = new_upper_lip_roll_morph();
        ulr_set_eversion(&mut m, 0.7);
        assert!((m.eversion - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_eversion_clamp_high() {
        /* clamp high */
        let mut m = new_upper_lip_roll_morph();
        ulr_set_eversion(&mut m, 2.0);
        assert_eq!(m.eversion, 1.0);
    }

    #[test]
    fn test_set_tubercle_clamp_low() {
        /* clamp low */
        let mut m = new_upper_lip_roll_morph();
        ulr_set_tubercle(&mut m, -0.5);
        assert_eq!(m.tubercle, 0.0);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = UpperLipRollMorph {
            eversion: 0.3,
            tubercle: 0.6,
            width: 0.9,
        };
        assert!((ulr_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_everted_false() {
        /* default not everted */
        let m = new_upper_lip_roll_morph();
        assert!(!ulr_is_everted(&m));
    }

    #[test]
    fn test_is_everted_true() {
        /* above threshold */
        let m = UpperLipRollMorph {
            eversion: 0.9,
            tubercle: 0.0,
            width: 0.0,
        };
        assert!(ulr_is_everted(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = UpperLipRollMorph {
            eversion: 0.0,
            tubercle: 0.0,
            width: 0.0,
        };
        let b = UpperLipRollMorph {
            eversion: 1.0,
            tubercle: 1.0,
            width: 1.0,
        };
        let c = ulr_blend(&a, &b, 0.5);
        assert!((c.eversion - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let m = UpperLipRollMorph {
            eversion: 0.3,
            tubercle: 0.4,
            width: 0.5,
        };
        let m2 = m.clone();
        assert_eq!(m.eversion, m2.eversion);
    }
}
