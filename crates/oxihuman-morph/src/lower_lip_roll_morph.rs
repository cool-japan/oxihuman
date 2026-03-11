// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct LowerLipRollMorph {
    pub eversion: f32,
    pub fullness: f32,
    pub sulcus: f32,
}

pub fn new_lower_lip_roll_morph() -> LowerLipRollMorph {
    LowerLipRollMorph {
        eversion: 0.0,
        fullness: 0.0,
        sulcus: 0.0,
    }
}

pub fn llr_set_eversion(m: &mut LowerLipRollMorph, v: f32) {
    m.eversion = v.clamp(0.0, 1.0);
}

pub fn llr_set_fullness(m: &mut LowerLipRollMorph, v: f32) {
    m.fullness = v.clamp(0.0, 1.0);
}

pub fn llr_overall_weight(m: &LowerLipRollMorph) -> f32 {
    (m.eversion + m.fullness + m.sulcus) / 3.0
}

pub fn llr_blend(a: &LowerLipRollMorph, b: &LowerLipRollMorph, t: f32) -> LowerLipRollMorph {
    let t = t.clamp(0.0, 1.0);
    LowerLipRollMorph {
        eversion: a.eversion + (b.eversion - a.eversion) * t,
        fullness: a.fullness + (b.fullness - a.fullness) * t,
        sulcus: a.sulcus + (b.sulcus - a.sulcus) * t,
    }
}

pub fn llr_is_everted(m: &LowerLipRollMorph) -> bool {
    m.eversion > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all zero */
        let m = new_lower_lip_roll_morph();
        assert_eq!(m.eversion, 0.0);
    }

    #[test]
    fn test_set_eversion() {
        /* valid value */
        let mut m = new_lower_lip_roll_morph();
        llr_set_eversion(&mut m, 0.6);
        assert!((m.eversion - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_eversion_clamp_high() {
        /* clamp high */
        let mut m = new_lower_lip_roll_morph();
        llr_set_eversion(&mut m, 3.0);
        assert_eq!(m.eversion, 1.0);
    }

    #[test]
    fn test_set_fullness_clamp_low() {
        /* clamp low */
        let mut m = new_lower_lip_roll_morph();
        llr_set_fullness(&mut m, -1.0);
        assert_eq!(m.fullness, 0.0);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = LowerLipRollMorph {
            eversion: 0.3,
            fullness: 0.6,
            sulcus: 0.9,
        };
        assert!((llr_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_everted_false() {
        /* default not everted */
        let m = new_lower_lip_roll_morph();
        assert!(!llr_is_everted(&m));
    }

    #[test]
    fn test_is_everted_true() {
        /* above threshold */
        let m = LowerLipRollMorph {
            eversion: 0.9,
            fullness: 0.0,
            sulcus: 0.0,
        };
        assert!(llr_is_everted(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = LowerLipRollMorph {
            eversion: 0.0,
            fullness: 0.0,
            sulcus: 0.0,
        };
        let b = LowerLipRollMorph {
            eversion: 1.0,
            fullness: 1.0,
            sulcus: 1.0,
        };
        let c = llr_blend(&a, &b, 0.5);
        assert!((c.eversion - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let m = LowerLipRollMorph {
            eversion: 0.3,
            fullness: 0.4,
            sulcus: 0.5,
        };
        let m2 = m.clone();
        assert_eq!(m.eversion, m2.eversion);
    }
}
