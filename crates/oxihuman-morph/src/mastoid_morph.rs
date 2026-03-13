// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct MastoidMorph {
    pub size: f32,
    pub protrusion: f32,
}

pub fn new_mastoid_morph() -> MastoidMorph {
    MastoidMorph {
        size: 0.5,
        protrusion: 0.0,
    }
}

pub fn mastoid_set_size(m: &mut MastoidMorph, v: f32) {
    m.size = v.clamp(0.0, 1.0);
}

pub fn mastoid_is_prominent(m: &MastoidMorph) -> bool {
    m.protrusion > 0.5 || m.size > 0.7
}

pub fn mastoid_overall_weight(m: &MastoidMorph) -> f32 {
    (m.size + m.protrusion) * 0.5
}

pub fn mastoid_blend(a: &MastoidMorph, b: &MastoidMorph, t: f32) -> MastoidMorph {
    let t = t.clamp(0.0, 1.0);
    MastoidMorph {
        size: a.size + (b.size - a.size) * t,
        protrusion: a.protrusion + (b.protrusion - a.protrusion) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* size starts at 0.5 */
        let m = new_mastoid_morph();
        assert!((m.size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_size_clamps() {
        /* clamp above 1 */
        let mut m = new_mastoid_morph();
        mastoid_set_size(&mut m, 2.0);
        assert!((m.size - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_prominent_false() {
        /* default not prominent */
        let m = new_mastoid_morph();
        assert!(!mastoid_is_prominent(&m));
    }

    #[test]
    fn test_overall_weight_range() {
        /* weight in [0,1] */
        let m = new_mastoid_morph();
        let w = mastoid_overall_weight(&m);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend_at_one() {
        /* blend t=1 returns b */
        let a = new_mastoid_morph();
        let mut b = new_mastoid_morph();
        mastoid_set_size(&mut b, 1.0);
        let r = mastoid_blend(&a, &b, 1.0);
        assert!((r.size - 1.0).abs() < 1e-6);
    }
}
