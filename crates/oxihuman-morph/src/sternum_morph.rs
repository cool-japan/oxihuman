// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct SternumMorph {
    pub length: f32,
    pub width: f32,
    pub protrusion: f32,
    pub angle_deg: f32,
}

pub fn new_sternum_morph() -> SternumMorph {
    SternumMorph {
        length: 0.5,
        width: 0.5,
        protrusion: 0.0,
        angle_deg: 0.0,
    }
}

pub fn sternum_set_protrusion(m: &mut SternumMorph, v: f32) {
    m.protrusion = v.clamp(0.0, 1.0);
}

pub fn sternum_is_protruding(m: &SternumMorph) -> bool {
    m.protrusion > 0.5
}

pub fn sternum_overall_weight(m: &SternumMorph) -> f32 {
    (m.length + m.width + m.protrusion) / 3.0
}

pub fn sternum_blend(a: &SternumMorph, b: &SternumMorph, t: f32) -> SternumMorph {
    let t = t.clamp(0.0, 1.0);
    SternumMorph {
        length: a.length + (b.length - a.length) * t,
        width: a.width + (b.width - a.width) * t,
        protrusion: a.protrusion + (b.protrusion - a.protrusion) * t,
        angle_deg: a.angle_deg + (b.angle_deg - a.angle_deg) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* protrusion starts at 0 */
        let m = new_sternum_morph();
        assert!((m.protrusion - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_protrusion_clamps() {
        /* clamp above 1 */
        let mut m = new_sternum_morph();
        sternum_set_protrusion(&mut m, 3.0);
        assert!((m.protrusion - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_protruding_true() {
        /* protrusion 0.7 => protruding */
        let mut m = new_sternum_morph();
        sternum_set_protrusion(&mut m, 0.7);
        assert!(sternum_is_protruding(&m));
    }

    #[test]
    fn test_is_protruding_false() {
        /* default not protruding */
        let m = new_sternum_morph();
        assert!(!sternum_is_protruding(&m));
    }

    #[test]
    fn test_overall_weight_range() {
        /* weight in [0,1] */
        let m = new_sternum_morph();
        let w = sternum_overall_weight(&m);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend_midpoint() {
        /* blend at 0.5 */
        let a = new_sternum_morph();
        let mut b = new_sternum_morph();
        sternum_set_protrusion(&mut b, 1.0);
        let r = sternum_blend(&a, &b, 0.5);
        assert!((r.protrusion - 0.5).abs() < 1e-5);
    }
}
