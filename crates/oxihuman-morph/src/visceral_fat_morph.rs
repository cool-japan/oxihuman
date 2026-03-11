// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct VisceralFatMorph {
    pub level: f32,
    pub belly_protrusion: f32,
}

pub fn new_visceral_fat_morph() -> VisceralFatMorph {
    VisceralFatMorph {
        level: 0.0,
        belly_protrusion: 0.0,
    }
}

pub fn visceral_set_level(m: &mut VisceralFatMorph, v: f32) {
    m.level = v.clamp(0.0, 1.0);
    m.belly_protrusion = m.level * 0.8;
}

pub fn visceral_is_high(m: &VisceralFatMorph) -> bool {
    m.level > 0.6
}

pub fn visceral_overall_weight(m: &VisceralFatMorph) -> f32 {
    (m.level + m.belly_protrusion) * 0.5
}

pub fn visceral_blend(a: &VisceralFatMorph, b: &VisceralFatMorph, t: f32) -> VisceralFatMorph {
    let t = t.clamp(0.0, 1.0);
    VisceralFatMorph {
        level: a.level + (b.level - a.level) * t,
        belly_protrusion: a.belly_protrusion + (b.belly_protrusion - a.belly_protrusion) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* default level zero */
        let m = new_visceral_fat_morph();
        assert!((m.level).abs() < 1e-5);
    }

    #[test]
    fn test_set_level() {
        /* set level clamps and updates protrusion */
        let mut m = new_visceral_fat_morph();
        visceral_set_level(&mut m, 0.5);
        assert!((m.level - 0.5).abs() < 1e-5);
        assert!(m.belly_protrusion > 0.0);
    }

    #[test]
    fn test_is_high() {
        /* high threshold */
        let mut m = new_visceral_fat_morph();
        visceral_set_level(&mut m, 0.8);
        assert!(visceral_is_high(&m));
    }

    #[test]
    fn test_overall_weight() {
        /* overall weight >= 0 */
        let m = new_visceral_fat_morph();
        assert!(visceral_overall_weight(&m) >= 0.0);
    }

    #[test]
    fn test_blend() {
        /* blend produces midpoint */
        let a = new_visceral_fat_morph();
        let mut b = new_visceral_fat_morph();
        visceral_set_level(&mut b, 1.0);
        let c = visceral_blend(&a, &b, 0.5);
        assert!((c.level - 0.5).abs() < 1e-5);
    }
}
