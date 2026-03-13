// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct RamusMorph {
    pub height: f32,
    pub angle: f32,
    pub width: f32,
}

pub fn new_ramus_morph() -> RamusMorph {
    RamusMorph {
        height: 0.0,
        angle: 0.0,
        width: 0.0,
    }
}

pub fn rm_set_height(m: &mut RamusMorph, v: f32) {
    m.height = v.clamp(0.0, 1.0);
}

pub fn rm_set_angle(m: &mut RamusMorph, v: f32) {
    m.angle = v.clamp(0.0, 1.0);
}

pub fn rm_set_width(m: &mut RamusMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

pub fn rm_overall_weight(m: &RamusMorph) -> f32 {
    (m.height + m.angle + m.width) / 3.0
}

pub fn rm_blend(a: &RamusMorph, b: &RamusMorph, t: f32) -> RamusMorph {
    let t = t.clamp(0.0, 1.0);
    RamusMorph {
        height: a.height + (b.height - a.height) * t,
        angle: a.angle + (b.angle - a.angle) * t,
        width: a.width + (b.width - a.width) * t,
    }
}

pub fn rm_is_tall(m: &RamusMorph) -> bool {
    m.height > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* zero init */
        let m = new_ramus_morph();
        assert_eq!(m.height, 0.0);
        assert_eq!(m.angle, 0.0);
        assert_eq!(m.width, 0.0);
    }

    #[test]
    fn test_set_height() {
        /* valid height */
        let mut m = new_ramus_morph();
        rm_set_height(&mut m, 0.7);
        assert!((m.height - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_height_clamp_high() {
        /* clamp high */
        let mut m = new_ramus_morph();
        rm_set_height(&mut m, 3.0);
        assert_eq!(m.height, 1.0);
    }

    #[test]
    fn test_set_angle_clamp_low() {
        /* clamp low */
        let mut m = new_ramus_morph();
        rm_set_angle(&mut m, -1.0);
        assert_eq!(m.angle, 0.0);
    }

    #[test]
    fn test_set_width() {
        /* valid width */
        let mut m = new_ramus_morph();
        rm_set_width(&mut m, 0.5);
        assert!((m.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = RamusMorph {
            height: 0.3,
            angle: 0.6,
            width: 0.9,
        };
        assert!((rm_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_tall_false() {
        /* default not tall */
        let m = new_ramus_morph();
        assert!(!rm_is_tall(&m));
    }

    #[test]
    fn test_is_tall_true() {
        /* above 0.5 */
        let m = RamusMorph {
            height: 0.8,
            angle: 0.0,
            width: 0.0,
        };
        assert!(rm_is_tall(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint blend */
        let a = RamusMorph {
            height: 0.0,
            angle: 0.0,
            width: 0.0,
        };
        let b = RamusMorph {
            height: 1.0,
            angle: 1.0,
            width: 1.0,
        };
        let c = rm_blend(&a, &b, 0.5);
        assert!((c.height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let m = RamusMorph {
            height: 0.2,
            angle: 0.5,
            width: 0.8,
        };
        let m2 = m.clone();
        assert_eq!(m.width, m2.width);
    }
}
