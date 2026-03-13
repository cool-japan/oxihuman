// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct OcciputMorph {
    pub protrusion: f32,
    pub flatness: f32,
    pub width: f32,
}

pub fn new_occiput_morph() -> OcciputMorph {
    OcciputMorph {
        protrusion: 0.0,
        flatness: 0.0,
        width: 0.5,
    }
}

pub fn occiput_set_protrusion(m: &mut OcciputMorph, v: f32) {
    m.protrusion = v.clamp(0.0, 1.0);
}

pub fn occiput_set_flatness(m: &mut OcciputMorph, v: f32) {
    m.flatness = v.clamp(0.0, 1.0);
}

pub fn occiput_overall_weight(m: &OcciputMorph) -> f32 {
    (m.protrusion + m.flatness + m.width) / 3.0
}

pub fn occiput_blend(a: &OcciputMorph, b: &OcciputMorph, t: f32) -> OcciputMorph {
    let t = t.clamp(0.0, 1.0);
    OcciputMorph {
        protrusion: a.protrusion + (b.protrusion - a.protrusion) * t,
        flatness: a.flatness + (b.flatness - a.flatness) * t,
        width: a.width + (b.width - a.width) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* protrusion starts at 0 */
        let m = new_occiput_morph();
        assert!((m.protrusion - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_protrusion_clamps_high() {
        /* clamp above 1 */
        let mut m = new_occiput_morph();
        occiput_set_protrusion(&mut m, 2.0);
        assert!((m.protrusion - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_flatness() {
        /* set flatness */
        let mut m = new_occiput_morph();
        occiput_set_flatness(&mut m, 0.6);
        assert!((m.flatness - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_overall_weight_in_range() {
        /* weight in [0,1] */
        let m = new_occiput_morph();
        let w = occiput_overall_weight(&m);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend_at_zero() {
        /* blend t=0 returns a */
        let a = new_occiput_morph();
        let mut b = new_occiput_morph();
        occiput_set_protrusion(&mut b, 1.0);
        let r = occiput_blend(&a, &b, 0.0);
        assert!((r.protrusion - a.protrusion).abs() < 1e-6);
    }

    #[test]
    fn test_blend_at_one() {
        /* blend t=1 returns b */
        let a = new_occiput_morph();
        let mut b = new_occiput_morph();
        occiput_set_protrusion(&mut b, 1.0);
        let r = occiput_blend(&a, &b, 1.0);
        assert!((r.protrusion - 1.0).abs() < 1e-6);
    }
}
