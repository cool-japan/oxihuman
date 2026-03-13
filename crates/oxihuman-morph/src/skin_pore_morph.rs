// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct SkinPoreMorph {
    pub pore_size: f32,
    pub pore_density: f32,
    pub visibility: f32,
}

pub fn new_skin_pore_morph() -> SkinPoreMorph {
    SkinPoreMorph {
        pore_size: 0.0,
        pore_density: 0.0,
        visibility: 0.0,
    }
}

pub fn pore_set_size(m: &mut SkinPoreMorph, v: f32) {
    m.pore_size = v.clamp(0.0, 1.0);
}

pub fn pore_set_density(m: &mut SkinPoreMorph, v: f32) {
    m.pore_density = v.clamp(0.0, 1.0);
}

pub fn pore_overall_weight(m: &SkinPoreMorph) -> f32 {
    (m.pore_size + m.pore_density + m.visibility) / 3.0
}

pub fn pore_blend(a: &SkinPoreMorph, b: &SkinPoreMorph, t: f32) -> SkinPoreMorph {
    let t = t.clamp(0.0, 1.0);
    SkinPoreMorph {
        pore_size: a.pore_size + (b.pore_size - a.pore_size) * t,
        pore_density: a.pore_density + (b.pore_density - a.pore_density) * t,
        visibility: a.visibility + (b.visibility - a.visibility) * t,
    }
}

pub fn pore_is_visible(m: &SkinPoreMorph) -> bool {
    m.visibility > 0.1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all fields should be zero */
        let m = new_skin_pore_morph();
        assert_eq!(m.pore_size, 0.0);
        assert_eq!(m.pore_density, 0.0);
        assert_eq!(m.visibility, 0.0);
    }

    #[test]
    fn test_set_size_clamp() {
        /* value is clamped to [0,1] */
        let mut m = new_skin_pore_morph();
        pore_set_size(&mut m, 2.5);
        assert_eq!(m.pore_size, 1.0);
        pore_set_size(&mut m, -1.0);
        assert_eq!(m.pore_size, 0.0);
    }

    #[test]
    fn test_set_density_clamp() {
        /* density clamped */
        let mut m = new_skin_pore_morph();
        pore_set_density(&mut m, 0.7);
        assert!((m.pore_density - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_overall_weight_average() {
        /* overall weight is mean of three fields */
        let mut m = new_skin_pore_morph();
        m.pore_size = 0.3;
        m.pore_density = 0.6;
        m.visibility = 0.9;
        let w = pore_overall_weight(&m);
        assert!((w - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_visible_false() {
        /* visibility 0 => not visible */
        let m = new_skin_pore_morph();
        assert!(!pore_is_visible(&m));
    }

    #[test]
    fn test_is_visible_true() {
        /* visibility above threshold */
        let mut m = new_skin_pore_morph();
        m.visibility = 0.5;
        assert!(pore_is_visible(&m));
    }

    #[test]
    fn test_blend_midpoint() {
        /* blend at t=0.5 gives average */
        let a = SkinPoreMorph {
            pore_size: 0.0,
            pore_density: 0.0,
            visibility: 0.0,
        };
        let b = SkinPoreMorph {
            pore_size: 1.0,
            pore_density: 1.0,
            visibility: 1.0,
        };
        let c = pore_blend(&a, &b, 0.5);
        assert!((c.pore_size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_at_zero() {
        /* blend t=0 returns a */
        let a = SkinPoreMorph {
            pore_size: 0.2,
            pore_density: 0.3,
            visibility: 0.4,
        };
        let b = SkinPoreMorph {
            pore_size: 0.8,
            pore_density: 0.9,
            visibility: 1.0,
        };
        let c = pore_blend(&a, &b, 0.0);
        assert!((c.pore_size - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_blend_at_one() {
        /* blend t=1 returns b */
        let a = SkinPoreMorph {
            pore_size: 0.0,
            pore_density: 0.0,
            visibility: 0.0,
        };
        let b = SkinPoreMorph {
            pore_size: 0.5,
            pore_density: 0.6,
            visibility: 0.7,
        };
        let c = pore_blend(&a, &b, 1.0);
        assert!((c.pore_size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone produces independent copy */
        let m = new_skin_pore_morph();
        let m2 = m.clone();
        assert_eq!(m.pore_size, m2.pore_size);
    }
}
