// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct BackMuscleMorph {
    pub latissimus: f32,
    pub trapezius: f32,
    pub erector_spinae: f32,
}

pub fn new_back_muscle_morph() -> BackMuscleMorph {
    BackMuscleMorph {
        latissimus: 0.0,
        trapezius: 0.0,
        erector_spinae: 0.0,
    }
}

pub fn back_set_latissimus(m: &mut BackMuscleMorph, v: f32) {
    m.latissimus = v.clamp(0.0, 1.0);
}

pub fn back_is_muscular(m: &BackMuscleMorph) -> bool {
    (m.latissimus + m.trapezius + m.erector_spinae) / 3.0 > 0.5
}

pub fn back_overall_weight(m: &BackMuscleMorph) -> f32 {
    (m.latissimus + m.trapezius + m.erector_spinae) / 3.0
}

pub fn back_blend(a: &BackMuscleMorph, b: &BackMuscleMorph, t: f32) -> BackMuscleMorph {
    let t = t.clamp(0.0, 1.0);
    BackMuscleMorph {
        latissimus: a.latissimus + (b.latissimus - a.latissimus) * t,
        trapezius: a.trapezius + (b.trapezius - a.trapezius) * t,
        erector_spinae: a.erector_spinae + (b.erector_spinae - a.erector_spinae) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_zero() {
        /* all zero by default */
        let m = new_back_muscle_morph();
        assert!((m.latissimus + m.trapezius + m.erector_spinae).abs() < 1e-5);
    }

    #[test]
    fn test_set_latissimus() {
        /* clamped */
        let mut m = new_back_muscle_morph();
        back_set_latissimus(&mut m, 0.7);
        assert!((m.latissimus - 0.7).abs() < 1e-5);
    }

    #[test]
    fn test_not_muscular_by_default() {
        /* default isn't muscular */
        let m = new_back_muscle_morph();
        assert!(!back_is_muscular(&m));
    }

    #[test]
    fn test_overall_weight_zero() {
        /* zero by default */
        let m = new_back_muscle_morph();
        assert!((back_overall_weight(&m)).abs() < 1e-5);
    }

    #[test]
    fn test_blend() {
        /* blend midpoint */
        let a = new_back_muscle_morph();
        let b = BackMuscleMorph {
            latissimus: 1.0,
            trapezius: 1.0,
            erector_spinae: 1.0,
        };
        let c = back_blend(&a, &b, 0.5);
        assert!((c.latissimus - 0.5).abs() < 1e-5);
    }
}
