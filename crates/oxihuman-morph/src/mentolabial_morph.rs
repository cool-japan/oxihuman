// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct MentolabialMorph {
    pub depth: f32,
    pub width: f32,
    pub position: f32,
}

pub fn new_mentolabial_morph() -> MentolabialMorph {
    MentolabialMorph {
        depth: 0.0,
        width: 0.5,
        position: 0.5,
    }
}

pub fn mentolabial_set_depth(m: &mut MentolabialMorph, v: f32) {
    m.depth = v.clamp(0.0, 1.0);
}

pub fn mentolabial_is_deep(m: &MentolabialMorph) -> bool {
    m.depth > 0.5
}

pub fn mentolabial_overall_weight(m: &MentolabialMorph) -> f32 {
    (m.depth + m.width + m.position) / 3.0
}

pub fn mentolabial_blend(a: &MentolabialMorph, b: &MentolabialMorph, t: f32) -> MentolabialMorph {
    let t = t.clamp(0.0, 1.0);
    MentolabialMorph {
        depth: a.depth + (b.depth - a.depth) * t,
        width: a.width + (b.width - a.width) * t,
        position: a.position + (b.position - a.position) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* depth starts at 0 */
        let m = new_mentolabial_morph();
        assert!((m.depth - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamps() {
        /* clamp above 1 */
        let mut m = new_mentolabial_morph();
        mentolabial_set_depth(&mut m, 2.0);
        assert!((m.depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_deep_true() {
        /* depth 0.7 => deep */
        let mut m = new_mentolabial_morph();
        mentolabial_set_depth(&mut m, 0.7);
        assert!(mentolabial_is_deep(&m));
    }

    #[test]
    fn test_is_deep_false() {
        /* default not deep */
        let m = new_mentolabial_morph();
        assert!(!mentolabial_is_deep(&m));
    }

    #[test]
    fn test_overall_weight_range() {
        /* weight in [0,1] */
        let m = new_mentolabial_morph();
        let w = mentolabial_overall_weight(&m);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend_at_one() {
        /* blend t=1 returns b */
        let a = new_mentolabial_morph();
        let mut b = new_mentolabial_morph();
        mentolabial_set_depth(&mut b, 1.0);
        let r = mentolabial_blend(&a, &b, 1.0);
        assert!((r.depth - 1.0).abs() < 1e-6);
    }
}
