// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct ChestMuscleMorph {
    pub pectoralis_major: f32,
    pub separation: f32,
    pub upper_chest: f32,
}

pub fn new_chest_muscle_morph() -> ChestMuscleMorph {
    ChestMuscleMorph {
        pectoralis_major: 0.0,
        separation: 0.0,
        upper_chest: 0.0,
    }
}

pub fn chest_set_pec(m: &mut ChestMuscleMorph, v: f32) {
    m.pectoralis_major = v.clamp(0.0, 1.0);
}

pub fn chest_is_muscular(m: &ChestMuscleMorph) -> bool {
    m.pectoralis_major > 0.5
}

pub fn chest_overall_weight(m: &ChestMuscleMorph) -> f32 {
    (m.pectoralis_major + m.separation + m.upper_chest) / 3.0
}

pub fn chest_blend(a: &ChestMuscleMorph, b: &ChestMuscleMorph, t: f32) -> ChestMuscleMorph {
    let t = t.clamp(0.0, 1.0);
    ChestMuscleMorph {
        pectoralis_major: a.pectoralis_major + (b.pectoralis_major - a.pectoralis_major) * t,
        separation: a.separation + (b.separation - a.separation) * t,
        upper_chest: a.upper_chest + (b.upper_chest - a.upper_chest) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_zero() {
        /* all zero */
        let m = new_chest_muscle_morph();
        assert!((m.pectoralis_major).abs() < 1e-5);
    }

    #[test]
    fn test_set_pec_clamped() {
        /* clamped */
        let mut m = new_chest_muscle_morph();
        chest_set_pec(&mut m, 1.5);
        assert!((m.pectoralis_major - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_not_muscular_by_default() {
        /* not muscular */
        let m = new_chest_muscle_morph();
        assert!(!chest_is_muscular(&m));
    }

    #[test]
    fn test_overall_weight_zero() {
        /* zero weight */
        let m = new_chest_muscle_morph();
        assert!((chest_overall_weight(&m)).abs() < 1e-5);
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = new_chest_muscle_morph();
        let b = ChestMuscleMorph {
            pectoralis_major: 1.0,
            separation: 1.0,
            upper_chest: 1.0,
        };
        let c = chest_blend(&a, &b, 0.5);
        assert!((c.pectoralis_major - 0.5).abs() < 1e-5);
    }
}
