// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct OralCommissureMorph {
    pub angle: f32,
    pub depth: f32,
}

pub fn new_oral_commissure_morph() -> OralCommissureMorph {
    OralCommissureMorph {
        angle: 0.0,
        depth: 0.0,
    }
}

pub fn oc_set_angle(m: &mut OralCommissureMorph, v: f32) {
    m.angle = v.clamp(-1.0, 1.0);
}

pub fn oc_set_depth(m: &mut OralCommissureMorph, v: f32) {
    m.depth = v.clamp(0.0, 1.0);
}

pub fn oc_overall_weight(m: &OralCommissureMorph) -> f32 {
    (m.angle.abs() + m.depth) / 2.0
}

pub fn oc_blend(a: &OralCommissureMorph, b: &OralCommissureMorph, t: f32) -> OralCommissureMorph {
    let t = t.clamp(0.0, 1.0);
    OralCommissureMorph {
        angle: a.angle + (b.angle - a.angle) * t,
        depth: a.depth + (b.depth - a.depth) * t,
    }
}

pub fn oc_is_downturned(m: &OralCommissureMorph) -> bool {
    m.angle < -0.2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* zero init */
        let m = new_oral_commissure_morph();
        assert_eq!(m.angle, 0.0);
        assert_eq!(m.depth, 0.0);
    }

    #[test]
    fn test_set_angle() {
        /* valid angle */
        let mut m = new_oral_commissure_morph();
        oc_set_angle(&mut m, 0.5);
        assert!((m.angle - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_angle_clamp_high() {
        /* clamp high */
        let mut m = new_oral_commissure_morph();
        oc_set_angle(&mut m, 5.0);
        assert_eq!(m.angle, 1.0);
    }

    #[test]
    fn test_set_angle_clamp_low() {
        /* clamp low */
        let mut m = new_oral_commissure_morph();
        oc_set_angle(&mut m, -5.0);
        assert_eq!(m.angle, -1.0);
    }

    #[test]
    fn test_set_depth() {
        /* valid depth */
        let mut m = new_oral_commissure_morph();
        oc_set_depth(&mut m, 0.7);
        assert!((m.depth - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_overall_weight() {
        /* combined weight */
        let m = OralCommissureMorph {
            angle: -0.4,
            depth: 0.8,
        };
        assert!((oc_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_downturned_false() {
        /* default not downturned */
        let m = new_oral_commissure_morph();
        assert!(!oc_is_downturned(&m));
    }

    #[test]
    fn test_is_downturned_true() {
        /* negative angle below -0.2 */
        let m = OralCommissureMorph {
            angle: -0.5,
            depth: 0.0,
        };
        assert!(oc_is_downturned(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint blend */
        let a = OralCommissureMorph {
            angle: -1.0,
            depth: 0.0,
        };
        let b = OralCommissureMorph {
            angle: 1.0,
            depth: 1.0,
        };
        let c = oc_blend(&a, &b, 0.5);
        assert!((c.angle - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let m = OralCommissureMorph {
            angle: 0.3,
            depth: 0.6,
        };
        let m2 = m.clone();
        assert_eq!(m.depth, m2.depth);
    }
}
