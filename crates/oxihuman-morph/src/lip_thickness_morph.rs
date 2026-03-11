// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct LipThicknessMorph {
    pub upper_thickness: f32,
    pub lower_thickness: f32,
    pub vermillion_height: f32,
}

pub fn new_lip_thickness_morph() -> LipThicknessMorph {
    LipThicknessMorph {
        upper_thickness: 0.0,
        lower_thickness: 0.0,
        vermillion_height: 0.0,
    }
}

pub fn lip_thickness_set_upper(m: &mut LipThicknessMorph, v: f32) {
    m.upper_thickness = v.clamp(0.0, 1.0);
}

pub fn lip_thickness_is_full(m: &LipThicknessMorph) -> bool {
    m.upper_thickness > 0.5 && m.lower_thickness > 0.5
}

pub fn lip_thickness_overall_weight(m: &LipThicknessMorph) -> f32 {
    (m.upper_thickness.abs() + m.lower_thickness.abs() + m.vermillion_height.abs()) / 3.0
}

pub fn lip_thickness_blend(
    a: &LipThicknessMorph,
    b: &LipThicknessMorph,
    t: f32,
) -> LipThicknessMorph {
    let t = t.clamp(0.0, 1.0);
    LipThicknessMorph {
        upper_thickness: a.upper_thickness + (b.upper_thickness - a.upper_thickness) * t,
        lower_thickness: a.lower_thickness + (b.lower_thickness - a.lower_thickness) * t,
        vermillion_height: a.vermillion_height + (b.vermillion_height - a.vermillion_height) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_lip_thickness_morph() {
        /* defaults to zero */
        let m = new_lip_thickness_morph();
        assert_eq!(m.upper_thickness, 0.0);
    }

    #[test]
    fn test_lip_thickness_set_upper() {
        /* upper is set correctly */
        let mut m = new_lip_thickness_morph();
        lip_thickness_set_upper(&mut m, 0.6);
        assert!((m.upper_thickness - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_lip_thickness_is_full_false() {
        /* not full when both are 0 */
        let m = new_lip_thickness_morph();
        assert!(!lip_thickness_is_full(&m));
    }

    #[test]
    fn test_lip_thickness_is_full_true() {
        /* full when both > 0.5 */
        let m = LipThicknessMorph {
            upper_thickness: 0.7,
            lower_thickness: 0.8,
            vermillion_height: 0.0,
        };
        assert!(lip_thickness_is_full(&m));
    }

    #[test]
    fn test_lip_thickness_blend() {
        /* blend at t=1 returns b values */
        let a = new_lip_thickness_morph();
        let b = LipThicknessMorph {
            upper_thickness: 1.0,
            lower_thickness: 1.0,
            vermillion_height: 1.0,
        };
        let r = lip_thickness_blend(&a, &b, 1.0);
        assert!((r.upper_thickness - 1.0).abs() < 1e-6);
    }
}
