// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct OrbitalDepthMorph {
    pub depth: f32,
    pub width: f32,
    pub tilt_deg: f32,
}

pub fn new_orbital_depth_morph() -> OrbitalDepthMorph {
    OrbitalDepthMorph {
        depth: 0.0,
        width: 0.5,
        tilt_deg: 0.0,
    }
}

pub fn orbital_set_depth(m: &mut OrbitalDepthMorph, v: f32) {
    m.depth = v.clamp(0.0, 1.0);
}

pub fn orbital_is_deep(m: &OrbitalDepthMorph) -> bool {
    m.depth > 0.5
}

pub fn orbital_overall_weight(m: &OrbitalDepthMorph) -> f32 {
    (m.depth + m.width) * 0.5
}

pub fn orbital_blend(a: &OrbitalDepthMorph, b: &OrbitalDepthMorph, t: f32) -> OrbitalDepthMorph {
    let t = t.clamp(0.0, 1.0);
    OrbitalDepthMorph {
        depth: a.depth + (b.depth - a.depth) * t,
        width: a.width + (b.width - a.width) * t,
        tilt_deg: a.tilt_deg + (b.tilt_deg - a.tilt_deg) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_orbital_depth_morph() {
        /* depth defaults to 0 */
        let m = new_orbital_depth_morph();
        assert!((m.depth - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_orbital_set_depth() {
        /* set depth and verify */
        let mut m = new_orbital_depth_morph();
        orbital_set_depth(&mut m, 0.8);
        assert!((m.depth - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_orbital_is_deep_true() {
        /* depth > 0.5 is deep */
        let mut m = new_orbital_depth_morph();
        orbital_set_depth(&mut m, 0.7);
        assert!(orbital_is_deep(&m));
    }

    #[test]
    fn test_orbital_is_deep_false() {
        /* depth <= 0.5 is not deep */
        let m = new_orbital_depth_morph();
        assert!(!orbital_is_deep(&m));
    }

    #[test]
    fn test_orbital_overall_weight() {
        /* weight is average of depth and width */
        let mut m = new_orbital_depth_morph();
        orbital_set_depth(&mut m, 1.0);
        m.width = 1.0;
        let w = orbital_overall_weight(&m);
        assert!((w - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_orbital_blend() {
        /* blend at t=0.5 produces midpoint */
        let a = new_orbital_depth_morph();
        let mut b = new_orbital_depth_morph();
        b.depth = 1.0;
        let mid = orbital_blend(&a, &b, 0.5);
        assert!((mid.depth - 0.5).abs() < 1e-6);
    }
}
