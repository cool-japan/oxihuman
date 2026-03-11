// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct MasseterMorph {
    pub hypertrophy: f32,
    pub width: f32,
    pub length: f32,
}

pub fn new_masseter_morph() -> MasseterMorph {
    MasseterMorph {
        hypertrophy: 0.0,
        width: 0.5,
        length: 0.5,
    }
}

pub fn masseter_set_hypertrophy(m: &mut MasseterMorph, v: f32) {
    m.hypertrophy = v.clamp(0.0, 1.0);
}

pub fn masseter_is_hypertrophied(m: &MasseterMorph) -> bool {
    m.hypertrophy > 0.6
}

pub fn masseter_overall_weight(m: &MasseterMorph) -> f32 {
    (m.hypertrophy + m.width) * 0.5
}

pub fn masseter_blend(a: &MasseterMorph, b: &MasseterMorph, t: f32) -> MasseterMorph {
    let t = t.clamp(0.0, 1.0);
    MasseterMorph {
        hypertrophy: a.hypertrophy + (b.hypertrophy - a.hypertrophy) * t,
        width: a.width + (b.width - a.width) * t,
        length: a.length + (b.length - a.length) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_masseter_morph() {
        /* hypertrophy defaults to 0 */
        let m = new_masseter_morph();
        assert!((m.hypertrophy - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_masseter_set_hypertrophy() {
        /* set and retrieve hypertrophy */
        let mut m = new_masseter_morph();
        masseter_set_hypertrophy(&mut m, 0.7);
        assert!((m.hypertrophy - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_masseter_is_hypertrophied_true() {
        /* > 0.6 is hypertrophied */
        let mut m = new_masseter_morph();
        masseter_set_hypertrophy(&mut m, 0.9);
        assert!(masseter_is_hypertrophied(&m));
    }

    #[test]
    fn test_masseter_is_hypertrophied_false() {
        /* zero not hypertrophied */
        let m = new_masseter_morph();
        assert!(!masseter_is_hypertrophied(&m));
    }

    #[test]
    fn test_masseter_blend() {
        /* blend at t=0.5 */
        let a = new_masseter_morph();
        let mut b = new_masseter_morph();
        b.hypertrophy = 1.0;
        let mid = masseter_blend(&a, &b, 0.5);
        assert!((mid.hypertrophy - 0.5).abs() < 1e-6);
    }
}
