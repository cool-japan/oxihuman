// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct GenialTubercleMorph {
    pub prominence: f32,
    pub width: f32,
    pub vertical_pos: f32,
}

pub fn new_genial_tubercle_morph() -> GenialTubercleMorph {
    GenialTubercleMorph {
        prominence: 0.0,
        width: 0.0,
        vertical_pos: 0.0,
    }
}

pub fn gt_set_prominence(m: &mut GenialTubercleMorph, v: f32) {
    m.prominence = v.clamp(0.0, 1.0);
}

pub fn gt_set_width(m: &mut GenialTubercleMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

pub fn gt_overall_weight(m: &GenialTubercleMorph) -> f32 {
    (m.prominence + m.width + m.vertical_pos) / 3.0
}

pub fn gt_blend(a: &GenialTubercleMorph, b: &GenialTubercleMorph, t: f32) -> GenialTubercleMorph {
    let t = t.clamp(0.0, 1.0);
    GenialTubercleMorph {
        prominence: a.prominence + (b.prominence - a.prominence) * t,
        width: a.width + (b.width - a.width) * t,
        vertical_pos: a.vertical_pos + (b.vertical_pos - a.vertical_pos) * t,
    }
}

pub fn gt_is_prominent(m: &GenialTubercleMorph) -> bool {
    m.prominence > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all zero */
        let m = new_genial_tubercle_morph();
        assert_eq!(m.prominence, 0.0);
    }

    #[test]
    fn test_set_prominence() {
        /* valid value stored */
        let mut m = new_genial_tubercle_morph();
        gt_set_prominence(&mut m, 0.7);
        assert!((m.prominence - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_prominence_clamp() {
        /* clamp above 1 */
        let mut m = new_genial_tubercle_morph();
        gt_set_prominence(&mut m, 9.0);
        assert_eq!(m.prominence, 1.0);
    }

    #[test]
    fn test_set_width_clamp() {
        /* clamp below 0 */
        let mut m = new_genial_tubercle_morph();
        gt_set_width(&mut m, -0.1);
        assert_eq!(m.width, 0.0);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = GenialTubercleMorph {
            prominence: 0.3,
            width: 0.6,
            vertical_pos: 0.9,
        };
        assert!((gt_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_prominent_false() {
        /* below threshold */
        let m = new_genial_tubercle_morph();
        assert!(!gt_is_prominent(&m));
    }

    #[test]
    fn test_is_prominent_true() {
        /* above threshold */
        let m = GenialTubercleMorph {
            prominence: 0.8,
            width: 0.0,
            vertical_pos: 0.0,
        };
        assert!(gt_is_prominent(&m));
    }

    #[test]
    fn test_blend_midpoint() {
        /* t=0.5 */
        let a = GenialTubercleMorph {
            prominence: 0.0,
            width: 0.0,
            vertical_pos: 0.0,
        };
        let b = GenialTubercleMorph {
            prominence: 1.0,
            width: 1.0,
            vertical_pos: 1.0,
        };
        let c = gt_blend(&a, &b, 0.5);
        assert!((c.prominence - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let m = GenialTubercleMorph {
            prominence: 0.5,
            width: 0.4,
            vertical_pos: 0.3,
        };
        let m2 = m.clone();
        assert_eq!(m.prominence, m2.prominence);
    }
}
