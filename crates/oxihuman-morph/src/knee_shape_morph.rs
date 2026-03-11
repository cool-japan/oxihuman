// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct KneeShapeMorph {
    pub prominence: f32,
    pub width: f32,
    pub valgus_angle_deg: f32,
}

pub fn new_knee_shape_morph() -> KneeShapeMorph {
    KneeShapeMorph {
        prominence: 0.3,
        width: 0.4,
        valgus_angle_deg: 0.0,
    }
}

pub fn knee_set_prominence(m: &mut KneeShapeMorph, v: f32) {
    m.prominence = v.clamp(0.0, 1.0);
}

pub fn knee_is_valgus(m: &KneeShapeMorph) -> bool {
    m.valgus_angle_deg > 5.0
}

pub fn knee_overall_weight(m: &KneeShapeMorph) -> f32 {
    (m.prominence + m.width) * 0.5
}

pub fn knee_blend(a: &KneeShapeMorph, b: &KneeShapeMorph, t: f32) -> KneeShapeMorph {
    let t = t.clamp(0.0, 1.0);
    KneeShapeMorph {
        prominence: a.prominence + (b.prominence - a.prominence) * t,
        width: a.width + (b.width - a.width) * t,
        valgus_angle_deg: a.valgus_angle_deg + (b.valgus_angle_deg - a.valgus_angle_deg) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* prominence > 0 */
        let m = new_knee_shape_morph();
        assert!(m.prominence > 0.0);
    }

    #[test]
    fn test_set_prominence() {
        /* clamped */
        let mut m = new_knee_shape_morph();
        knee_set_prominence(&mut m, 0.9);
        assert!((m.prominence - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_not_valgus_by_default() {
        /* default is not valgus */
        let m = new_knee_shape_morph();
        assert!(!knee_is_valgus(&m));
    }

    #[test]
    fn test_is_valgus() {
        /* valgus angle > 5 degrees */
        let m = KneeShapeMorph {
            prominence: 0.3,
            width: 0.4,
            valgus_angle_deg: 10.0,
        };
        assert!(knee_is_valgus(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = KneeShapeMorph {
            prominence: 0.0,
            width: 0.0,
            valgus_angle_deg: 0.0,
        };
        let b = KneeShapeMorph {
            prominence: 1.0,
            width: 1.0,
            valgus_angle_deg: 10.0,
        };
        let c = knee_blend(&a, &b, 0.5);
        assert!((c.prominence - 0.5).abs() < 1e-5);
        assert!((c.valgus_angle_deg - 5.0).abs() < 1e-4);
    }
}
