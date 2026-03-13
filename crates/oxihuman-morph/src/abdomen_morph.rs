// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct AbdomenMorph {
    pub protrusion: f32,
    pub muscle_definition: f32,
    pub navel_depth: f32,
}

pub fn new_abdomen_morph() -> AbdomenMorph {
    AbdomenMorph {
        protrusion: 0.2,
        muscle_definition: 0.0,
        navel_depth: 0.3,
    }
}

pub fn abdomen_set_protrusion(m: &mut AbdomenMorph, v: f32) {
    m.protrusion = v.clamp(0.0, 1.0);
}

pub fn abdomen_shows_abs(m: &AbdomenMorph) -> bool {
    m.muscle_definition > 0.5
}

pub fn abdomen_overall_weight(m: &AbdomenMorph) -> f32 {
    (m.protrusion + m.muscle_definition + m.navel_depth) / 3.0
}

pub fn abdomen_blend(a: &AbdomenMorph, b: &AbdomenMorph, t: f32) -> AbdomenMorph {
    let t = t.clamp(0.0, 1.0);
    AbdomenMorph {
        protrusion: a.protrusion + (b.protrusion - a.protrusion) * t,
        muscle_definition: a.muscle_definition + (b.muscle_definition - a.muscle_definition) * t,
        navel_depth: a.navel_depth + (b.navel_depth - a.navel_depth) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* protrusion > 0 by default */
        let m = new_abdomen_morph();
        assert!(m.protrusion > 0.0);
    }

    #[test]
    fn test_set_protrusion_clamp() {
        /* clamped */
        let mut m = new_abdomen_morph();
        abdomen_set_protrusion(&mut m, -1.0);
        assert!((m.protrusion).abs() < 1e-5);
    }

    #[test]
    fn test_shows_abs_false_by_default() {
        /* new morph doesn't show abs */
        let m = new_abdomen_morph();
        assert!(!abdomen_shows_abs(&m));
    }

    #[test]
    fn test_overall_weight_range() {
        /* in range */
        let m = new_abdomen_morph();
        let w = abdomen_overall_weight(&m);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend_midpoint() {
        /* blend at 0.5 */
        let a = AbdomenMorph {
            protrusion: 0.0,
            muscle_definition: 0.0,
            navel_depth: 0.0,
        };
        let b = AbdomenMorph {
            protrusion: 1.0,
            muscle_definition: 1.0,
            navel_depth: 1.0,
        };
        let c = abdomen_blend(&a, &b, 0.5);
        assert!((c.protrusion - 0.5).abs() < 1e-5);
    }
}
