// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct FrontalisMorph {
    pub contraction: f32,
    pub line_depth: f32,
    pub brow_elevation: f32,
}

pub fn new_frontalis_morph() -> FrontalisMorph {
    FrontalisMorph {
        contraction: 0.0,
        line_depth: 0.0,
        brow_elevation: 0.0,
    }
}

pub fn frontalis_set_contraction(m: &mut FrontalisMorph, v: f32) {
    m.contraction = v.clamp(0.0, 1.0);
}

pub fn frontalis_shows_lines(m: &FrontalisMorph) -> bool {
    m.line_depth > 0.3
}

pub fn frontalis_overall_weight(m: &FrontalisMorph) -> f32 {
    (m.contraction + m.line_depth) * 0.5
}

pub fn frontalis_blend(a: &FrontalisMorph, b: &FrontalisMorph, t: f32) -> FrontalisMorph {
    let t = t.clamp(0.0, 1.0);
    FrontalisMorph {
        contraction: a.contraction + (b.contraction - a.contraction) * t,
        line_depth: a.line_depth + (b.line_depth - a.line_depth) * t,
        brow_elevation: a.brow_elevation + (b.brow_elevation - a.brow_elevation) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_frontalis_morph() {
        /* contraction starts at 0 */
        let m = new_frontalis_morph();
        assert!((m.contraction - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_frontalis_set_contraction() {
        /* set and retrieve contraction */
        let mut m = new_frontalis_morph();
        frontalis_set_contraction(&mut m, 0.5);
        assert!((m.contraction - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_frontalis_shows_lines_true() {
        /* line_depth > 0.3 shows lines */
        let mut m = new_frontalis_morph();
        m.line_depth = 0.5;
        assert!(frontalis_shows_lines(&m));
    }

    #[test]
    fn test_frontalis_shows_lines_false() {
        /* zero depth no lines */
        let m = new_frontalis_morph();
        assert!(!frontalis_shows_lines(&m));
    }

    #[test]
    fn test_frontalis_overall_weight() {
        /* weight average of contraction and line_depth */
        let mut m = new_frontalis_morph();
        frontalis_set_contraction(&mut m, 1.0);
        m.line_depth = 1.0;
        assert!((frontalis_overall_weight(&m) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_frontalis_blend() {
        /* blend midpoint */
        let a = new_frontalis_morph();
        let mut b = new_frontalis_morph();
        b.contraction = 1.0;
        let mid = frontalis_blend(&a, &b, 0.5);
        assert!((mid.contraction - 0.5).abs() < 1e-6);
    }
}
