// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct MarionetteMorph {
    pub depth: f32,
    pub length: f32,
    pub angle_deg: f32,
}

pub fn new_marionette_morph() -> MarionetteMorph {
    MarionetteMorph {
        depth: 0.0,
        length: 0.5,
        angle_deg: 0.0,
    }
}

pub fn marionette_set_depth(m: &mut MarionetteMorph, v: f32) {
    m.depth = v.clamp(0.0, 1.0);
}

pub fn marionette_is_pronounced(m: &MarionetteMorph) -> bool {
    m.depth > 0.5
}

pub fn marionette_overall_weight(m: &MarionetteMorph) -> f32 {
    (m.depth + m.length) * 0.5
}

pub fn marionette_blend(a: &MarionetteMorph, b: &MarionetteMorph, t: f32) -> MarionetteMorph {
    let t = t.clamp(0.0, 1.0);
    MarionetteMorph {
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
        let m = new_marionette_morph();
        assert!((m.depth - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamps() {
        /* clamp above 1 */
        let mut m = new_marionette_morph();
        marionette_set_depth(&mut m, 2.0);
        assert!((m.depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_pronounced_true() {
        /* depth 0.7 => pronounced */
        let mut m = new_marionette_morph();
        marionette_set_depth(&mut m, 0.7);
        assert!(marionette_is_pronounced(&m));
    }

    #[test]
    fn test_is_pronounced_false() {
        /* default not pronounced */
        let m = new_marionette_morph();
        assert!(!marionette_is_pronounced(&m));
    }

    #[test]
    fn test_overall_weight_range() {
        /* weight in [0,1] */
        let m = new_marionette_morph();
        let w = marionette_overall_weight(&m);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend_at_one() {
        /* blend t=1 returns b */
        let a = new_marionette_morph();
        let mut b = new_marionette_morph();
        marionette_set_depth(&mut b, 1.0);
        let r = marionette_blend(&a, &b, 1.0);
        assert!((r.depth - 1.0).abs() < 1e-6);
    }
}
