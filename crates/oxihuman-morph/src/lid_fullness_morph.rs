// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct LidFullnessMorph {
    pub upper_left: f32,
    pub upper_right: f32,
    pub lower_left: f32,
    pub lower_right: f32,
}

pub fn new_lid_fullness_morph() -> LidFullnessMorph {
    LidFullnessMorph {
        upper_left: 0.0,
        upper_right: 0.0,
        lower_left: 0.0,
        lower_right: 0.0,
    }
}

pub fn lid_set_upper(m: &mut LidFullnessMorph, v: f32) {
    let v = v.clamp(0.0, 1.0);
    m.upper_left = v;
    m.upper_right = v;
}

pub fn lid_set_lower(m: &mut LidFullnessMorph, v: f32) {
    let v = v.clamp(0.0, 1.0);
    m.lower_left = v;
    m.lower_right = v;
}

pub fn lid_overall_weight(m: &LidFullnessMorph) -> f32 {
    (m.upper_left + m.upper_right + m.lower_left + m.lower_right) / 4.0
}

pub fn lid_blend(a: &LidFullnessMorph, b: &LidFullnessMorph, t: f32) -> LidFullnessMorph {
    let t = t.clamp(0.0, 1.0);
    LidFullnessMorph {
        upper_left: a.upper_left + (b.upper_left - a.upper_left) * t,
        upper_right: a.upper_right + (b.upper_right - a.upper_right) * t,
        lower_left: a.lower_left + (b.lower_left - a.lower_left) * t,
        lower_right: a.lower_right + (b.lower_right - a.lower_right) * t,
    }
}

pub fn lid_is_puffy(m: &LidFullnessMorph) -> bool {
    m.upper_left > 0.6 || m.upper_right > 0.6 || m.lower_left > 0.6 || m.lower_right > 0.6
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* four fields zero */
        let m = new_lid_fullness_morph();
        assert_eq!(m.upper_left, 0.0);
        assert_eq!(m.lower_right, 0.0);
    }

    #[test]
    fn test_set_upper_clamp() {
        /* clamp above 1 */
        let mut m = new_lid_fullness_morph();
        lid_set_upper(&mut m, 2.0);
        assert_eq!(m.upper_left, 1.0);
        assert_eq!(m.upper_right, 1.0);
    }

    #[test]
    fn test_set_lower() {
        /* stores value for both lower lids */
        let mut m = new_lid_fullness_morph();
        lid_set_lower(&mut m, 0.4);
        assert!((m.lower_left - 0.4).abs() < 1e-6);
        assert!((m.lower_right - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_overall_weight_mean() {
        /* mean of four fields */
        let m = LidFullnessMorph {
            upper_left: 0.4,
            upper_right: 0.4,
            lower_left: 0.4,
            lower_right: 0.4,
        };
        assert!((lid_overall_weight(&m) - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_is_puffy_false() {
        /* all zero => not puffy */
        let m = new_lid_fullness_morph();
        assert!(!lid_is_puffy(&m));
    }

    #[test]
    fn test_is_puffy_true() {
        /* one field above 0.6 => puffy */
        let m = LidFullnessMorph {
            upper_left: 0.0,
            upper_right: 0.7,
            lower_left: 0.0,
            lower_right: 0.0,
        };
        assert!(lid_is_puffy(&m));
    }

    #[test]
    fn test_blend_midpoint() {
        /* t=0.5 */
        let a = LidFullnessMorph {
            upper_left: 0.0,
            upper_right: 0.0,
            lower_left: 0.0,
            lower_right: 0.0,
        };
        let b = LidFullnessMorph {
            upper_left: 1.0,
            upper_right: 1.0,
            lower_left: 1.0,
            lower_right: 1.0,
        };
        let c = lid_blend(&a, &b, 0.5);
        assert!((c.upper_left - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_endpoints() {
        /* t=0 and t=1 */
        let a = LidFullnessMorph {
            upper_left: 0.2,
            upper_right: 0.2,
            lower_left: 0.2,
            lower_right: 0.2,
        };
        let b = LidFullnessMorph {
            upper_left: 0.8,
            upper_right: 0.8,
            lower_left: 0.8,
            lower_right: 0.8,
        };
        let r0 = lid_blend(&a, &b, 0.0);
        assert!((r0.upper_left - 0.2).abs() < 1e-6);
        let r1 = lid_blend(&a, &b, 1.0);
        assert!((r1.upper_left - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let m = new_lid_fullness_morph();
        let m2 = m.clone();
        assert_eq!(m.upper_left, m2.upper_left);
    }
}
