// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct AlarBaseMorph {
    pub width: f32,
    pub flare: f32,
    pub elevation: f32,
}

pub fn new_alar_base_morph() -> AlarBaseMorph {
    AlarBaseMorph {
        width: 0.0,
        flare: 0.0,
        elevation: 0.0,
    }
}

pub fn alar_set_width(m: &mut AlarBaseMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

pub fn alar_set_flare(m: &mut AlarBaseMorph, v: f32) {
    m.flare = v.clamp(0.0, 1.0);
}

pub fn alar_overall_weight(m: &AlarBaseMorph) -> f32 {
    (m.width + m.flare + m.elevation) / 3.0
}

pub fn alar_blend(a: &AlarBaseMorph, b: &AlarBaseMorph, t: f32) -> AlarBaseMorph {
    let t = t.clamp(0.0, 1.0);
    AlarBaseMorph {
        width: a.width + (b.width - a.width) * t,
        flare: a.flare + (b.flare - a.flare) * t,
        elevation: a.elevation + (b.elevation - a.elevation) * t,
    }
}

pub fn alar_is_wide(m: &AlarBaseMorph) -> bool {
    m.width > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* fields start at zero */
        let m = new_alar_base_morph();
        assert_eq!(m.width, 0.0);
        assert_eq!(m.flare, 0.0);
        assert_eq!(m.elevation, 0.0);
    }

    #[test]
    fn test_set_width_clamp_high() {
        /* clamp above 1 */
        let mut m = new_alar_base_morph();
        alar_set_width(&mut m, 5.0);
        assert_eq!(m.width, 1.0);
    }

    #[test]
    fn test_set_width_clamp_low() {
        /* clamp below 0 */
        let mut m = new_alar_base_morph();
        alar_set_width(&mut m, -1.0);
        assert_eq!(m.width, 0.0);
    }

    #[test]
    fn test_set_flare() {
        /* normal value stored */
        let mut m = new_alar_base_morph();
        alar_set_flare(&mut m, 0.4);
        assert!((m.flare - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_overall_weight() {
        /* mean of three */
        let m = AlarBaseMorph {
            width: 0.6,
            flare: 0.3,
            elevation: 0.9,
        };
        assert!((alar_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_wide_false() {
        /* below threshold */
        let m = new_alar_base_morph();
        assert!(!alar_is_wide(&m));
    }

    #[test]
    fn test_is_wide_true() {
        /* above threshold */
        let m = AlarBaseMorph {
            width: 0.8,
            flare: 0.0,
            elevation: 0.0,
        };
        assert!(alar_is_wide(&m));
    }

    #[test]
    fn test_blend_midpoint() {
        /* midpoint blend */
        let a = AlarBaseMorph {
            width: 0.0,
            flare: 0.0,
            elevation: 0.0,
        };
        let b = AlarBaseMorph {
            width: 1.0,
            flare: 1.0,
            elevation: 1.0,
        };
        let c = alar_blend(&a, &b, 0.5);
        assert!((c.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamp_t() {
        /* t>1 clamped to 1 */
        let a = AlarBaseMorph {
            width: 0.0,
            flare: 0.0,
            elevation: 0.0,
        };
        let b = AlarBaseMorph {
            width: 1.0,
            flare: 1.0,
            elevation: 1.0,
        };
        let c = alar_blend(&a, &b, 2.0);
        assert!((c.width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let m = AlarBaseMorph {
            width: 0.3,
            flare: 0.4,
            elevation: 0.5,
        };
        let m2 = m.clone();
        assert_eq!(m.width, m2.width);
    }
}
