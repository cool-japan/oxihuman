// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct HairlineMorph {
    pub position: f32,
    pub recession_left: f32,
    pub recession_right: f32,
    pub density: f32,
}

pub fn new_hairline_morph() -> HairlineMorph {
    HairlineMorph {
        position: 0.0,
        recession_left: 0.0,
        recession_right: 0.0,
        density: 1.0,
    }
}

pub fn hairline_set_position(m: &mut HairlineMorph, v: f32) {
    m.position = v.clamp(0.0, 1.0);
}

pub fn hairline_recession_symmetric(m: &mut HairlineMorph, v: f32) {
    let v = v.clamp(0.0, 1.0);
    m.recession_left = v;
    m.recession_right = v;
}

pub fn hairline_is_receded(m: &HairlineMorph) -> bool {
    m.position > 0.5 || m.recession_left > 0.3 || m.recession_right > 0.3
}

pub fn hairline_overall_weight(m: &HairlineMorph) -> f32 {
    (m.position + m.recession_left + m.recession_right) / 3.0
}

pub fn hairline_blend(a: &HairlineMorph, b: &HairlineMorph, t: f32) -> HairlineMorph {
    let t = t.clamp(0.0, 1.0);
    HairlineMorph {
        position: a.position + (b.position - a.position) * t,
        recession_left: a.recession_left + (b.recession_left - a.recession_left) * t,
        recession_right: a.recession_right + (b.recession_right - a.recession_right) * t,
        density: a.density + (b.density - a.density) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* default position is 0 (high hairline) */
        let m = new_hairline_morph();
        assert!((m.position - 0.0).abs() < 1e-6);
        assert!((m.density - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_position_clamps_max() {
        /* position clamped to 1.0 */
        let mut m = new_hairline_morph();
        hairline_set_position(&mut m, 2.5);
        assert!((m.position - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_position_clamps_min() {
        /* position clamped to 0.0 */
        let mut m = new_hairline_morph();
        hairline_set_position(&mut m, -1.0);
        assert!((m.position - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_recession_symmetric_sets_both() {
        /* both sides get same value */
        let mut m = new_hairline_morph();
        hairline_recession_symmetric(&mut m, 0.5);
        assert!((m.recession_left - 0.5).abs() < 1e-6);
        assert!((m.recession_right - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_is_receded_by_position() {
        /* position 0.8 => receded */
        let mut m = new_hairline_morph();
        hairline_set_position(&mut m, 0.8);
        assert!(hairline_is_receded(&m));
    }

    #[test]
    fn test_not_receded_default() {
        /* default is not receded */
        let m = new_hairline_morph();
        assert!(!hairline_is_receded(&m));
    }

    #[test]
    fn test_blend_at_zero() {
        /* blend t=0 returns a */
        let a = new_hairline_morph();
        let mut b = new_hairline_morph();
        hairline_set_position(&mut b, 1.0);
        let r = hairline_blend(&a, &b, 0.0);
        assert!((r.position - a.position).abs() < 1e-6);
    }
}
