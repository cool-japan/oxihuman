// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct EyelidCreaseMorph {
    pub crease_height: f32,
    pub fold_depth: f32,
    pub double_lid: f32,
}

pub fn new_eyelid_crease_morph() -> EyelidCreaseMorph {
    EyelidCreaseMorph {
        crease_height: 0.0,
        fold_depth: 0.0,
        double_lid: 0.0,
    }
}

pub fn eyelid_set_crease(m: &mut EyelidCreaseMorph, v: f32) {
    m.crease_height = v.clamp(0.0, 1.0);
}

pub fn eyelid_has_crease(m: &EyelidCreaseMorph) -> bool {
    m.crease_height > 0.2
}

pub fn eyelid_overall_weight(m: &EyelidCreaseMorph) -> f32 {
    (m.crease_height.abs() + m.fold_depth.abs() + m.double_lid.abs()) / 3.0
}

pub fn eyelid_blend(a: &EyelidCreaseMorph, b: &EyelidCreaseMorph, t: f32) -> EyelidCreaseMorph {
    let t = t.clamp(0.0, 1.0);
    EyelidCreaseMorph {
        crease_height: a.crease_height + (b.crease_height - a.crease_height) * t,
        fold_depth: a.fold_depth + (b.fold_depth - a.fold_depth) * t,
        double_lid: a.double_lid + (b.double_lid - a.double_lid) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_eyelid_crease_morph() {
        /* defaults to zero */
        let m = new_eyelid_crease_morph();
        assert_eq!(m.crease_height, 0.0);
    }

    #[test]
    fn test_eyelid_set_crease() {
        /* crease is set */
        let mut m = new_eyelid_crease_morph();
        eyelid_set_crease(&mut m, 0.5);
        assert!((m.crease_height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_eyelid_has_crease_false() {
        /* no crease at default */
        let m = new_eyelid_crease_morph();
        assert!(!eyelid_has_crease(&m));
    }

    #[test]
    fn test_eyelid_has_crease_true() {
        /* has crease when > 0.2 */
        let mut m = new_eyelid_crease_morph();
        m.crease_height = 0.3;
        assert!(eyelid_has_crease(&m));
    }

    #[test]
    fn test_eyelid_blend() {
        /* midpoint blend */
        let a = new_eyelid_crease_morph();
        let b = EyelidCreaseMorph {
            crease_height: 1.0,
            fold_depth: 1.0,
            double_lid: 1.0,
        };
        let r = eyelid_blend(&a, &b, 0.5);
        assert!((r.crease_height - 0.5).abs() < 1e-6);
    }
}
