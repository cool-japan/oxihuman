// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct JowlMorph {
    pub volume: f32,
    pub ptosis: f32,
    pub width: f32,
}

pub fn new_jowl_morph() -> JowlMorph {
    JowlMorph {
        volume: 0.0,
        ptosis: 0.0,
        width: 0.5,
    }
}

pub fn jowl_set_volume(m: &mut JowlMorph, v: f32) {
    m.volume = v.clamp(0.0, 1.0);
}

pub fn jowl_is_prominent(m: &JowlMorph) -> bool {
    m.volume > 0.5
}

pub fn jowl_overall_weight(m: &JowlMorph) -> f32 {
    (m.volume + m.ptosis) * 0.5
}

pub fn jowl_blend(a: &JowlMorph, b: &JowlMorph, t: f32) -> JowlMorph {
    let t = t.clamp(0.0, 1.0);
    JowlMorph {
        volume: a.volume + (b.volume - a.volume) * t,
        ptosis: a.ptosis + (b.ptosis - a.ptosis) * t,
        width: a.width + (b.width - a.width) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_jowl_morph() {
        /* volume defaults to 0 */
        let m = new_jowl_morph();
        assert!((m.volume - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_jowl_set_volume() {
        /* set and retrieve volume */
        let mut m = new_jowl_morph();
        jowl_set_volume(&mut m, 0.6);
        assert!((m.volume - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_jowl_is_prominent_true() {
        /* volume > 0.5 is prominent */
        let mut m = new_jowl_morph();
        jowl_set_volume(&mut m, 0.7);
        assert!(jowl_is_prominent(&m));
    }

    #[test]
    fn test_jowl_is_prominent_false() {
        /* zero volume not prominent */
        let m = new_jowl_morph();
        assert!(!jowl_is_prominent(&m));
    }

    #[test]
    fn test_jowl_blend() {
        /* blend midpoint */
        let a = new_jowl_morph();
        let mut b = new_jowl_morph();
        b.volume = 1.0;
        let mid = jowl_blend(&a, &b, 0.5);
        assert!((mid.volume - 0.5).abs() < 1e-6);
    }
}
