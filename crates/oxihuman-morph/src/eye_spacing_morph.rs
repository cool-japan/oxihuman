// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct EyeSpacingMorph {
    pub distance: f32,
    pub convergence: f32,
}

pub fn new_eye_spacing_morph() -> EyeSpacingMorph {
    EyeSpacingMorph {
        distance: 0.0,
        convergence: 0.0,
    }
}

pub fn eye_spacing_set_distance(m: &mut EyeSpacingMorph, v: f32) {
    m.distance = v.clamp(0.0, 1.0);
}

pub fn eye_spacing_is_wide(m: &EyeSpacingMorph) -> bool {
    m.distance > 0.5
}

pub fn eye_spacing_overall_weight(m: &EyeSpacingMorph) -> f32 {
    (m.distance.abs() + m.convergence.abs()) * 0.5
}

pub fn eye_spacing_blend(a: &EyeSpacingMorph, b: &EyeSpacingMorph, t: f32) -> EyeSpacingMorph {
    let t = t.clamp(0.0, 1.0);
    EyeSpacingMorph {
        distance: a.distance + (b.distance - a.distance) * t,
        convergence: a.convergence + (b.convergence - a.convergence) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_eye_spacing_morph() {
        /* default distance is 0 */
        let m = new_eye_spacing_morph();
        assert_eq!(m.distance, 0.0);
    }

    #[test]
    fn test_eye_spacing_set_distance() {
        /* distance is set */
        let mut m = new_eye_spacing_morph();
        eye_spacing_set_distance(&mut m, 0.3);
        assert!((m.distance - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_eye_spacing_is_wide_false() {
        /* not wide at default */
        let m = new_eye_spacing_morph();
        assert!(!eye_spacing_is_wide(&m));
    }

    #[test]
    fn test_eye_spacing_is_wide_true() {
        /* wide when distance > 0.5 */
        let mut m = new_eye_spacing_morph();
        m.distance = 0.6;
        assert!(eye_spacing_is_wide(&m));
    }

    #[test]
    fn test_eye_spacing_blend() {
        /* blend at 1 returns b */
        let a = new_eye_spacing_morph();
        let b = EyeSpacingMorph {
            distance: 1.0,
            convergence: 1.0,
        };
        let r = eye_spacing_blend(&a, &b, 1.0);
        assert!((r.distance - 1.0).abs() < 1e-6);
    }
}
