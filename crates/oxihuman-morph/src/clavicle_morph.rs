// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct ClavicleMorph {
    pub prominence: f32,
    pub width: f32,
    pub angle_deg: f32,
}

pub fn new_clavicle_morph() -> ClavicleMorph {
    ClavicleMorph {
        prominence: 0.0,
        width: 0.5,
        angle_deg: 0.0,
    }
}

pub fn clavicle_set_prominence(m: &mut ClavicleMorph, v: f32) {
    m.prominence = v.clamp(0.0, 1.0);
}

pub fn clavicle_is_prominent(m: &ClavicleMorph) -> bool {
    m.prominence > 0.5
}

pub fn clavicle_overall_weight(m: &ClavicleMorph) -> f32 {
    (m.prominence + m.width) * 0.5
}

pub fn clavicle_blend(a: &ClavicleMorph, b: &ClavicleMorph, t: f32) -> ClavicleMorph {
    let t = t.clamp(0.0, 1.0);
    ClavicleMorph {
        prominence: a.prominence + (b.prominence - a.prominence) * t,
        width: a.width + (b.width - a.width) * t,
        angle_deg: a.angle_deg + (b.angle_deg - a.angle_deg) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* prominence starts at 0 */
        let m = new_clavicle_morph();
        assert!((m.prominence - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_prominence_clamps() {
        /* clamp above 1 */
        let mut m = new_clavicle_morph();
        clavicle_set_prominence(&mut m, 2.0);
        assert!((m.prominence - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_prominent_true() {
        /* prominence 0.8 => prominent */
        let mut m = new_clavicle_morph();
        clavicle_set_prominence(&mut m, 0.8);
        assert!(clavicle_is_prominent(&m));
    }

    #[test]
    fn test_is_prominent_false() {
        /* default not prominent */
        let m = new_clavicle_morph();
        assert!(!clavicle_is_prominent(&m));
    }

    #[test]
    fn test_overall_weight_range() {
        /* weight in [0,1] */
        let m = new_clavicle_morph();
        let w = clavicle_overall_weight(&m);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend_at_zero() {
        /* blend t=0 returns a */
        let a = new_clavicle_morph();
        let mut b = new_clavicle_morph();
        clavicle_set_prominence(&mut b, 1.0);
        let r = clavicle_blend(&a, &b, 0.0);
        assert!((r.prominence - a.prominence).abs() < 1e-6);
    }
}
