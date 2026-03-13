// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct MandibleAngleMorph {
    pub gonial_angle: f32,
    pub masseter_prominence: f32,
    pub flare: f32,
}

pub fn new_mandible_angle_morph() -> MandibleAngleMorph {
    MandibleAngleMorph {
        gonial_angle: 0.0,
        masseter_prominence: 0.0,
        flare: 0.0,
    }
}

pub fn mand_set_gonial_angle(m: &mut MandibleAngleMorph, v: f32) {
    m.gonial_angle = v.clamp(0.0, 1.0);
}

pub fn mand_set_masseter(m: &mut MandibleAngleMorph, v: f32) {
    m.masseter_prominence = v.clamp(0.0, 1.0);
}

pub fn mand_overall_weight(m: &MandibleAngleMorph) -> f32 {
    (m.gonial_angle + m.masseter_prominence + m.flare) / 3.0
}

pub fn mand_blend(a: &MandibleAngleMorph, b: &MandibleAngleMorph, t: f32) -> MandibleAngleMorph {
    let t = t.clamp(0.0, 1.0);
    MandibleAngleMorph {
        gonial_angle: a.gonial_angle + (b.gonial_angle - a.gonial_angle) * t,
        masseter_prominence: a.masseter_prominence
            + (b.masseter_prominence - a.masseter_prominence) * t,
        flare: a.flare + (b.flare - a.flare) * t,
    }
}

pub fn mand_is_square(m: &MandibleAngleMorph) -> bool {
    m.flare > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all zero */
        let m = new_mandible_angle_morph();
        assert_eq!(m.gonial_angle, 0.0);
    }

    #[test]
    fn test_set_gonial_angle() {
        /* stores value */
        let mut m = new_mandible_angle_morph();
        mand_set_gonial_angle(&mut m, 0.5);
        assert!((m.gonial_angle - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_gonial_angle_clamp() {
        /* clamp high */
        let mut m = new_mandible_angle_morph();
        mand_set_gonial_angle(&mut m, 5.0);
        assert_eq!(m.gonial_angle, 1.0);
    }

    #[test]
    fn test_set_masseter() {
        /* valid value */
        let mut m = new_mandible_angle_morph();
        mand_set_masseter(&mut m, 0.4);
        assert!((m.masseter_prominence - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = MandibleAngleMorph {
            gonial_angle: 0.3,
            masseter_prominence: 0.6,
            flare: 0.9,
        };
        assert!((mand_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_square_false() {
        /* default not square */
        let m = new_mandible_angle_morph();
        assert!(!mand_is_square(&m));
    }

    #[test]
    fn test_is_square_true() {
        /* flare > 0.5 */
        let m = MandibleAngleMorph {
            gonial_angle: 0.0,
            masseter_prominence: 0.0,
            flare: 0.9,
        };
        assert!(mand_is_square(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint blend */
        let a = MandibleAngleMorph {
            gonial_angle: 0.0,
            masseter_prominence: 0.0,
            flare: 0.0,
        };
        let b = MandibleAngleMorph {
            gonial_angle: 1.0,
            masseter_prominence: 1.0,
            flare: 1.0,
        };
        let c = mand_blend(&a, &b, 0.5);
        assert!((c.flare - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let m = MandibleAngleMorph {
            gonial_angle: 0.2,
            masseter_prominence: 0.3,
            flare: 0.4,
        };
        let m2 = m.clone();
        assert_eq!(m.flare, m2.flare);
    }
}
