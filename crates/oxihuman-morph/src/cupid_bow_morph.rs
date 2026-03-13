// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct CupidBowMorph {
    pub peak_height: f32,
    pub valley_depth: f32,
    pub width: f32,
}

pub fn new_cupid_bow_morph() -> CupidBowMorph {
    CupidBowMorph {
        peak_height: 0.0,
        valley_depth: 0.0,
        width: 0.0,
    }
}

pub fn cb_set_peak_height(m: &mut CupidBowMorph, v: f32) {
    m.peak_height = v.clamp(0.0, 1.0);
}

pub fn cb_set_valley_depth(m: &mut CupidBowMorph, v: f32) {
    m.valley_depth = v.clamp(0.0, 1.0);
}

pub fn cb_set_width(m: &mut CupidBowMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

pub fn cb_overall_weight(m: &CupidBowMorph) -> f32 {
    (m.peak_height + m.valley_depth + m.width) / 3.0
}

pub fn cb_blend(a: &CupidBowMorph, b: &CupidBowMorph, t: f32) -> CupidBowMorph {
    let t = t.clamp(0.0, 1.0);
    CupidBowMorph {
        peak_height: a.peak_height + (b.peak_height - a.peak_height) * t,
        valley_depth: a.valley_depth + (b.valley_depth - a.valley_depth) * t,
        width: a.width + (b.width - a.width) * t,
    }
}

pub fn cb_is_pronounced(m: &CupidBowMorph) -> bool {
    m.peak_height > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* zero init */
        let m = new_cupid_bow_morph();
        assert_eq!(m.peak_height, 0.0);
        assert_eq!(m.valley_depth, 0.0);
        assert_eq!(m.width, 0.0);
    }

    #[test]
    fn test_set_peak_height() {
        /* valid value */
        let mut m = new_cupid_bow_morph();
        cb_set_peak_height(&mut m, 0.6);
        assert!((m.peak_height - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_peak_height_clamp_high() {
        /* clamp high */
        let mut m = new_cupid_bow_morph();
        cb_set_peak_height(&mut m, 3.0);
        assert_eq!(m.peak_height, 1.0);
    }

    #[test]
    fn test_set_valley_depth_clamp_low() {
        /* clamp low */
        let mut m = new_cupid_bow_morph();
        cb_set_valley_depth(&mut m, -0.5);
        assert_eq!(m.valley_depth, 0.0);
    }

    #[test]
    fn test_set_width() {
        /* valid width */
        let mut m = new_cupid_bow_morph();
        cb_set_width(&mut m, 0.5);
        assert!((m.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = CupidBowMorph {
            peak_height: 0.3,
            valley_depth: 0.6,
            width: 0.9,
        };
        assert!((cb_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_pronounced_false() {
        /* default not pronounced */
        let m = new_cupid_bow_morph();
        assert!(!cb_is_pronounced(&m));
    }

    #[test]
    fn test_is_pronounced_true() {
        /* above 0.5 */
        let m = CupidBowMorph {
            peak_height: 0.8,
            valley_depth: 0.0,
            width: 0.0,
        };
        assert!(cb_is_pronounced(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint blend */
        let a = CupidBowMorph {
            peak_height: 0.0,
            valley_depth: 0.0,
            width: 0.0,
        };
        let b = CupidBowMorph {
            peak_height: 1.0,
            valley_depth: 1.0,
            width: 1.0,
        };
        let c = cb_blend(&a, &b, 0.5);
        assert!((c.peak_height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let m = CupidBowMorph {
            peak_height: 0.2,
            valley_depth: 0.5,
            width: 0.8,
        };
        let m2 = m.clone();
        assert_eq!(m.width, m2.width);
    }
}
