// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct LegMuscleMorph {
    pub quadricep: f32,
    pub hamstring: f32,
    pub calf: f32,
    pub definition: f32,
}

pub fn new_leg_muscle_morph() -> LegMuscleMorph {
    LegMuscleMorph {
        quadricep: 0.0,
        hamstring: 0.0,
        calf: 0.0,
        definition: 0.0,
    }
}

pub fn leg_set_quad(m: &mut LegMuscleMorph, v: f32) {
    m.quadricep = v.clamp(0.0, 1.0);
}

pub fn leg_is_muscular(m: &LegMuscleMorph) -> bool {
    (m.quadricep + m.hamstring + m.calf) / 3.0 > 0.5
}

pub fn leg_overall_weight(m: &LegMuscleMorph) -> f32 {
    (m.quadricep + m.hamstring + m.calf + m.definition) / 4.0
}

pub fn leg_blend(a: &LegMuscleMorph, b: &LegMuscleMorph, t: f32) -> LegMuscleMorph {
    let t = t.clamp(0.0, 1.0);
    LegMuscleMorph {
        quadricep: a.quadricep + (b.quadricep - a.quadricep) * t,
        hamstring: a.hamstring + (b.hamstring - a.hamstring) * t,
        calf: a.calf + (b.calf - a.calf) * t,
        definition: a.definition + (b.definition - a.definition) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_zero() {
        /* all zero */
        let m = new_leg_muscle_morph();
        assert!((m.quadricep).abs() < 1e-5);
    }

    #[test]
    fn test_set_quad_clamp() {
        /* quad clamped */
        let mut m = new_leg_muscle_morph();
        leg_set_quad(&mut m, 0.7);
        assert!((m.quadricep - 0.7).abs() < 1e-5);
    }

    #[test]
    fn test_not_muscular_by_default() {
        /* not muscular */
        let m = new_leg_muscle_morph();
        assert!(!leg_is_muscular(&m));
    }

    #[test]
    fn test_overall_weight_zero() {
        /* zero */
        let m = new_leg_muscle_morph();
        assert!((leg_overall_weight(&m)).abs() < 1e-5);
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = new_leg_muscle_morph();
        let b = LegMuscleMorph {
            quadricep: 1.0,
            hamstring: 1.0,
            calf: 1.0,
            definition: 1.0,
        };
        let c = leg_blend(&a, &b, 0.5);
        assert!((c.quadricep - 0.5).abs() < 1e-5);
    }
}
