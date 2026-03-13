// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct InfraorbitalRimMorph {
    pub depth: f32,
    pub width: f32,
    pub angle_deg: f32,
}

pub fn new_infraorbital_rim_morph() -> InfraorbitalRimMorph {
    InfraorbitalRimMorph {
        depth: 0.0,
        width: 0.5,
        angle_deg: 0.0,
    }
}

pub fn infraorbital_set_depth(m: &mut InfraorbitalRimMorph, v: f32) {
    m.depth = v.clamp(0.0, 1.0);
}

pub fn infraorbital_set_width(m: &mut InfraorbitalRimMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

pub fn infraorbital_overall_weight(m: &InfraorbitalRimMorph) -> f32 {
    (m.depth + m.width) * 0.5
}

pub fn infraorbital_blend(
    a: &InfraorbitalRimMorph,
    b: &InfraorbitalRimMorph,
    t: f32,
) -> InfraorbitalRimMorph {
    let t = t.clamp(0.0, 1.0);
    InfraorbitalRimMorph {
        depth: a.depth + (b.depth - a.depth) * t,
        width: a.width + (b.width - a.width) * t,
        angle_deg: a.angle_deg + (b.angle_deg - a.angle_deg) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* depth starts at 0 */
        let m = new_infraorbital_rim_morph();
        assert!((m.depth - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamps() {
        /* clamp above 1 */
        let mut m = new_infraorbital_rim_morph();
        infraorbital_set_depth(&mut m, 2.0);
        assert!((m.depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        /* set width */
        let mut m = new_infraorbital_rim_morph();
        infraorbital_set_width(&mut m, 0.8);
        assert!((m.width - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_overall_weight_range() {
        /* weight in [0,1] */
        let m = new_infraorbital_rim_morph();
        let w = infraorbital_overall_weight(&m);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend_at_zero() {
        /* blend t=0 returns a */
        let a = new_infraorbital_rim_morph();
        let mut b = new_infraorbital_rim_morph();
        infraorbital_set_depth(&mut b, 1.0);
        let r = infraorbital_blend(&a, &b, 0.0);
        assert!((r.depth - a.depth).abs() < 1e-6);
    }

    #[test]
    fn test_blend_at_one() {
        /* blend t=1 returns b */
        let a = new_infraorbital_rim_morph();
        let mut b = new_infraorbital_rim_morph();
        infraorbital_set_depth(&mut b, 1.0);
        let r = infraorbital_blend(&a, &b, 1.0);
        assert!((r.depth - 1.0).abs() < 1e-6);
    }
}
