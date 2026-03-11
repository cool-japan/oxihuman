// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct MalarEminenceMorph {
    pub left: f32,
    pub right: f32,
    pub projection: f32,
}

pub fn new_malar_eminence_morph() -> MalarEminenceMorph {
    MalarEminenceMorph {
        left: 0.0,
        right: 0.0,
        projection: 0.0,
    }
}

pub fn mal_set_left(m: &mut MalarEminenceMorph, v: f32) {
    m.left = v.clamp(0.0, 1.0);
}

pub fn mal_set_right(m: &mut MalarEminenceMorph, v: f32) {
    m.right = v.clamp(0.0, 1.0);
}

pub fn mal_set_projection(m: &mut MalarEminenceMorph, v: f32) {
    m.projection = v.clamp(0.0, 1.0);
}

pub fn mal_overall_weight(m: &MalarEminenceMorph) -> f32 {
    (m.left + m.right + m.projection) / 3.0
}

pub fn mal_blend(a: &MalarEminenceMorph, b: &MalarEminenceMorph, t: f32) -> MalarEminenceMorph {
    let t = t.clamp(0.0, 1.0);
    MalarEminenceMorph {
        left: a.left + (b.left - a.left) * t,
        right: a.right + (b.right - a.right) * t,
        projection: a.projection + (b.projection - a.projection) * t,
    }
}

pub fn mal_is_prominent(m: &MalarEminenceMorph) -> bool {
    m.projection > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all zero */
        let m = new_malar_eminence_morph();
        assert_eq!(m.projection, 0.0);
    }

    #[test]
    fn test_set_left_clamp() {
        /* clamps high */
        let mut m = new_malar_eminence_morph();
        mal_set_left(&mut m, 3.0);
        assert_eq!(m.left, 1.0);
    }

    #[test]
    fn test_set_right_clamp() {
        /* clamps low */
        let mut m = new_malar_eminence_morph();
        mal_set_right(&mut m, -1.0);
        assert_eq!(m.right, 0.0);
    }

    #[test]
    fn test_set_projection() {
        /* valid value */
        let mut m = new_malar_eminence_morph();
        mal_set_projection(&mut m, 0.7);
        assert!((m.projection - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = MalarEminenceMorph {
            left: 0.3,
            right: 0.6,
            projection: 0.9,
        };
        assert!((mal_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_prominent_false() {
        /* default not prominent */
        let m = new_malar_eminence_morph();
        assert!(!mal_is_prominent(&m));
    }

    #[test]
    fn test_is_prominent_true() {
        /* projection > 0.5 */
        let m = MalarEminenceMorph {
            left: 0.0,
            right: 0.0,
            projection: 0.8,
        };
        assert!(mal_is_prominent(&m));
    }

    #[test]
    fn test_blend_midpoint() {
        /* midpoint */
        let a = MalarEminenceMorph {
            left: 0.0,
            right: 0.0,
            projection: 0.0,
        };
        let b = MalarEminenceMorph {
            left: 1.0,
            right: 1.0,
            projection: 1.0,
        };
        let c = mal_blend(&a, &b, 0.5);
        assert!((c.projection - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let m = MalarEminenceMorph {
            left: 0.2,
            right: 0.3,
            projection: 0.4,
        };
        let m2 = m.clone();
        assert_eq!(m.projection, m2.projection);
    }
}
