// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct SupraorbitalMorph {
    pub ridge_height: f32,
    pub brow_slope: f32,
    pub overhang: f32,
}

pub fn new_supraorbital_morph() -> SupraorbitalMorph {
    SupraorbitalMorph {
        ridge_height: 0.0,
        brow_slope: 0.0,
        overhang: 0.0,
    }
}

pub fn supraorbital_set_ridge(m: &mut SupraorbitalMorph, v: f32) {
    m.ridge_height = v.clamp(0.0, 1.0);
}

pub fn supraorbital_set_slope(m: &mut SupraorbitalMorph, v: f32) {
    m.brow_slope = v.clamp(0.0, 1.0);
}

pub fn supraorbital_overall_weight(m: &SupraorbitalMorph) -> f32 {
    (m.ridge_height + m.brow_slope + m.overhang) / 3.0
}

pub fn supraorbital_is_heavy(m: &SupraorbitalMorph) -> bool {
    m.ridge_height > 0.5 || m.overhang > 0.5
}

pub fn supraorbital_blend(
    a: &SupraorbitalMorph,
    b: &SupraorbitalMorph,
    t: f32,
) -> SupraorbitalMorph {
    let t = t.clamp(0.0, 1.0);
    SupraorbitalMorph {
        ridge_height: a.ridge_height + (b.ridge_height - a.ridge_height) * t,
        brow_slope: a.brow_slope + (b.brow_slope - a.brow_slope) * t,
        overhang: a.overhang + (b.overhang - a.overhang) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all start at 0 */
        let m = new_supraorbital_morph();
        assert!((m.ridge_height - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_ridge_clamps() {
        /* clamp above 1 */
        let mut m = new_supraorbital_morph();
        supraorbital_set_ridge(&mut m, 2.0);
        assert!((m.ridge_height - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_slope() {
        /* set slope */
        let mut m = new_supraorbital_morph();
        supraorbital_set_slope(&mut m, 0.6);
        assert!((m.brow_slope - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_overall_weight_range() {
        /* weight in [0,1] */
        let m = new_supraorbital_morph();
        let w = supraorbital_overall_weight(&m);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_is_heavy_false() {
        /* default not heavy */
        let m = new_supraorbital_morph();
        assert!(!supraorbital_is_heavy(&m));
    }

    #[test]
    fn test_blend_at_one() {
        /* blend t=1 returns b */
        let a = new_supraorbital_morph();
        let mut b = new_supraorbital_morph();
        supraorbital_set_ridge(&mut b, 1.0);
        let r = supraorbital_blend(&a, &b, 1.0);
        assert!((r.ridge_height - 1.0).abs() < 1e-6);
    }
}
