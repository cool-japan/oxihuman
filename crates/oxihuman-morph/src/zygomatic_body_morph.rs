// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Zygomatic body (malar eminence) shape morph.
#[derive(Debug, Clone)]
pub struct ZygomaticBodyMorph {
    /// Anterior projection of the malar body (0.0 = flat, 1.0 = prominent).
    pub projection: f32,
    /// Vertical height of the body (0.0 = low, 1.0 = high).
    pub height: f32,
    /// Horizontal breadth (0.0 = narrow, 1.0 = wide).
    pub breadth: f32,
}

pub fn new_zygomatic_body_morph() -> ZygomaticBodyMorph {
    ZygomaticBodyMorph {
        projection: 0.0,
        height: 0.0,
        breadth: 0.0,
    }
}

pub fn zyg_set_projection(m: &mut ZygomaticBodyMorph, v: f32) {
    m.projection = v.clamp(0.0, 1.0);
}

pub fn zyg_set_height(m: &mut ZygomaticBodyMorph, v: f32) {
    m.height = v.clamp(0.0, 1.0);
}

pub fn zyg_set_breadth(m: &mut ZygomaticBodyMorph, v: f32) {
    m.breadth = v.clamp(0.0, 1.0);
}

pub fn zyg_overall_weight(m: &ZygomaticBodyMorph) -> f32 {
    (m.projection + m.height + m.breadth) / 3.0
}

pub fn zyg_blend(a: &ZygomaticBodyMorph, b: &ZygomaticBodyMorph, t: f32) -> ZygomaticBodyMorph {
    let t = t.clamp(0.0, 1.0);
    ZygomaticBodyMorph {
        projection: a.projection + (b.projection - a.projection) * t,
        height: a.height + (b.height - a.height) * t,
        breadth: a.breadth + (b.breadth - a.breadth) * t,
    }
}

pub fn zyg_is_neutral(m: &ZygomaticBodyMorph) -> bool {
    m.projection < 1e-5 && m.height < 1e-5 && m.breadth < 1e-5
}

pub fn zyg_to_json(m: &ZygomaticBodyMorph) -> String {
    format!(
        r#"{{"projection":{:.4},"height":{:.4},"breadth":{:.4}}}"#,
        m.projection, m.height, m.breadth
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all zero */
        let m = new_zygomatic_body_morph();
        assert_eq!(m.projection, 0.0);
        assert_eq!(m.height, 0.0);
        assert_eq!(m.breadth, 0.0);
    }

    #[test]
    fn test_set_projection() {
        /* valid value stored */
        let mut m = new_zygomatic_body_morph();
        zyg_set_projection(&mut m, 0.8);
        assert!((m.projection - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_projection_clamp_high() {
        /* clamp above 1 */
        let mut m = new_zygomatic_body_morph();
        zyg_set_projection(&mut m, 3.0);
        assert_eq!(m.projection, 1.0);
    }

    #[test]
    fn test_breadth_clamp_low() {
        /* clamp below 0 */
        let mut m = new_zygomatic_body_morph();
        zyg_set_breadth(&mut m, -0.5);
        assert_eq!(m.breadth, 0.0);
    }

    #[test]
    fn test_is_neutral_true() {
        /* default is neutral */
        assert!(zyg_is_neutral(&new_zygomatic_body_morph()));
    }

    #[test]
    fn test_is_neutral_false() {
        /* after setting projection */
        let mut m = new_zygomatic_body_morph();
        zyg_set_projection(&mut m, 0.3);
        assert!(!zyg_is_neutral(&m));
    }

    #[test]
    fn test_overall_weight() {
        /* equal weights average */
        let m = ZygomaticBodyMorph {
            projection: 0.9,
            height: 0.9,
            breadth: 0.9,
        };
        assert!((zyg_overall_weight(&m) - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_blend_at_zero() {
        /* blend 0 gives a */
        let a = ZygomaticBodyMorph {
            projection: 0.4,
            height: 0.5,
            breadth: 0.6,
        };
        let b = new_zygomatic_body_morph();
        let c = zyg_blend(&a, &b, 0.0);
        assert!((c.projection - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        /* JSON has projection */
        assert!(zyg_to_json(&new_zygomatic_body_morph()).contains("projection"));
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let m = ZygomaticBodyMorph {
            projection: 0.2,
            height: 0.3,
            breadth: 0.4,
        };
        let m2 = m.clone();
        assert_eq!(m.height, m2.height);
    }
}
