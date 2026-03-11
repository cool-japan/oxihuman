// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct TempleFossaMorph {
    pub depth: f32,
    pub width: f32,
    pub temporal_muscle: f32,
}

pub fn new_temple_fossa_morph() -> TempleFossaMorph {
    TempleFossaMorph {
        depth: 0.0,
        width: 0.0,
        temporal_muscle: 0.0,
    }
}

pub fn tf_set_depth(m: &mut TempleFossaMorph, v: f32) {
    m.depth = v.clamp(0.0, 1.0);
}

pub fn tf_set_width(m: &mut TempleFossaMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

pub fn tf_overall_weight(m: &TempleFossaMorph) -> f32 {
    (m.depth + m.width + m.temporal_muscle) / 3.0
}

pub fn tf_blend(a: &TempleFossaMorph, b: &TempleFossaMorph, t: f32) -> TempleFossaMorph {
    let t = t.clamp(0.0, 1.0);
    TempleFossaMorph {
        depth: a.depth + (b.depth - a.depth) * t,
        width: a.width + (b.width - a.width) * t,
        temporal_muscle: a.temporal_muscle + (b.temporal_muscle - a.temporal_muscle) * t,
    }
}

pub fn tf_is_hollow(m: &TempleFossaMorph) -> bool {
    m.depth > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all zero */
        let m = new_temple_fossa_morph();
        assert_eq!(m.depth, 0.0);
    }

    #[test]
    fn test_set_depth() {
        /* stores valid value */
        let mut m = new_temple_fossa_morph();
        tf_set_depth(&mut m, 0.6);
        assert!((m.depth - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamp_high() {
        /* clamp above 1 */
        let mut m = new_temple_fossa_morph();
        tf_set_depth(&mut m, 2.0);
        assert_eq!(m.depth, 1.0);
    }

    #[test]
    fn test_set_width_clamp_low() {
        /* clamp below 0 */
        let mut m = new_temple_fossa_morph();
        tf_set_width(&mut m, -0.5);
        assert_eq!(m.width, 0.0);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = TempleFossaMorph {
            depth: 0.3,
            width: 0.6,
            temporal_muscle: 0.9,
        };
        assert!((tf_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_hollow_false() {
        /* default not hollow */
        let m = new_temple_fossa_morph();
        assert!(!tf_is_hollow(&m));
    }

    #[test]
    fn test_is_hollow_true() {
        /* depth > 0.5 */
        let m = TempleFossaMorph {
            depth: 0.9,
            width: 0.0,
            temporal_muscle: 0.0,
        };
        assert!(tf_is_hollow(&m));
    }

    #[test]
    fn test_blend() {
        /* t=0.5 */
        let a = TempleFossaMorph {
            depth: 0.0,
            width: 0.0,
            temporal_muscle: 0.0,
        };
        let b = TempleFossaMorph {
            depth: 1.0,
            width: 1.0,
            temporal_muscle: 1.0,
        };
        let c = tf_blend(&a, &b, 0.5);
        assert!((c.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let m = TempleFossaMorph {
            depth: 0.5,
            width: 0.4,
            temporal_muscle: 0.3,
        };
        let m2 = m.clone();
        assert_eq!(m.depth, m2.depth);
    }
}
