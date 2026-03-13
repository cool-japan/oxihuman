// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct TongueShapeMorph {
    pub width: f32,
    pub thickness: f32,
    pub length: f32,
    pub tip_shape: f32,
}

pub fn new_tongue_shape_morph() -> TongueShapeMorph {
    TongueShapeMorph {
        width: 0.0,
        thickness: 0.0,
        length: 0.0,
        tip_shape: 0.0,
    }
}

pub fn tongue_set_width(m: &mut TongueShapeMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

pub fn tongue_is_wide(m: &TongueShapeMorph) -> bool {
    m.width > 0.5
}

pub fn tongue_overall_weight(m: &TongueShapeMorph) -> f32 {
    (m.width.abs() + m.thickness.abs() + m.length.abs() + m.tip_shape.abs()) * 0.25
}

pub fn tongue_blend(a: &TongueShapeMorph, b: &TongueShapeMorph, t: f32) -> TongueShapeMorph {
    let t = t.clamp(0.0, 1.0);
    TongueShapeMorph {
        width: a.width + (b.width - a.width) * t,
        thickness: a.thickness + (b.thickness - a.thickness) * t,
        length: a.length + (b.length - a.length) * t,
        tip_shape: a.tip_shape + (b.tip_shape - a.tip_shape) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tongue_shape_morph() {
        /* all defaults should be zero */
        let m = new_tongue_shape_morph();
        assert_eq!(m.width, 0.0);
    }

    #[test]
    fn test_tongue_set_width() {
        /* width is clamped between 0 and 1 */
        let mut m = new_tongue_shape_morph();
        tongue_set_width(&mut m, 2.0);
        assert_eq!(m.width, 1.0);
    }

    #[test]
    fn test_tongue_is_wide() {
        /* wide when width > 0.5 */
        let mut m = new_tongue_shape_morph();
        m.width = 0.8;
        assert!(tongue_is_wide(&m));
    }

    #[test]
    fn test_tongue_overall_weight_zero() {
        /* zero fields => zero weight */
        let m = new_tongue_shape_morph();
        assert_eq!(tongue_overall_weight(&m), 0.0);
    }

    #[test]
    fn test_tongue_blend_midpoint() {
        /* blend at 0.5 gives midpoint */
        let a = new_tongue_shape_morph();
        let b = TongueShapeMorph {
            width: 1.0,
            thickness: 1.0,
            length: 1.0,
            tip_shape: 1.0,
        };
        let r = tongue_blend(&a, &b, 0.5);
        assert!((r.width - 0.5).abs() < 1e-6);
    }
}
