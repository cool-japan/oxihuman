// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct CorrugatorMorph {
    pub contraction: f32,
    pub line_depth: f32,
}

pub fn new_corrugator_morph() -> CorrugatorMorph {
    CorrugatorMorph {
        contraction: 0.0,
        line_depth: 0.0,
    }
}

pub fn corrugator_set_contraction(m: &mut CorrugatorMorph, v: f32) {
    m.contraction = v.clamp(0.0, 1.0);
}

pub fn corrugator_is_contracted(m: &CorrugatorMorph) -> bool {
    m.contraction > 0.3
}

pub fn corrugator_overall_weight(m: &CorrugatorMorph) -> f32 {
    (m.contraction + m.line_depth) * 0.5
}

pub fn corrugator_blend(a: &CorrugatorMorph, b: &CorrugatorMorph, t: f32) -> CorrugatorMorph {
    let t = t.clamp(0.0, 1.0);
    CorrugatorMorph {
        contraction: a.contraction + (b.contraction - a.contraction) * t,
        line_depth: a.line_depth + (b.line_depth - a.line_depth) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_corrugator_morph() {
        /* contraction starts at 0 */
        let m = new_corrugator_morph();
        assert!((m.contraction - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_corrugator_set_contraction() {
        /* set and retrieve contraction */
        let mut m = new_corrugator_morph();
        corrugator_set_contraction(&mut m, 0.4);
        assert!((m.contraction - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_corrugator_is_contracted_true() {
        /* > 0.3 is contracted */
        let mut m = new_corrugator_morph();
        corrugator_set_contraction(&mut m, 0.5);
        assert!(corrugator_is_contracted(&m));
    }

    #[test]
    fn test_corrugator_is_contracted_false() {
        /* 0 is not contracted */
        let m = new_corrugator_morph();
        assert!(!corrugator_is_contracted(&m));
    }

    #[test]
    fn test_corrugator_blend() {
        /* blend midpoint */
        let a = new_corrugator_morph();
        let mut b = new_corrugator_morph();
        b.contraction = 1.0;
        let mid = corrugator_blend(&a, &b, 0.5);
        assert!((mid.contraction - 0.5).abs() < 1e-6);
    }
}
