// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct NasalRootMorph {
    pub depth: f32,
    pub width: f32,
}

pub fn new_nasal_root_morph() -> NasalRootMorph {
    NasalRootMorph {
        depth: 0.0,
        width: 0.0,
    }
}

pub fn nr_set_depth(m: &mut NasalRootMorph, v: f32) {
    m.depth = v.clamp(0.0, 1.0);
}

pub fn nr_set_width(m: &mut NasalRootMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

pub fn nr_overall_weight(m: &NasalRootMorph) -> f32 {
    (m.depth + m.width) / 2.0
}

pub fn nr_blend(a: &NasalRootMorph, b: &NasalRootMorph, t: f32) -> NasalRootMorph {
    let t = t.clamp(0.0, 1.0);
    NasalRootMorph {
        depth: a.depth + (b.depth - a.depth) * t,
        width: a.width + (b.width - a.width) * t,
    }
}

pub fn nr_is_deep(m: &NasalRootMorph) -> bool {
    m.depth > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* zero init */
        let m = new_nasal_root_morph();
        assert_eq!(m.depth, 0.0);
        assert_eq!(m.width, 0.0);
    }

    #[test]
    fn test_set_depth() {
        /* valid depth */
        let mut m = new_nasal_root_morph();
        nr_set_depth(&mut m, 0.5);
        assert!((m.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamp_high() {
        /* clamp high */
        let mut m = new_nasal_root_morph();
        nr_set_depth(&mut m, 2.0);
        assert_eq!(m.depth, 1.0);
    }

    #[test]
    fn test_set_width() {
        /* valid width */
        let mut m = new_nasal_root_morph();
        nr_set_width(&mut m, 0.7);
        assert!((m.width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamp_low() {
        /* clamp low */
        let mut m = new_nasal_root_morph();
        nr_set_width(&mut m, -0.5);
        assert_eq!(m.width, 0.0);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = NasalRootMorph {
            depth: 0.4,
            width: 0.8,
        };
        assert!((nr_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_deep_false() {
        /* default not deep */
        let m = new_nasal_root_morph();
        assert!(!nr_is_deep(&m));
    }

    #[test]
    fn test_is_deep_true() {
        /* above 0.5 */
        let m = NasalRootMorph {
            depth: 0.9,
            width: 0.0,
        };
        assert!(nr_is_deep(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint blend */
        let a = NasalRootMorph {
            depth: 0.0,
            width: 0.0,
        };
        let b = NasalRootMorph {
            depth: 1.0,
            width: 1.0,
        };
        let c = nr_blend(&a, &b, 0.5);
        assert!((c.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let m = NasalRootMorph {
            depth: 0.3,
            width: 0.6,
        };
        let m2 = m.clone();
        assert_eq!(m.width, m2.width);
    }
}
