// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct CondyleMorph {
    pub offset_x: f32,
    pub offset_y: f32,
    pub size: f32,
}

pub fn new_condyle_morph() -> CondyleMorph {
    CondyleMorph {
        offset_x: 0.0,
        offset_y: 0.0,
        size: 0.0,
    }
}

pub fn cy_set_offset_x(m: &mut CondyleMorph, v: f32) {
    m.offset_x = v.clamp(-1.0, 1.0);
}

pub fn cy_set_offset_y(m: &mut CondyleMorph, v: f32) {
    m.offset_y = v.clamp(-1.0, 1.0);
}

pub fn cy_set_size(m: &mut CondyleMorph, v: f32) {
    m.size = v.clamp(0.0, 1.0);
}

pub fn cy_overall_weight(m: &CondyleMorph) -> f32 {
    (m.offset_x.abs() + m.offset_y.abs() + m.size) / 3.0
}

pub fn cy_blend(a: &CondyleMorph, b: &CondyleMorph, t: f32) -> CondyleMorph {
    let t = t.clamp(0.0, 1.0);
    CondyleMorph {
        offset_x: a.offset_x + (b.offset_x - a.offset_x) * t,
        offset_y: a.offset_y + (b.offset_y - a.offset_y) * t,
        size: a.size + (b.size - a.size) * t,
    }
}

pub fn cy_is_displaced(m: &CondyleMorph) -> bool {
    m.offset_x.abs() > 0.3 || m.offset_y.abs() > 0.3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* zero init */
        let m = new_condyle_morph();
        assert_eq!(m.offset_x, 0.0);
        assert_eq!(m.offset_y, 0.0);
        assert_eq!(m.size, 0.0);
    }

    #[test]
    fn test_set_offset_x() {
        /* valid offset */
        let mut m = new_condyle_morph();
        cy_set_offset_x(&mut m, 0.5);
        assert!((m.offset_x - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_offset_x_clamp_high() {
        /* clamp high */
        let mut m = new_condyle_morph();
        cy_set_offset_x(&mut m, 5.0);
        assert_eq!(m.offset_x, 1.0);
    }

    #[test]
    fn test_set_offset_y_clamp_low() {
        /* clamp low */
        let mut m = new_condyle_morph();
        cy_set_offset_y(&mut m, -5.0);
        assert_eq!(m.offset_y, -1.0);
    }

    #[test]
    fn test_set_size() {
        /* valid size */
        let mut m = new_condyle_morph();
        cy_set_size(&mut m, 0.6);
        assert!((m.size - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_overall_weight() {
        /* average of abs offsets and size */
        let m = CondyleMorph {
            offset_x: 0.3,
            offset_y: 0.6,
            size: 0.9,
        };
        assert!((cy_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_displaced_false() {
        /* default not displaced */
        let m = new_condyle_morph();
        assert!(!cy_is_displaced(&m));
    }

    #[test]
    fn test_is_displaced_true() {
        /* offset above threshold */
        let m = CondyleMorph {
            offset_x: 0.5,
            offset_y: 0.0,
            size: 0.0,
        };
        assert!(cy_is_displaced(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint blend */
        let a = CondyleMorph {
            offset_x: -1.0,
            offset_y: -1.0,
            size: 0.0,
        };
        let b = CondyleMorph {
            offset_x: 1.0,
            offset_y: 1.0,
            size: 1.0,
        };
        let c = cy_blend(&a, &b, 0.5);
        assert!((c.offset_x - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let m = CondyleMorph {
            offset_x: 0.2,
            offset_y: 0.5,
            size: 0.8,
        };
        let m2 = m.clone();
        assert_eq!(m.size, m2.size);
    }
}
