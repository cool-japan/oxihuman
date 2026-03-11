// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct PhiltrumMorph {
    pub depth: f32,
    pub width: f32,
    pub length: f32,
}

pub fn new_philtrum_morph() -> PhiltrumMorph {
    PhiltrumMorph {
        depth: 0.0,
        width: 0.0,
        length: 0.0,
    }
}

pub fn philtrum_set_depth(m: &mut PhiltrumMorph, v: f32) {
    m.depth = v.clamp(0.0, 1.0);
}

pub fn philtrum_set_width(m: &mut PhiltrumMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

pub fn philtrum_overall_weight(m: &PhiltrumMorph) -> f32 {
    (m.depth + m.width + m.length) / 3.0
}

pub fn philtrum_blend(a: &PhiltrumMorph, b: &PhiltrumMorph, t: f32) -> PhiltrumMorph {
    let t = t.clamp(0.0, 1.0);
    PhiltrumMorph {
        depth: a.depth + (b.depth - a.depth) * t,
        width: a.width + (b.width - a.width) * t,
        length: a.length + (b.length - a.length) * t,
    }
}

pub fn philtrum_is_deep(m: &PhiltrumMorph) -> bool {
    m.depth > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all fields zero */
        let m = new_philtrum_morph();
        assert_eq!(m.depth, 0.0);
    }

    #[test]
    fn test_set_depth_clamp() {
        /* clamps above 1 */
        let mut m = new_philtrum_morph();
        philtrum_set_depth(&mut m, 3.0);
        assert_eq!(m.depth, 1.0);
    }

    #[test]
    fn test_set_width_clamp() {
        /* clamps below 0 */
        let mut m = new_philtrum_morph();
        philtrum_set_width(&mut m, -0.5);
        assert_eq!(m.width, 0.0);
    }

    #[test]
    fn test_overall_weight() {
        /* average of fields */
        let m = PhiltrumMorph {
            depth: 0.9,
            width: 0.3,
            length: 0.6,
        };
        assert!((philtrum_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_deep_false() {
        /* depth=0 => not deep */
        let m = new_philtrum_morph();
        assert!(!philtrum_is_deep(&m));
    }

    #[test]
    fn test_is_deep_true() {
        /* depth=0.8 => deep */
        let m = PhiltrumMorph {
            depth: 0.8,
            width: 0.0,
            length: 0.0,
        };
        assert!(philtrum_is_deep(&m));
    }

    #[test]
    fn test_blend_midpoint() {
        /* t=0.5 interpolates correctly */
        let a = PhiltrumMorph {
            depth: 0.0,
            width: 0.0,
            length: 0.0,
        };
        let b = PhiltrumMorph {
            depth: 1.0,
            width: 1.0,
            length: 1.0,
        };
        let c = philtrum_blend(&a, &b, 0.5);
        assert!((c.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_at_zero() {
        /* t=0 returns a */
        let a = PhiltrumMorph {
            depth: 0.2,
            width: 0.3,
            length: 0.4,
        };
        let b = PhiltrumMorph {
            depth: 0.9,
            width: 0.8,
            length: 0.7,
        };
        let r = philtrum_blend(&a, &b, 0.0);
        assert!((r.depth - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_blend_at_one() {
        /* t=1 returns b */
        let a = PhiltrumMorph {
            depth: 0.2,
            width: 0.3,
            length: 0.4,
        };
        let b = PhiltrumMorph {
            depth: 0.9,
            width: 0.8,
            length: 0.7,
        };
        let r = philtrum_blend(&a, &b, 1.0);
        assert!((r.depth - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let m = PhiltrumMorph {
            depth: 0.5,
            width: 0.3,
            length: 0.1,
        };
        let m2 = m.clone();
        assert_eq!(m.depth, m2.depth);
    }
}
