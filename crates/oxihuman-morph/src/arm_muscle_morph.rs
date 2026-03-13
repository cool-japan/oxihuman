// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct ArmMuscleMorph {
    pub bicep: f32,
    pub tricep: f32,
    pub forearm: f32,
    pub definition: f32,
}

pub fn new_arm_muscle_morph() -> ArmMuscleMorph {
    ArmMuscleMorph {
        bicep: 0.0,
        tricep: 0.0,
        forearm: 0.0,
        definition: 0.0,
    }
}

pub fn arm_set_bicep(m: &mut ArmMuscleMorph, v: f32) {
    m.bicep = v.clamp(0.0, 1.0);
}

pub fn arm_is_muscular(m: &ArmMuscleMorph) -> bool {
    (m.bicep + m.tricep) * 0.5 > 0.5
}

pub fn arm_overall_weight(m: &ArmMuscleMorph) -> f32 {
    (m.bicep + m.tricep + m.forearm + m.definition) / 4.0
}

pub fn arm_blend(a: &ArmMuscleMorph, b: &ArmMuscleMorph, t: f32) -> ArmMuscleMorph {
    let t = t.clamp(0.0, 1.0);
    ArmMuscleMorph {
        bicep: a.bicep + (b.bicep - a.bicep) * t,
        tricep: a.tricep + (b.tricep - a.tricep) * t,
        forearm: a.forearm + (b.forearm - a.forearm) * t,
        definition: a.definition + (b.definition - a.definition) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_zero() {
        /* all zero */
        let m = new_arm_muscle_morph();
        assert!((m.bicep + m.tricep).abs() < 1e-5);
    }

    #[test]
    fn test_set_bicep_clamped() {
        /* bicep clamped */
        let mut m = new_arm_muscle_morph();
        arm_set_bicep(&mut m, 0.8);
        assert!((m.bicep - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_not_muscular_by_default() {
        /* not muscular */
        let m = new_arm_muscle_morph();
        assert!(!arm_is_muscular(&m));
    }

    #[test]
    fn test_overall_weight_zero() {
        /* zero */
        let m = new_arm_muscle_morph();
        assert!((arm_overall_weight(&m)).abs() < 1e-5);
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = new_arm_muscle_morph();
        let b = ArmMuscleMorph {
            bicep: 1.0,
            tricep: 1.0,
            forearm: 1.0,
            definition: 1.0,
        };
        let c = arm_blend(&a, &b, 0.5);
        assert!((c.bicep - 0.5).abs() < 1e-5);
    }
}
