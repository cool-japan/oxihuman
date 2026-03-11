// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct PhiltrumDepthMorph {
    pub depth: f32,
    pub width: f32,
    pub length: f32,
}

pub fn new_philtrum_depth_morph() -> PhiltrumDepthMorph {
    PhiltrumDepthMorph {
        depth: 0.0,
        width: 0.0,
        length: 0.0,
    }
}

pub fn philtrum_depth_set_depth(m: &mut PhiltrumDepthMorph, v: f32) {
    m.depth = v.clamp(0.0, 1.0);
}

pub fn philtrum_depth_is_deep(m: &PhiltrumDepthMorph) -> bool {
    m.depth > 0.5
}

pub fn philtrum_depth_overall_weight(m: &PhiltrumDepthMorph) -> f32 {
    (m.depth.abs() + m.width.abs() + m.length.abs()) / 3.0
}

pub fn philtrum_depth_blend(
    a: &PhiltrumDepthMorph,
    b: &PhiltrumDepthMorph,
    t: f32,
) -> PhiltrumDepthMorph {
    let t = t.clamp(0.0, 1.0);
    PhiltrumDepthMorph {
        depth: a.depth + (b.depth - a.depth) * t,
        width: a.width + (b.width - a.width) * t,
        length: a.length + (b.length - a.length) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_philtrum_depth_morph() {
        /* depth defaults to 0 */
        let m = new_philtrum_depth_morph();
        assert_eq!(m.depth, 0.0);
    }

    #[test]
    fn test_philtrum_depth_set_depth() {
        /* depth is set */
        let mut m = new_philtrum_depth_morph();
        philtrum_depth_set_depth(&mut m, 0.6);
        assert!((m.depth - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_philtrum_depth_is_deep_false() {
        /* not deep at 0 */
        let m = new_philtrum_depth_morph();
        assert!(!philtrum_depth_is_deep(&m));
    }

    #[test]
    fn test_philtrum_depth_is_deep_true() {
        /* deep when depth > 0.5 */
        let mut m = new_philtrum_depth_morph();
        m.depth = 0.7;
        assert!(philtrum_depth_is_deep(&m));
    }

    #[test]
    fn test_philtrum_depth_blend() {
        /* blend at 0.5 is midpoint */
        let a = new_philtrum_depth_morph();
        let b = PhiltrumDepthMorph {
            depth: 1.0,
            width: 1.0,
            length: 1.0,
        };
        let r = philtrum_depth_blend(&a, &b, 0.5);
        assert!((r.depth - 0.5).abs() < 1e-6);
    }
}
