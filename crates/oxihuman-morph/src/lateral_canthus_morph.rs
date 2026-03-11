// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct LateralCanthusMorph {
    pub tilt_deg: f32,
    pub sharpness: f32,
    pub position_y: f32,
}

pub fn new_lateral_canthus_morph() -> LateralCanthusMorph {
    LateralCanthusMorph {
        tilt_deg: 0.0,
        sharpness: 0.0,
        position_y: 0.0,
    }
}

pub fn lateral_canthus_set_tilt(m: &mut LateralCanthusMorph, v: f32) {
    m.tilt_deg = v;
}

pub fn lateral_canthus_is_upturned(m: &LateralCanthusMorph) -> bool {
    m.tilt_deg > 0.0
}

pub fn lateral_canthus_overall_weight(m: &LateralCanthusMorph) -> f32 {
    (m.tilt_deg.abs() / 45.0 + m.sharpness.abs() + m.position_y.abs()) / 3.0
}

pub fn lateral_canthus_blend(
    a: &LateralCanthusMorph,
    b: &LateralCanthusMorph,
    t: f32,
) -> LateralCanthusMorph {
    let t = t.clamp(0.0, 1.0);
    LateralCanthusMorph {
        tilt_deg: a.tilt_deg + (b.tilt_deg - a.tilt_deg) * t,
        sharpness: a.sharpness + (b.sharpness - a.sharpness) * t,
        position_y: a.position_y + (b.position_y - a.position_y) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_lateral_canthus_morph() {
        /* default tilt is 0 */
        let m = new_lateral_canthus_morph();
        assert_eq!(m.tilt_deg, 0.0);
    }

    #[test]
    fn test_lateral_canthus_set_tilt() {
        /* tilt is set */
        let mut m = new_lateral_canthus_morph();
        lateral_canthus_set_tilt(&mut m, 10.0);
        assert!((m.tilt_deg - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_lateral_canthus_is_upturned_false() {
        /* zero tilt is not upturned */
        let m = new_lateral_canthus_morph();
        assert!(!lateral_canthus_is_upturned(&m));
    }

    #[test]
    fn test_lateral_canthus_is_upturned_true() {
        /* positive tilt is upturned */
        let mut m = new_lateral_canthus_morph();
        m.tilt_deg = 5.0;
        assert!(lateral_canthus_is_upturned(&m));
    }

    #[test]
    fn test_lateral_canthus_blend() {
        /* blend at 0.5 is midpoint */
        let a = new_lateral_canthus_morph();
        let b = LateralCanthusMorph {
            tilt_deg: 20.0,
            sharpness: 1.0,
            position_y: 1.0,
        };
        let r = lateral_canthus_blend(&a, &b, 0.5);
        assert!((r.tilt_deg - 10.0).abs() < 1e-6);
    }
}
