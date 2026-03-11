// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct SubcutaneousFatMorphNew {
    pub torso: f32,
    pub limbs: f32,
    pub face: f32,
}

pub fn new_subcut_fat_morph() -> SubcutaneousFatMorphNew {
    SubcutaneousFatMorphNew {
        torso: 0.0,
        limbs: 0.0,
        face: 0.0,
    }
}

pub fn subcut_set_torso(m: &mut SubcutaneousFatMorphNew, v: f32) {
    m.torso = v.clamp(0.0, 1.0);
}

pub fn subcut_is_uniform(m: &SubcutaneousFatMorphNew) -> bool {
    let avg = (m.torso + m.limbs + m.face) / 3.0;
    let dev = (m.torso - avg)
        .abs()
        .max((m.limbs - avg).abs())
        .max((m.face - avg).abs());
    dev < 0.15
}

pub fn subcut_overall_weight(m: &SubcutaneousFatMorphNew) -> f32 {
    (m.torso + m.limbs + m.face) / 3.0
}

pub fn subcut_blend(
    a: &SubcutaneousFatMorphNew,
    b: &SubcutaneousFatMorphNew,
    t: f32,
) -> SubcutaneousFatMorphNew {
    let t = t.clamp(0.0, 1.0);
    SubcutaneousFatMorphNew {
        torso: a.torso + (b.torso - a.torso) * t,
        limbs: a.limbs + (b.limbs - a.limbs) * t,
        face: a.face + (b.face - a.face) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* default all zero */
        let m = new_subcut_fat_morph();
        assert!((m.torso + m.limbs + m.face).abs() < 1e-5);
    }

    #[test]
    fn test_set_torso() {
        /* set torso clamps */
        let mut m = new_subcut_fat_morph();
        subcut_set_torso(&mut m, 0.7);
        assert!((m.torso - 0.7).abs() < 1e-5);
    }

    #[test]
    fn test_is_uniform_when_equal() {
        /* equal values are uniform */
        let m = SubcutaneousFatMorphNew {
            torso: 0.5,
            limbs: 0.5,
            face: 0.5,
        };
        assert!(subcut_is_uniform(&m));
    }

    #[test]
    fn test_overall_weight() {
        /* average of three fields */
        let m = SubcutaneousFatMorphNew {
            torso: 0.6,
            limbs: 0.3,
            face: 0.3,
        };
        let w = subcut_overall_weight(&m);
        assert!((w - 0.4).abs() < 1e-5);
    }

    #[test]
    fn test_blend() {
        /* blend interpolates linearly */
        let a = new_subcut_fat_morph();
        let b = SubcutaneousFatMorphNew {
            torso: 1.0,
            limbs: 1.0,
            face: 1.0,
        };
        let c = subcut_blend(&a, &b, 0.5);
        assert!((c.torso - 0.5).abs() < 1e-5);
    }
}
