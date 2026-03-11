// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct GnathionMorph {
    pub vertical_drop: f32,
    pub roundness: f32,
}

pub fn new_gnathion_morph() -> GnathionMorph {
    GnathionMorph {
        vertical_drop: 0.0,
        roundness: 0.0,
    }
}

pub fn gn_set_vertical_drop(m: &mut GnathionMorph, v: f32) {
    m.vertical_drop = v.clamp(0.0, 1.0);
}

pub fn gn_set_roundness(m: &mut GnathionMorph, v: f32) {
    m.roundness = v.clamp(0.0, 1.0);
}

pub fn gn_overall_weight(m: &GnathionMorph) -> f32 {
    (m.vertical_drop + m.roundness) / 2.0
}

pub fn gn_blend(a: &GnathionMorph, b: &GnathionMorph, t: f32) -> GnathionMorph {
    let t = t.clamp(0.0, 1.0);
    GnathionMorph {
        vertical_drop: a.vertical_drop + (b.vertical_drop - a.vertical_drop) * t,
        roundness: a.roundness + (b.roundness - a.roundness) * t,
    }
}

pub fn gn_is_elongated(m: &GnathionMorph) -> bool {
    m.vertical_drop > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* zero init */
        let m = new_gnathion_morph();
        assert_eq!(m.vertical_drop, 0.0);
        assert_eq!(m.roundness, 0.0);
    }

    #[test]
    fn test_set_vertical_drop() {
        /* valid value */
        let mut m = new_gnathion_morph();
        gn_set_vertical_drop(&mut m, 0.6);
        assert!((m.vertical_drop - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_vertical_drop_clamp_high() {
        /* clamp high */
        let mut m = new_gnathion_morph();
        gn_set_vertical_drop(&mut m, 5.0);
        assert_eq!(m.vertical_drop, 1.0);
    }

    #[test]
    fn test_set_roundness() {
        /* valid roundness */
        let mut m = new_gnathion_morph();
        gn_set_roundness(&mut m, 0.4);
        assert!((m.roundness - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_set_roundness_clamp_low() {
        /* clamp low */
        let mut m = new_gnathion_morph();
        gn_set_roundness(&mut m, -1.0);
        assert_eq!(m.roundness, 0.0);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = GnathionMorph {
            vertical_drop: 0.4,
            roundness: 0.8,
        };
        assert!((gn_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_elongated_false() {
        /* default not elongated */
        let m = new_gnathion_morph();
        assert!(!gn_is_elongated(&m));
    }

    #[test]
    fn test_is_elongated_true() {
        /* above 0.5 */
        let m = GnathionMorph {
            vertical_drop: 0.8,
            roundness: 0.0,
        };
        assert!(gn_is_elongated(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint blend */
        let a = GnathionMorph {
            vertical_drop: 0.0,
            roundness: 0.0,
        };
        let b = GnathionMorph {
            vertical_drop: 1.0,
            roundness: 1.0,
        };
        let c = gn_blend(&a, &b, 0.5);
        assert!((c.vertical_drop - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let m = GnathionMorph {
            vertical_drop: 0.3,
            roundness: 0.7,
        };
        let m2 = m.clone();
        assert_eq!(m.roundness, m2.roundness);
    }
}
