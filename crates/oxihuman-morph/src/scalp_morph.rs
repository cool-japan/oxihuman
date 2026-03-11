// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct ScalpMorph {
    pub hairline_height: f32,
    pub crown_width: f32,
    pub recession: f32,
}

pub fn new_scalp_morph() -> ScalpMorph {
    ScalpMorph {
        hairline_height: 0.0,
        crown_width: 0.0,
        recession: 0.0,
    }
}

pub fn scalp_set_hairline(m: &mut ScalpMorph, v: f32) {
    m.hairline_height = v.clamp(0.0, 1.0);
}

pub fn scalp_set_crown_width(m: &mut ScalpMorph, v: f32) {
    m.crown_width = v.clamp(0.0, 1.0);
}

pub fn scalp_set_recession(m: &mut ScalpMorph, v: f32) {
    m.recession = v.clamp(0.0, 1.0);
}

pub fn scalp_overall_weight(m: &ScalpMorph) -> f32 {
    (m.hairline_height + m.crown_width + m.recession) / 3.0
}

pub fn scalp_blend(a: &ScalpMorph, b: &ScalpMorph, t: f32) -> ScalpMorph {
    let t = t.clamp(0.0, 1.0);
    ScalpMorph {
        hairline_height: a.hairline_height + (b.hairline_height - a.hairline_height) * t,
        crown_width: a.crown_width + (b.crown_width - a.crown_width) * t,
        recession: a.recession + (b.recession - a.recession) * t,
    }
}

pub fn scalp_is_receding(m: &ScalpMorph) -> bool {
    m.recession > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all zero */
        let m = new_scalp_morph();
        assert_eq!(m.hairline_height, 0.0);
        assert_eq!(m.crown_width, 0.0);
        assert_eq!(m.recession, 0.0);
    }

    #[test]
    fn test_set_hairline() {
        /* valid value */
        let mut m = new_scalp_morph();
        scalp_set_hairline(&mut m, 0.6);
        assert!((m.hairline_height - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_hairline_clamp_high() {
        /* clamp above 1 */
        let mut m = new_scalp_morph();
        scalp_set_hairline(&mut m, 2.0);
        assert_eq!(m.hairline_height, 1.0);
    }

    #[test]
    fn test_set_crown_width_clamp_low() {
        /* clamp below 0 */
        let mut m = new_scalp_morph();
        scalp_set_crown_width(&mut m, -0.5);
        assert_eq!(m.crown_width, 0.0);
    }

    #[test]
    fn test_set_recession() {
        /* valid recession */
        let mut m = new_scalp_morph();
        scalp_set_recession(&mut m, 0.75);
        assert!((m.recession - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_overall_weight() {
        /* average of three */
        let m = ScalpMorph {
            hairline_height: 0.3,
            crown_width: 0.6,
            recession: 0.9,
        };
        assert!((scalp_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_receding_false() {
        /* default not receding */
        let m = new_scalp_morph();
        assert!(!scalp_is_receding(&m));
    }

    #[test]
    fn test_is_receding_true() {
        /* recession above 0.5 */
        let m = ScalpMorph {
            hairline_height: 0.0,
            crown_width: 0.0,
            recession: 0.8,
        };
        assert!(scalp_is_receding(&m));
    }

    #[test]
    fn test_blend_midpoint() {
        /* blend at t=0.5 */
        let a = ScalpMorph {
            hairline_height: 0.0,
            crown_width: 0.0,
            recession: 0.0,
        };
        let b = ScalpMorph {
            hairline_height: 1.0,
            crown_width: 1.0,
            recession: 1.0,
        };
        let c = scalp_blend(&a, &b, 0.5);
        assert!((c.hairline_height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let m = ScalpMorph {
            hairline_height: 0.2,
            crown_width: 0.4,
            recession: 0.6,
        };
        let m2 = m.clone();
        assert_eq!(m.recession, m2.recession);
    }
}
