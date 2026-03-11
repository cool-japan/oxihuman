// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct SubmentalMorph {
    pub fat_volume: f32,
    pub ptosis: f32,
    pub neck_angle_deg: f32,
}

pub fn new_submental_morph() -> SubmentalMorph {
    SubmentalMorph {
        fat_volume: 0.0,
        ptosis: 0.0,
        neck_angle_deg: 0.0,
    }
}

pub fn submental_set_fat(m: &mut SubmentalMorph, v: f32) {
    m.fat_volume = v.clamp(0.0, 1.0);
}

pub fn submental_has_double_chin(m: &SubmentalMorph) -> bool {
    m.fat_volume > 0.5
}

pub fn submental_overall_weight(m: &SubmentalMorph) -> f32 {
    (m.fat_volume + m.ptosis) * 0.5
}

pub fn submental_blend(a: &SubmentalMorph, b: &SubmentalMorph, t: f32) -> SubmentalMorph {
    let t = t.clamp(0.0, 1.0);
    SubmentalMorph {
        fat_volume: a.fat_volume + (b.fat_volume - a.fat_volume) * t,
        ptosis: a.ptosis + (b.ptosis - a.ptosis) * t,
        neck_angle_deg: a.neck_angle_deg + (b.neck_angle_deg - a.neck_angle_deg) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_submental_morph() {
        /* fat_volume starts at 0 */
        let m = new_submental_morph();
        assert!((m.fat_volume - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_submental_set_fat() {
        /* set and retrieve fat */
        let mut m = new_submental_morph();
        submental_set_fat(&mut m, 0.7);
        assert!((m.fat_volume - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_submental_has_double_chin_true() {
        /* fat > 0.5 means double chin */
        let mut m = new_submental_morph();
        submental_set_fat(&mut m, 0.6);
        assert!(submental_has_double_chin(&m));
    }

    #[test]
    fn test_submental_has_double_chin_false() {
        /* no fat, no double chin */
        let m = new_submental_morph();
        assert!(!submental_has_double_chin(&m));
    }

    #[test]
    fn test_submental_overall_weight() {
        /* weight is average of fat and ptosis */
        let mut m = new_submental_morph();
        submental_set_fat(&mut m, 1.0);
        m.ptosis = 1.0;
        assert!((submental_overall_weight(&m) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_submental_blend() {
        /* blend t=0 gives a */
        let a = new_submental_morph();
        let mut b = new_submental_morph();
        b.fat_volume = 1.0;
        let out = submental_blend(&a, &b, 0.0);
        assert!((out.fat_volume - 0.0).abs() < 1e-6);
    }
}
