// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct OrbitalRimMorph {
    pub depth: f32,
    pub roundness: f32,
    pub height: f32,
}

pub fn new_orbital_rim_morph() -> OrbitalRimMorph {
    OrbitalRimMorph {
        depth: 0.0,
        roundness: 0.0,
        height: 0.0,
    }
}

pub fn or_set_depth(m: &mut OrbitalRimMorph, v: f32) {
    m.depth = v.clamp(0.0, 1.0);
}

pub fn or_set_roundness(m: &mut OrbitalRimMorph, v: f32) {
    m.roundness = v.clamp(0.0, 1.0);
}

pub fn or_set_height(m: &mut OrbitalRimMorph, v: f32) {
    m.height = v.clamp(0.0, 1.0);
}

pub fn or_overall_weight(m: &OrbitalRimMorph) -> f32 {
    (m.depth + m.roundness + m.height) / 3.0
}

pub fn or_blend(a: &OrbitalRimMorph, b: &OrbitalRimMorph, t: f32) -> OrbitalRimMorph {
    let t = t.clamp(0.0, 1.0);
    OrbitalRimMorph {
        depth: a.depth + (b.depth - a.depth) * t,
        roundness: a.roundness + (b.roundness - a.roundness) * t,
        height: a.height + (b.height - a.height) * t,
    }
}

pub fn or_is_deep(m: &OrbitalRimMorph) -> bool {
    m.depth > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* zero init */
        let m = new_orbital_rim_morph();
        assert_eq!(m.depth, 0.0);
        assert_eq!(m.roundness, 0.0);
        assert_eq!(m.height, 0.0);
    }

    #[test]
    fn test_set_depth() {
        /* valid depth */
        let mut m = new_orbital_rim_morph();
        or_set_depth(&mut m, 0.5);
        assert!((m.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamp_high() {
        /* clamp high */
        let mut m = new_orbital_rim_morph();
        or_set_depth(&mut m, 2.0);
        assert_eq!(m.depth, 1.0);
    }

    #[test]
    fn test_set_roundness_clamp_low() {
        /* clamp low */
        let mut m = new_orbital_rim_morph();
        or_set_roundness(&mut m, -0.5);
        assert_eq!(m.roundness, 0.0);
    }

    #[test]
    fn test_set_height() {
        /* valid height */
        let mut m = new_orbital_rim_morph();
        or_set_height(&mut m, 0.8);
        assert!((m.height - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = OrbitalRimMorph {
            depth: 0.3,
            roundness: 0.6,
            height: 0.9,
        };
        assert!((or_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_deep_false() {
        /* default not deep */
        let m = new_orbital_rim_morph();
        assert!(!or_is_deep(&m));
    }

    #[test]
    fn test_is_deep_true() {
        /* above 0.5 */
        let m = OrbitalRimMorph {
            depth: 0.9,
            roundness: 0.0,
            height: 0.0,
        };
        assert!(or_is_deep(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = OrbitalRimMorph {
            depth: 0.0,
            roundness: 0.0,
            height: 0.0,
        };
        let b = OrbitalRimMorph {
            depth: 1.0,
            roundness: 1.0,
            height: 1.0,
        };
        let c = or_blend(&a, &b, 0.5);
        assert!((c.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let m = OrbitalRimMorph {
            depth: 0.2,
            roundness: 0.5,
            height: 0.8,
        };
        let m2 = m.clone();
        assert_eq!(m.height, m2.height);
    }
}
