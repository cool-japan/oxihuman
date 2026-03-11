// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct MuscleDefinitionMorph {
    pub tone: f32,
    pub vascularity: f32,
    pub symmetry: f32,
}

pub fn new_muscle_definition_morph() -> MuscleDefinitionMorph {
    MuscleDefinitionMorph {
        tone: 0.0,
        vascularity: 0.0,
        symmetry: 1.0,
    }
}

pub fn muscle_def_set_tone(m: &mut MuscleDefinitionMorph, v: f32) {
    m.tone = v.clamp(0.0, 1.0);
}

pub fn muscle_def_is_athletic(m: &MuscleDefinitionMorph) -> bool {
    m.tone > 0.6
}

pub fn muscle_def_overall_weight(m: &MuscleDefinitionMorph) -> f32 {
    (m.tone + m.vascularity * 0.5) * 0.5 * m.symmetry
}

pub fn muscle_def_blend(
    a: &MuscleDefinitionMorph,
    b: &MuscleDefinitionMorph,
    t: f32,
) -> MuscleDefinitionMorph {
    let t = t.clamp(0.0, 1.0);
    MuscleDefinitionMorph {
        tone: a.tone + (b.tone - a.tone) * t,
        vascularity: a.vascularity + (b.vascularity - a.vascularity) * t,
        symmetry: a.symmetry + (b.symmetry - a.symmetry) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* default is untoned */
        let m = new_muscle_definition_morph();
        assert!((m.tone).abs() < 1e-5);
    }

    #[test]
    fn test_set_tone_clamped() {
        /* tone clamps */
        let mut m = new_muscle_definition_morph();
        muscle_def_set_tone(&mut m, 2.0);
        assert!((m.tone - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_is_athletic() {
        /* athletic threshold */
        let mut m = new_muscle_definition_morph();
        muscle_def_set_tone(&mut m, 0.8);
        assert!(muscle_def_is_athletic(&m));
    }

    #[test]
    fn test_overall_weight() {
        /* overall weight non-negative */
        let m = new_muscle_definition_morph();
        assert!(muscle_def_overall_weight(&m) >= 0.0);
    }

    #[test]
    fn test_blend() {
        /* blend midpoint */
        let a = new_muscle_definition_morph();
        let mut b = new_muscle_definition_morph();
        muscle_def_set_tone(&mut b, 1.0);
        let c = muscle_def_blend(&a, &b, 0.5);
        assert!((c.tone - 0.5).abs() < 1e-5);
    }
}
