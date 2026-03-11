// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct CupidsBowMorph {
    pub peak_height: f32,
    pub valley_depth: f32,
    pub width: f32,
}

pub fn new_cupids_bow_morph() -> CupidsBowMorph {
    CupidsBowMorph {
        peak_height: 0.0,
        valley_depth: 0.0,
        width: 0.0,
    }
}

pub fn cupids_set_peak(m: &mut CupidsBowMorph, v: f32) {
    m.peak_height = v.clamp(0.0, 1.0);
}

pub fn cupids_is_defined(m: &CupidsBowMorph) -> bool {
    m.peak_height > 0.3
}

pub fn cupids_overall_weight(m: &CupidsBowMorph) -> f32 {
    (m.peak_height.abs() + m.valley_depth.abs() + m.width.abs()) / 3.0
}

pub fn cupids_blend(a: &CupidsBowMorph, b: &CupidsBowMorph, t: f32) -> CupidsBowMorph {
    let t = t.clamp(0.0, 1.0);
    CupidsBowMorph {
        peak_height: a.peak_height + (b.peak_height - a.peak_height) * t,
        valley_depth: a.valley_depth + (b.valley_depth - a.valley_depth) * t,
        width: a.width + (b.width - a.width) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cupids_bow_morph() {
        /* peaks at zero by default */
        let m = new_cupids_bow_morph();
        assert_eq!(m.peak_height, 0.0);
    }

    #[test]
    fn test_cupids_set_peak() {
        /* peak is set */
        let mut m = new_cupids_bow_morph();
        cupids_set_peak(&mut m, 0.5);
        assert!((m.peak_height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_cupids_is_defined_false() {
        /* not defined at zero */
        let m = new_cupids_bow_morph();
        assert!(!cupids_is_defined(&m));
    }

    #[test]
    fn test_cupids_is_defined_true() {
        /* defined when peak > 0.3 */
        let mut m = new_cupids_bow_morph();
        m.peak_height = 0.4;
        assert!(cupids_is_defined(&m));
    }

    #[test]
    fn test_cupids_blend() {
        /* blend at t=0.5 gives midpoint */
        let a = new_cupids_bow_morph();
        let b = CupidsBowMorph {
            peak_height: 1.0,
            valley_depth: 1.0,
            width: 1.0,
        };
        let r = cupids_blend(&a, &b, 0.5);
        assert!((r.peak_height - 0.5).abs() < 1e-6);
    }
}
