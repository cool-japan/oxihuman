// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct HipShapeMorph {
    pub width: f32,
    pub roundness: f32,
    pub height: f32,
}

pub fn new_hip_shape_morph() -> HipShapeMorph {
    HipShapeMorph {
        width: 0.4,
        roundness: 0.5,
        height: 0.5,
    }
}

pub fn hip_set_width(m: &mut HipShapeMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

pub fn hip_is_wide(m: &HipShapeMorph) -> bool {
    m.width > 0.6
}

pub fn hip_overall_weight(m: &HipShapeMorph) -> f32 {
    (m.width + m.roundness + m.height) / 3.0
}

pub fn hip_blend(a: &HipShapeMorph, b: &HipShapeMorph, t: f32) -> HipShapeMorph {
    let t = t.clamp(0.0, 1.0);
    HipShapeMorph {
        width: a.width + (b.width - a.width) * t,
        roundness: a.roundness + (b.roundness - a.roundness) * t,
        height: a.height + (b.height - a.height) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* default width set */
        let m = new_hip_shape_morph();
        assert!(m.width > 0.0);
    }

    #[test]
    fn test_set_width() {
        /* width clamped */
        let mut m = new_hip_shape_morph();
        hip_set_width(&mut m, 0.8);
        assert!((m.width - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_is_wide() {
        /* wide threshold */
        let mut m = new_hip_shape_morph();
        hip_set_width(&mut m, 0.7);
        assert!(hip_is_wide(&m));
    }

    #[test]
    fn test_overall_weight() {
        /* weight bounded */
        let m = new_hip_shape_morph();
        let w = hip_overall_weight(&m);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend() {
        /* blend midpoint */
        let a = HipShapeMorph {
            width: 0.0,
            roundness: 0.0,
            height: 0.0,
        };
        let b = HipShapeMorph {
            width: 1.0,
            roundness: 1.0,
            height: 1.0,
        };
        let c = hip_blend(&a, &b, 0.5);
        assert!((c.width - 0.5).abs() < 1e-5);
    }
}
