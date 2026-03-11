// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct PogonionMorph {
    pub protrusion: f32,
    pub has_cleft: bool,
}

pub fn new_pogonion_morph() -> PogonionMorph {
    PogonionMorph {
        protrusion: 0.0,
        has_cleft: false,
    }
}

pub fn pogonion_set_protrusion(m: &mut PogonionMorph, v: f32) {
    m.protrusion = v.clamp(0.0, 1.0);
}

pub fn pogonion_has_cleft(m: &PogonionMorph) -> bool {
    m.has_cleft
}

pub fn pogonion_overall_weight(m: &PogonionMorph) -> f32 {
    m.protrusion
}

pub fn pogonion_blend(a: &PogonionMorph, b: &PogonionMorph, t: f32) -> PogonionMorph {
    let t = t.clamp(0.0, 1.0);
    PogonionMorph {
        protrusion: a.protrusion + (b.protrusion - a.protrusion) * t,
        has_cleft: if t < 0.5 { a.has_cleft } else { b.has_cleft },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* zero protrusion, no cleft */
        let m = new_pogonion_morph();
        assert_eq!(m.protrusion, 0.0);
        assert!(!m.has_cleft);
    }

    #[test]
    fn test_set_protrusion() {
        /* valid protrusion */
        let mut m = new_pogonion_morph();
        pogonion_set_protrusion(&mut m, 0.7);
        assert!((m.protrusion - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_protrusion_clamp_high() {
        /* clamp high */
        let mut m = new_pogonion_morph();
        pogonion_set_protrusion(&mut m, 3.0);
        assert_eq!(m.protrusion, 1.0);
    }

    #[test]
    fn test_set_protrusion_clamp_low() {
        /* clamp low */
        let mut m = new_pogonion_morph();
        pogonion_set_protrusion(&mut m, -1.0);
        assert_eq!(m.protrusion, 0.0);
    }

    #[test]
    fn test_has_cleft_false() {
        /* default no cleft */
        let m = new_pogonion_morph();
        assert!(!pogonion_has_cleft(&m));
    }

    #[test]
    fn test_has_cleft_true() {
        /* set cleft */
        let m = PogonionMorph {
            protrusion: 0.5,
            has_cleft: true,
        };
        assert!(pogonion_has_cleft(&m));
    }

    #[test]
    fn test_overall_weight() {
        /* weight = protrusion */
        let m = PogonionMorph {
            protrusion: 0.7,
            has_cleft: false,
        };
        assert!((pogonion_overall_weight(&m) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_blend() {
        /* midpoint blend */
        let a = PogonionMorph {
            protrusion: 0.0,
            has_cleft: false,
        };
        let b = PogonionMorph {
            protrusion: 1.0,
            has_cleft: false,
        };
        let c = pogonion_blend(&a, &b, 0.5);
        assert!((c.protrusion - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let m = PogonionMorph {
            protrusion: 0.3,
            has_cleft: true,
        };
        let m2 = m.clone();
        assert_eq!(m.has_cleft, m2.has_cleft);
    }
}
