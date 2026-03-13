// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct CanthalTiltMorph {
    pub outer_tilt_deg: f32,
    pub inner_tilt_deg: f32,
}

pub fn new_canthal_tilt_morph() -> CanthalTiltMorph {
    CanthalTiltMorph {
        outer_tilt_deg: 0.0,
        inner_tilt_deg: 0.0,
    }
}

pub fn canthal_set_outer_tilt(m: &mut CanthalTiltMorph, v: f32) {
    m.outer_tilt_deg = v;
}

pub fn canthal_is_upswept(m: &CanthalTiltMorph) -> bool {
    m.outer_tilt_deg > 0.0
}

pub fn canthal_overall_weight(m: &CanthalTiltMorph) -> f32 {
    (m.outer_tilt_deg.abs() + m.inner_tilt_deg.abs()) * 0.5 / 45.0
}

pub fn canthal_blend(a: &CanthalTiltMorph, b: &CanthalTiltMorph, t: f32) -> CanthalTiltMorph {
    let t = t.clamp(0.0, 1.0);
    CanthalTiltMorph {
        outer_tilt_deg: a.outer_tilt_deg + (b.outer_tilt_deg - a.outer_tilt_deg) * t,
        inner_tilt_deg: a.inner_tilt_deg + (b.inner_tilt_deg - a.inner_tilt_deg) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_canthal_tilt_morph() {
        /* default is zero */
        let m = new_canthal_tilt_morph();
        assert_eq!(m.outer_tilt_deg, 0.0);
    }

    #[test]
    fn test_canthal_set_outer_tilt() {
        /* outer tilt is set */
        let mut m = new_canthal_tilt_morph();
        canthal_set_outer_tilt(&mut m, 15.0);
        assert!((m.outer_tilt_deg - 15.0).abs() < 1e-6);
    }

    #[test]
    fn test_canthal_is_upswept_false() {
        /* zero tilt is not upswept */
        let m = new_canthal_tilt_morph();
        assert!(!canthal_is_upswept(&m));
    }

    #[test]
    fn test_canthal_is_upswept_true() {
        /* positive outer tilt is upswept */
        let mut m = new_canthal_tilt_morph();
        m.outer_tilt_deg = 5.0;
        assert!(canthal_is_upswept(&m));
    }

    #[test]
    fn test_canthal_blend() {
        /* midpoint blend */
        let a = new_canthal_tilt_morph();
        let b = CanthalTiltMorph {
            outer_tilt_deg: 10.0,
            inner_tilt_deg: 10.0,
        };
        let r = canthal_blend(&a, &b, 0.5);
        assert!((r.outer_tilt_deg - 5.0).abs() < 1e-6);
    }
}
