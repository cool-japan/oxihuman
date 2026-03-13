// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct DepressorAnguliMorph {
    pub contraction: f32,
    pub ptosis: f32,
}

pub fn new_depressor_anguli_morph() -> DepressorAnguliMorph {
    DepressorAnguliMorph {
        contraction: 0.0,
        ptosis: 0.0,
    }
}

pub fn depressor_set_contraction(m: &mut DepressorAnguliMorph, v: f32) {
    m.contraction = v.clamp(0.0, 1.0);
}

pub fn depressor_is_active(m: &DepressorAnguliMorph) -> bool {
    m.contraction > 0.2
}

pub fn depressor_overall_weight(m: &DepressorAnguliMorph) -> f32 {
    (m.contraction + m.ptosis) * 0.5
}

pub fn depressor_blend(
    a: &DepressorAnguliMorph,
    b: &DepressorAnguliMorph,
    t: f32,
) -> DepressorAnguliMorph {
    let t = t.clamp(0.0, 1.0);
    DepressorAnguliMorph {
        contraction: a.contraction + (b.contraction - a.contraction) * t,
        ptosis: a.ptosis + (b.ptosis - a.ptosis) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_depressor_anguli_morph() {
        /* contraction starts at 0 */
        let m = new_depressor_anguli_morph();
        assert!((m.contraction - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_depressor_set_contraction() {
        /* set and retrieve contraction */
        let mut m = new_depressor_anguli_morph();
        depressor_set_contraction(&mut m, 0.3);
        assert!((m.contraction - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_depressor_is_active_true() {
        /* > 0.2 is active */
        let mut m = new_depressor_anguli_morph();
        depressor_set_contraction(&mut m, 0.5);
        assert!(depressor_is_active(&m));
    }

    #[test]
    fn test_depressor_is_active_false() {
        /* 0 not active */
        let m = new_depressor_anguli_morph();
        assert!(!depressor_is_active(&m));
    }

    #[test]
    fn test_depressor_blend() {
        /* blend midpoint */
        let a = new_depressor_anguli_morph();
        let mut b = new_depressor_anguli_morph();
        b.contraction = 1.0;
        let mid = depressor_blend(&a, &b, 0.5);
        assert!((mid.contraction - 0.5).abs() < 1e-6);
    }
}
