// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Styloid process length morph.
#[derive(Debug, Clone)]
pub struct StyloidMorph {
    /// Process length (0.0 = absent, 1.0 = elongated).
    pub length: f32,
    /// Angulation deviation from neutral (-1.0 … 1.0).
    pub angle_offset: f32,
    /// Tip sharpness (0.0 = blunt, 1.0 = sharp).
    pub tip_sharpness: f32,
}

pub fn new_styloid_morph() -> StyloidMorph {
    StyloidMorph {
        length: 0.0,
        angle_offset: 0.0,
        tip_sharpness: 0.0,
    }
}

pub fn sty_set_length(m: &mut StyloidMorph, v: f32) {
    m.length = v.clamp(0.0, 1.0);
}

pub fn sty_set_angle_offset(m: &mut StyloidMorph, v: f32) {
    m.angle_offset = v.clamp(-1.0, 1.0);
}

pub fn sty_set_tip_sharpness(m: &mut StyloidMorph, v: f32) {
    m.tip_sharpness = v.clamp(0.0, 1.0);
}

pub fn sty_overall_weight(m: &StyloidMorph) -> f32 {
    (m.length + m.angle_offset.abs() + m.tip_sharpness) / 3.0
}

pub fn sty_blend(a: &StyloidMorph, b: &StyloidMorph, t: f32) -> StyloidMorph {
    let t = t.clamp(0.0, 1.0);
    StyloidMorph {
        length: a.length + (b.length - a.length) * t,
        angle_offset: a.angle_offset + (b.angle_offset - a.angle_offset) * t,
        tip_sharpness: a.tip_sharpness + (b.tip_sharpness - a.tip_sharpness) * t,
    }
}

pub fn sty_is_neutral(m: &StyloidMorph) -> bool {
    m.length < 1e-5 && m.angle_offset.abs() < 1e-5 && m.tip_sharpness < 1e-5
}

pub fn sty_to_json(m: &StyloidMorph) -> String {
    format!(
        r#"{{"length":{:.4},"angle_offset":{:.4},"tip_sharpness":{:.4}}}"#,
        m.length, m.angle_offset, m.tip_sharpness
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all zero */
        let m = new_styloid_morph();
        assert_eq!(m.length, 0.0);
        assert_eq!(m.angle_offset, 0.0);
        assert_eq!(m.tip_sharpness, 0.0);
    }

    #[test]
    fn test_set_length() {
        /* valid length stored */
        let mut m = new_styloid_morph();
        sty_set_length(&mut m, 0.5);
        assert!((m.length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_length_clamp_high() {
        /* clamp above 1 */
        let mut m = new_styloid_morph();
        sty_set_length(&mut m, 2.0);
        assert_eq!(m.length, 1.0);
    }

    #[test]
    fn test_angle_offset_clamp() {
        /* clamp below -1 */
        let mut m = new_styloid_morph();
        sty_set_angle_offset(&mut m, -5.0);
        assert_eq!(m.angle_offset, -1.0);
    }

    #[test]
    fn test_is_neutral_true() {
        /* default neutral */
        assert!(sty_is_neutral(&new_styloid_morph()));
    }

    #[test]
    fn test_is_neutral_false() {
        /* after set length not neutral */
        let mut m = new_styloid_morph();
        sty_set_length(&mut m, 0.2);
        assert!(!sty_is_neutral(&m));
    }

    #[test]
    fn test_overall_weight() {
        /* formula check */
        let m = StyloidMorph {
            length: 0.9,
            angle_offset: 0.0,
            tip_sharpness: 0.9,
        };
        assert!((sty_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_blend() {
        /* blend at 1.0 gives b */
        let a = new_styloid_morph();
        let b = StyloidMorph {
            length: 1.0,
            angle_offset: 0.5,
            tip_sharpness: 0.8,
        };
        let c = sty_blend(&a, &b, 1.0);
        assert!((c.length - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        /* JSON has length key */
        assert!(sty_to_json(&new_styloid_morph()).contains("length"));
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let m = StyloidMorph {
            length: 0.3,
            angle_offset: 0.2,
            tip_sharpness: 0.1,
        };
        let m2 = m.clone();
        assert_eq!(m.length, m2.length);
    }
}
