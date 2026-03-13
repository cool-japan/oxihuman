// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct BuccalFatMorph {
    pub volume: f32,
    pub prominence: f32,
}

pub fn new_buccal_fat_morph() -> BuccalFatMorph {
    BuccalFatMorph {
        volume: 0.0,
        prominence: 0.0,
    }
}

pub fn buccal_set_volume(m: &mut BuccalFatMorph, v: f32) {
    m.volume = v.clamp(0.0, 1.0);
}

pub fn buccal_is_prominent(m: &BuccalFatMorph) -> bool {
    m.volume > 0.5
}

pub fn buccal_overall_weight(m: &BuccalFatMorph) -> f32 {
    (m.volume + m.prominence) * 0.5
}

pub fn buccal_blend(a: &BuccalFatMorph, b: &BuccalFatMorph, t: f32) -> BuccalFatMorph {
    let t = t.clamp(0.0, 1.0);
    BuccalFatMorph {
        volume: a.volume + (b.volume - a.volume) * t,
        prominence: a.prominence + (b.prominence - a.prominence) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buccal_fat_morph() {
        /* volume starts at 0 */
        let m = new_buccal_fat_morph();
        assert!((m.volume - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_buccal_set_volume() {
        /* set and retrieve volume */
        let mut m = new_buccal_fat_morph();
        buccal_set_volume(&mut m, 0.4);
        assert!((m.volume - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_buccal_is_prominent_true() {
        /* volume > 0.5 is prominent */
        let mut m = new_buccal_fat_morph();
        buccal_set_volume(&mut m, 0.9);
        assert!(buccal_is_prominent(&m));
    }

    #[test]
    fn test_buccal_is_prominent_false() {
        /* zero volume is not prominent */
        let m = new_buccal_fat_morph();
        assert!(!buccal_is_prominent(&m));
    }

    #[test]
    fn test_buccal_blend() {
        /* blend t=1.0 gives b */
        let a = new_buccal_fat_morph();
        let mut b = new_buccal_fat_morph();
        b.volume = 0.8;
        let out = buccal_blend(&a, &b, 1.0);
        assert!((out.volume - 0.8).abs() < 1e-6);
    }
}
