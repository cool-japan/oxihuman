// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct VermillionWidthMorph {
    pub upper_width: f32,
    pub lower_width: f32,
    pub corner_width: f32,
}

pub fn new_vermillion_width_morph() -> VermillionWidthMorph {
    VermillionWidthMorph {
        upper_width: 0.0,
        lower_width: 0.0,
        corner_width: 0.0,
    }
}

pub fn vw_set_upper(m: &mut VermillionWidthMorph, v: f32) {
    m.upper_width = v.clamp(0.0, 1.0);
}

pub fn vw_set_lower(m: &mut VermillionWidthMorph, v: f32) {
    m.lower_width = v.clamp(0.0, 1.0);
}

pub fn vw_overall_weight(m: &VermillionWidthMorph) -> f32 {
    (m.upper_width + m.lower_width + m.corner_width) / 3.0
}

pub fn vw_blend(
    a: &VermillionWidthMorph,
    b: &VermillionWidthMorph,
    t: f32,
) -> VermillionWidthMorph {
    let t = t.clamp(0.0, 1.0);
    VermillionWidthMorph {
        upper_width: a.upper_width + (b.upper_width - a.upper_width) * t,
        lower_width: a.lower_width + (b.lower_width - a.lower_width) * t,
        corner_width: a.corner_width + (b.corner_width - a.corner_width) * t,
    }
}

pub fn vw_is_wide(m: &VermillionWidthMorph) -> bool {
    m.upper_width > 0.5 && m.lower_width > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all zero */
        let m = new_vermillion_width_morph();
        assert_eq!(m.upper_width, 0.0);
    }

    #[test]
    fn test_set_upper() {
        /* valid value */
        let mut m = new_vermillion_width_morph();
        vw_set_upper(&mut m, 0.6);
        assert!((m.upper_width - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_upper_clamp_high() {
        /* clamp above 1 */
        let mut m = new_vermillion_width_morph();
        vw_set_upper(&mut m, 2.0);
        assert_eq!(m.upper_width, 1.0);
    }

    #[test]
    fn test_set_lower_clamp_low() {
        /* clamp below 0 */
        let mut m = new_vermillion_width_morph();
        vw_set_lower(&mut m, -0.5);
        assert_eq!(m.lower_width, 0.0);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = VermillionWidthMorph {
            upper_width: 0.3,
            lower_width: 0.6,
            corner_width: 0.9,
        };
        assert!((vw_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_wide_false_both_zero() {
        /* both zero => not wide */
        let m = new_vermillion_width_morph();
        assert!(!vw_is_wide(&m));
    }

    #[test]
    fn test_is_wide_false_one_below() {
        /* only upper wide */
        let m = VermillionWidthMorph {
            upper_width: 0.8,
            lower_width: 0.2,
            corner_width: 0.0,
        };
        assert!(!vw_is_wide(&m));
    }

    #[test]
    fn test_is_wide_true() {
        /* both above 0.5 */
        let m = VermillionWidthMorph {
            upper_width: 0.8,
            lower_width: 0.7,
            corner_width: 0.0,
        };
        assert!(vw_is_wide(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = VermillionWidthMorph {
            upper_width: 0.0,
            lower_width: 0.0,
            corner_width: 0.0,
        };
        let b = VermillionWidthMorph {
            upper_width: 1.0,
            lower_width: 1.0,
            corner_width: 1.0,
        };
        let c = vw_blend(&a, &b, 0.5);
        assert!((c.upper_width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let m = VermillionWidthMorph {
            upper_width: 0.3,
            lower_width: 0.4,
            corner_width: 0.5,
        };
        let m2 = m.clone();
        assert_eq!(m.upper_width, m2.upper_width);
    }
}
