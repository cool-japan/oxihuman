// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct PhiltrumRidgeMorph {
    pub definition: f32,
    pub width: f32,
    pub length: f32,
}

pub fn new_philtrum_ridge_morph() -> PhiltrumRidgeMorph {
    PhiltrumRidgeMorph {
        definition: 0.0,
        width: 0.0,
        length: 0.0,
    }
}

pub fn pr_set_definition(m: &mut PhiltrumRidgeMorph, v: f32) {
    m.definition = v.clamp(0.0, 1.0);
}

pub fn pr_set_width(m: &mut PhiltrumRidgeMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

pub fn pr_set_length(m: &mut PhiltrumRidgeMorph, v: f32) {
    m.length = v.clamp(0.0, 1.0);
}

pub fn pr_overall_weight(m: &PhiltrumRidgeMorph) -> f32 {
    (m.definition + m.width + m.length) / 3.0
}

pub fn pr_blend(a: &PhiltrumRidgeMorph, b: &PhiltrumRidgeMorph, t: f32) -> PhiltrumRidgeMorph {
    let t = t.clamp(0.0, 1.0);
    PhiltrumRidgeMorph {
        definition: a.definition + (b.definition - a.definition) * t,
        width: a.width + (b.width - a.width) * t,
        length: a.length + (b.length - a.length) * t,
    }
}

pub fn pr_is_defined(m: &PhiltrumRidgeMorph) -> bool {
    m.definition > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* zero init */
        let m = new_philtrum_ridge_morph();
        assert_eq!(m.definition, 0.0);
        assert_eq!(m.width, 0.0);
        assert_eq!(m.length, 0.0);
    }

    #[test]
    fn test_set_definition() {
        /* valid value */
        let mut m = new_philtrum_ridge_morph();
        pr_set_definition(&mut m, 0.7);
        assert!((m.definition - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_definition_clamp_high() {
        /* clamp high */
        let mut m = new_philtrum_ridge_morph();
        pr_set_definition(&mut m, 3.0);
        assert_eq!(m.definition, 1.0);
    }

    #[test]
    fn test_set_width_clamp_low() {
        /* clamp low */
        let mut m = new_philtrum_ridge_morph();
        pr_set_width(&mut m, -0.5);
        assert_eq!(m.width, 0.0);
    }

    #[test]
    fn test_set_length() {
        /* valid length */
        let mut m = new_philtrum_ridge_morph();
        pr_set_length(&mut m, 0.5);
        assert!((m.length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = PhiltrumRidgeMorph {
            definition: 0.3,
            width: 0.6,
            length: 0.9,
        };
        assert!((pr_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_defined_false() {
        /* default not defined */
        let m = new_philtrum_ridge_morph();
        assert!(!pr_is_defined(&m));
    }

    #[test]
    fn test_is_defined_true() {
        /* above 0.5 */
        let m = PhiltrumRidgeMorph {
            definition: 0.8,
            width: 0.0,
            length: 0.0,
        };
        assert!(pr_is_defined(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint blend */
        let a = PhiltrumRidgeMorph {
            definition: 0.0,
            width: 0.0,
            length: 0.0,
        };
        let b = PhiltrumRidgeMorph {
            definition: 1.0,
            width: 1.0,
            length: 1.0,
        };
        let c = pr_blend(&a, &b, 0.5);
        assert!((c.definition - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let m = PhiltrumRidgeMorph {
            definition: 0.2,
            width: 0.5,
            length: 0.8,
        };
        let m2 = m.clone();
        assert_eq!(m.length, m2.length);
    }
}
