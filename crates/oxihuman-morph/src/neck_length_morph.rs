// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct NeckLengthMorph {
    pub length: f32,
    pub width: f32,
    pub forward_tilt: f32,
}

pub fn new_neck_length_morph() -> NeckLengthMorph {
    NeckLengthMorph {
        length: 0.5,
        width: 0.5,
        forward_tilt: 0.0,
    }
}

pub fn neck_set_length(m: &mut NeckLengthMorph, v: f32) {
    m.length = v.clamp(0.0, 1.0);
}

pub fn neck_set_width(m: &mut NeckLengthMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

pub fn neck_is_long(m: &NeckLengthMorph) -> bool {
    m.length > 0.6
}

pub fn neck_overall_weight(m: &NeckLengthMorph) -> f32 {
    (m.length + m.width + m.forward_tilt) / 3.0
}

pub fn neck_blend(a: &NeckLengthMorph, b: &NeckLengthMorph, t: f32) -> NeckLengthMorph {
    let t = t.clamp(0.0, 1.0);
    NeckLengthMorph {
        length: a.length + (b.length - a.length) * t,
        width: a.width + (b.width - a.width) * t,
        forward_tilt: a.forward_tilt + (b.forward_tilt - a.forward_tilt) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* length starts at 0.5 */
        let m = new_neck_length_morph();
        assert!((m.length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_length_clamps_high() {
        /* clamp above 1 */
        let mut m = new_neck_length_morph();
        neck_set_length(&mut m, 2.0);
        assert!((m.length - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_length_clamps_low() {
        /* clamp below 0 */
        let mut m = new_neck_length_morph();
        neck_set_length(&mut m, -1.0);
        assert!((m.length - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        /* set width */
        let mut m = new_neck_length_morph();
        neck_set_width(&mut m, 0.7);
        assert!((m.width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_is_long_true() {
        /* length 0.8 => long */
        let mut m = new_neck_length_morph();
        neck_set_length(&mut m, 0.8);
        assert!(neck_is_long(&m));
    }

    #[test]
    fn test_is_long_false() {
        /* default 0.5 not long */
        let m = new_neck_length_morph();
        assert!(!neck_is_long(&m));
    }

    #[test]
    fn test_blend_at_one() {
        /* blend t=1 returns b */
        let a = new_neck_length_morph();
        let mut b = new_neck_length_morph();
        neck_set_length(&mut b, 1.0);
        let r = neck_blend(&a, &b, 1.0);
        assert!((r.length - 1.0).abs() < 1e-6);
    }
}
