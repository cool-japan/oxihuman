// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct OrbicularisOculiMorph {
    pub contraction: f32,
    pub crow_feet_depth: f32,
    pub lower_bag_weight: f32,
}

pub fn new_orbicularis_oculi_morph() -> OrbicularisOculiMorph {
    OrbicularisOculiMorph {
        contraction: 0.0,
        crow_feet_depth: 0.0,
        lower_bag_weight: 0.0,
    }
}

pub fn orbicularis_set_contraction(m: &mut OrbicularisOculiMorph, v: f32) {
    m.contraction = v.clamp(0.0, 1.0);
}

pub fn orbicularis_shows_crow_feet(m: &OrbicularisOculiMorph) -> bool {
    m.crow_feet_depth > 0.3
}

pub fn orbicularis_overall_weight(m: &OrbicularisOculiMorph) -> f32 {
    (m.contraction + m.crow_feet_depth) * 0.5
}

pub fn orbicularis_blend(
    a: &OrbicularisOculiMorph,
    b: &OrbicularisOculiMorph,
    t: f32,
) -> OrbicularisOculiMorph {
    let t = t.clamp(0.0, 1.0);
    OrbicularisOculiMorph {
        contraction: a.contraction + (b.contraction - a.contraction) * t,
        crow_feet_depth: a.crow_feet_depth + (b.crow_feet_depth - a.crow_feet_depth) * t,
        lower_bag_weight: a.lower_bag_weight + (b.lower_bag_weight - a.lower_bag_weight) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_orbicularis_oculi_morph() {
        /* contraction starts at 0 */
        let m = new_orbicularis_oculi_morph();
        assert!((m.contraction - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_orbicularis_set_contraction() {
        /* set contraction */
        let mut m = new_orbicularis_oculi_morph();
        orbicularis_set_contraction(&mut m, 0.6);
        assert!((m.contraction - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_orbicularis_shows_crow_feet_true() {
        /* crow feet depth > 0.3 */
        let mut m = new_orbicularis_oculi_morph();
        m.crow_feet_depth = 0.5;
        assert!(orbicularis_shows_crow_feet(&m));
    }

    #[test]
    fn test_orbicularis_shows_crow_feet_false() {
        /* zero depth no crow feet */
        let m = new_orbicularis_oculi_morph();
        assert!(!orbicularis_shows_crow_feet(&m));
    }

    #[test]
    fn test_orbicularis_blend() {
        /* blend midpoint contraction */
        let a = new_orbicularis_oculi_morph();
        let mut b = new_orbicularis_oculi_morph();
        b.contraction = 1.0;
        let mid = orbicularis_blend(&a, &b, 0.5);
        assert!((mid.contraction - 0.5).abs() < 1e-6);
    }
}
