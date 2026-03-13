// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct PlatysmaeMorph {
    pub band_prominence: f32,
    pub width: f32,
    pub ptosis: f32,
}

pub fn new_platysmae_morph() -> PlatysmaeMorph {
    PlatysmaeMorph {
        band_prominence: 0.0,
        width: 0.5,
        ptosis: 0.0,
    }
}

pub fn platysmae_set_band_prominence(m: &mut PlatysmaeMorph, v: f32) {
    m.band_prominence = v.clamp(0.0, 1.0);
}

pub fn platysmae_shows_bands(m: &PlatysmaeMorph) -> bool {
    m.band_prominence > 0.4
}

pub fn platysmae_overall_weight(m: &PlatysmaeMorph) -> f32 {
    (m.band_prominence + m.width) * 0.5
}

pub fn platysmae_blend(a: &PlatysmaeMorph, b: &PlatysmaeMorph, t: f32) -> PlatysmaeMorph {
    let t = t.clamp(0.0, 1.0);
    PlatysmaeMorph {
        band_prominence: a.band_prominence + (b.band_prominence - a.band_prominence) * t,
        width: a.width + (b.width - a.width) * t,
        ptosis: a.ptosis + (b.ptosis - a.ptosis) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_platysmae_morph() {
        /* band_prominence starts at 0 */
        let m = new_platysmae_morph();
        assert!((m.band_prominence - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_platysmae_set_band_prominence() {
        /* set and retrieve band prominence */
        let mut m = new_platysmae_morph();
        platysmae_set_band_prominence(&mut m, 0.5);
        assert!((m.band_prominence - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_platysmae_shows_bands_true() {
        /* > 0.4 shows bands */
        let mut m = new_platysmae_morph();
        platysmae_set_band_prominence(&mut m, 0.6);
        assert!(platysmae_shows_bands(&m));
    }

    #[test]
    fn test_platysmae_shows_bands_false() {
        /* 0 no bands */
        let m = new_platysmae_morph();
        assert!(!platysmae_shows_bands(&m));
    }

    #[test]
    fn test_platysmae_blend() {
        /* blend midpoint */
        let a = new_platysmae_morph();
        let mut b = new_platysmae_morph();
        b.band_prominence = 1.0;
        let mid = platysmae_blend(&a, &b, 0.5);
        assert!((mid.band_prominence - 0.5).abs() < 1e-6);
    }
}
