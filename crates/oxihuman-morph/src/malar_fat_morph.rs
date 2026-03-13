// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct MalarFatMorph {
    pub volume: f32,
    pub position_y: f32,
    pub lateral_extent: f32,
}

pub fn new_malar_fat_morph() -> MalarFatMorph {
    MalarFatMorph {
        volume: 0.0,
        position_y: 0.0,
        lateral_extent: 0.5,
    }
}

pub fn malar_fat_set_volume(m: &mut MalarFatMorph, v: f32) {
    m.volume = v.clamp(0.0, 1.0);
}

pub fn malar_fat_is_full(m: &MalarFatMorph) -> bool {
    m.volume > 0.6
}

pub fn malar_fat_overall_weight(m: &MalarFatMorph) -> f32 {
    (m.volume + m.lateral_extent) * 0.5
}

pub fn malar_fat_blend(a: &MalarFatMorph, b: &MalarFatMorph, t: f32) -> MalarFatMorph {
    let t = t.clamp(0.0, 1.0);
    MalarFatMorph {
        volume: a.volume + (b.volume - a.volume) * t,
        position_y: a.position_y + (b.position_y - a.position_y) * t,
        lateral_extent: a.lateral_extent + (b.lateral_extent - a.lateral_extent) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_malar_fat_morph() {
        /* volume starts at 0 */
        let m = new_malar_fat_morph();
        assert!((m.volume - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_malar_fat_set_volume() {
        /* set and retrieve volume */
        let mut m = new_malar_fat_morph();
        malar_fat_set_volume(&mut m, 0.5);
        assert!((m.volume - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_malar_fat_is_full_true() {
        /* volume > 0.6 is full */
        let mut m = new_malar_fat_morph();
        malar_fat_set_volume(&mut m, 0.8);
        assert!(malar_fat_is_full(&m));
    }

    #[test]
    fn test_malar_fat_is_full_false() {
        /* low volume is not full */
        let m = new_malar_fat_morph();
        assert!(!malar_fat_is_full(&m));
    }

    #[test]
    fn test_malar_fat_overall_weight() {
        /* weight average of volume and lateral_extent */
        let mut m = new_malar_fat_morph();
        malar_fat_set_volume(&mut m, 1.0);
        m.lateral_extent = 1.0;
        assert!((malar_fat_overall_weight(&m) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_malar_fat_blend() {
        /* blend midpoint */
        let a = new_malar_fat_morph();
        let mut b = new_malar_fat_morph();
        b.volume = 1.0;
        let mid = malar_fat_blend(&a, &b, 0.5);
        assert!((mid.volume - 0.5).abs() < 1e-6);
    }
}
