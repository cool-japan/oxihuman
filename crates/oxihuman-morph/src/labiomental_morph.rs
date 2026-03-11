// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct LabiomentalMorph {
    pub fold_depth: f32,
    pub width: f32,
}

pub fn new_labiomental_morph() -> LabiomentalMorph {
    LabiomentalMorph {
        fold_depth: 0.0,
        width: 0.0,
    }
}

pub fn lm_set_fold_depth(m: &mut LabiomentalMorph, v: f32) {
    m.fold_depth = v.clamp(0.0, 1.0);
}

pub fn lm_set_width(m: &mut LabiomentalMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

pub fn lm_overall_weight(m: &LabiomentalMorph) -> f32 {
    (m.fold_depth + m.width) / 2.0
}

pub fn lm_blend(a: &LabiomentalMorph, b: &LabiomentalMorph, t: f32) -> LabiomentalMorph {
    let t = t.clamp(0.0, 1.0);
    LabiomentalMorph {
        fold_depth: a.fold_depth + (b.fold_depth - a.fold_depth) * t,
        width: a.width + (b.width - a.width) * t,
    }
}

pub fn lm_is_deep(m: &LabiomentalMorph) -> bool {
    m.fold_depth > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* zero init */
        let m = new_labiomental_morph();
        assert_eq!(m.fold_depth, 0.0);
        assert_eq!(m.width, 0.0);
    }

    #[test]
    fn test_set_fold_depth() {
        /* valid value */
        let mut m = new_labiomental_morph();
        lm_set_fold_depth(&mut m, 0.6);
        assert!((m.fold_depth - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_fold_depth_clamp_high() {
        /* clamp high */
        let mut m = new_labiomental_morph();
        lm_set_fold_depth(&mut m, 2.0);
        assert_eq!(m.fold_depth, 1.0);
    }

    #[test]
    fn test_set_width() {
        /* valid width */
        let mut m = new_labiomental_morph();
        lm_set_width(&mut m, 0.4);
        assert!((m.width - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamp_low() {
        /* clamp low */
        let mut m = new_labiomental_morph();
        lm_set_width(&mut m, -0.5);
        assert_eq!(m.width, 0.0);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = LabiomentalMorph {
            fold_depth: 0.4,
            width: 0.8,
        };
        assert!((lm_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_deep_false() {
        /* default not deep */
        let m = new_labiomental_morph();
        assert!(!lm_is_deep(&m));
    }

    #[test]
    fn test_is_deep_true() {
        /* above 0.5 */
        let m = LabiomentalMorph {
            fold_depth: 0.8,
            width: 0.0,
        };
        assert!(lm_is_deep(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint blend */
        let a = LabiomentalMorph {
            fold_depth: 0.0,
            width: 0.0,
        };
        let b = LabiomentalMorph {
            fold_depth: 1.0,
            width: 1.0,
        };
        let c = lm_blend(&a, &b, 0.5);
        assert!((c.fold_depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let m = LabiomentalMorph {
            fold_depth: 0.3,
            width: 0.7,
        };
        let m2 = m.clone();
        assert_eq!(m.width, m2.width);
    }
}
