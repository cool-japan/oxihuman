// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct NasolabialMorph {
    pub depth: f32,
    pub length: f32,
    pub angle_deg: f32,
}

pub fn new_nasolabial_morph() -> NasolabialMorph {
    NasolabialMorph {
        depth: 0.0,
        length: 0.5,
        angle_deg: 0.0,
    }
}

pub fn nasolabial_set_depth(m: &mut NasolabialMorph, v: f32) {
    m.depth = v.clamp(0.0, 1.0);
}

pub fn nasolabial_is_deep(m: &NasolabialMorph) -> bool {
    m.depth > 0.6
}

pub fn nasolabial_overall_weight(m: &NasolabialMorph) -> f32 {
    (m.depth + m.length) * 0.5
}

pub fn nasolabial_blend(a: &NasolabialMorph, b: &NasolabialMorph, t: f32) -> NasolabialMorph {
    let t = t.clamp(0.0, 1.0);
    NasolabialMorph {
        depth: a.depth + (b.depth - a.depth) * t,
        length: a.length + (b.length - a.length) * t,
        angle_deg: a.angle_deg + (b.angle_deg - a.angle_deg) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* depth starts at 0 */
        let m = new_nasolabial_morph();
        assert!((m.depth - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamps() {
        /* clamp above 1 */
        let mut m = new_nasolabial_morph();
        nasolabial_set_depth(&mut m, 2.0);
        assert!((m.depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_deep_true() {
        /* depth 0.7 => deep */
        let mut m = new_nasolabial_morph();
        nasolabial_set_depth(&mut m, 0.7);
        assert!(nasolabial_is_deep(&m));
    }

    #[test]
    fn test_is_deep_false() {
        /* default not deep */
        let m = new_nasolabial_morph();
        assert!(!nasolabial_is_deep(&m));
    }

    #[test]
    fn test_overall_weight_range() {
        /* weight in [0,1] */
        let m = new_nasolabial_morph();
        let w = nasolabial_overall_weight(&m);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend_at_one() {
        /* blend t=1 returns b */
        let a = new_nasolabial_morph();
        let mut b = new_nasolabial_morph();
        nasolabial_set_depth(&mut b, 1.0);
        let r = nasolabial_blend(&a, &b, 1.0);
        assert!((r.depth - 1.0).abs() < 1e-6);
    }
}
