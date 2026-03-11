// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct NasalDorsumMorph {
    pub height: f32,
    pub width: f32,
    pub hump: f32,
}

pub fn new_nasal_dorsum_morph() -> NasalDorsumMorph {
    NasalDorsumMorph {
        height: 0.0,
        width: 0.0,
        hump: 0.0,
    }
}

pub fn nd_set_height(m: &mut NasalDorsumMorph, v: f32) {
    m.height = v.clamp(0.0, 1.0);
}

pub fn nd_set_width(m: &mut NasalDorsumMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

pub fn nd_set_hump(m: &mut NasalDorsumMorph, v: f32) {
    m.hump = v.clamp(0.0, 1.0);
}

pub fn nd_overall_weight(m: &NasalDorsumMorph) -> f32 {
    (m.height + m.width + m.hump) / 3.0
}

pub fn nd_blend(a: &NasalDorsumMorph, b: &NasalDorsumMorph, t: f32) -> NasalDorsumMorph {
    let t = t.clamp(0.0, 1.0);
    NasalDorsumMorph {
        height: a.height + (b.height - a.height) * t,
        width: a.width + (b.width - a.width) * t,
        hump: a.hump + (b.hump - a.hump) * t,
    }
}

pub fn nd_is_humped(m: &NasalDorsumMorph) -> bool {
    m.hump > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all zero */
        let m = new_nasal_dorsum_morph();
        assert_eq!(m.hump, 0.0);
    }

    #[test]
    fn test_set_height() {
        /* valid value */
        let mut m = new_nasal_dorsum_morph();
        nd_set_height(&mut m, 0.5);
        assert!((m.height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_height_clamp() {
        /* clamp high */
        let mut m = new_nasal_dorsum_morph();
        nd_set_height(&mut m, 5.0);
        assert_eq!(m.height, 1.0);
    }

    #[test]
    fn test_set_width_clamp() {
        /* clamp low */
        let mut m = new_nasal_dorsum_morph();
        nd_set_width(&mut m, -1.0);
        assert_eq!(m.width, 0.0);
    }

    #[test]
    fn test_set_hump() {
        /* valid value */
        let mut m = new_nasal_dorsum_morph();
        nd_set_hump(&mut m, 0.8);
        assert!((m.hump - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = NasalDorsumMorph {
            height: 0.3,
            width: 0.6,
            hump: 0.9,
        };
        assert!((nd_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_humped_false() {
        /* default not humped */
        let m = new_nasal_dorsum_morph();
        assert!(!nd_is_humped(&m));
    }

    #[test]
    fn test_is_humped_true() {
        /* above threshold */
        let m = NasalDorsumMorph {
            height: 0.0,
            width: 0.0,
            hump: 0.9,
        };
        assert!(nd_is_humped(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = NasalDorsumMorph {
            height: 0.0,
            width: 0.0,
            hump: 0.0,
        };
        let b = NasalDorsumMorph {
            height: 1.0,
            width: 1.0,
            hump: 1.0,
        };
        let c = nd_blend(&a, &b, 0.5);
        assert!((c.hump - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let m = NasalDorsumMorph {
            height: 0.3,
            width: 0.4,
            hump: 0.5,
        };
        let m2 = m.clone();
        assert_eq!(m.hump, m2.hump);
    }
}
