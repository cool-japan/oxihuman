// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct ZygomaticArchMorph {
    pub width: f32,
    pub prominence: f32,
}

pub fn new_zygomatic_arch_morph() -> ZygomaticArchMorph {
    ZygomaticArchMorph {
        width: 0.0,
        prominence: 0.0,
    }
}

pub fn zygomatic_set_width(m: &mut ZygomaticArchMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

pub fn zygomatic_set_prominence(m: &mut ZygomaticArchMorph, v: f32) {
    m.prominence = v.clamp(0.0, 1.0);
}

pub fn zygomatic_is_prominent(m: &ZygomaticArchMorph) -> bool {
    m.prominence > 0.5
}

pub fn zygomatic_overall_weight(m: &ZygomaticArchMorph) -> f32 {
    (m.width + m.prominence) / 2.0
}

pub fn zygomatic_blend(
    a: &ZygomaticArchMorph,
    b: &ZygomaticArchMorph,
    t: f32,
) -> ZygomaticArchMorph {
    let t = t.clamp(0.0, 1.0);
    ZygomaticArchMorph {
        width: a.width + (b.width - a.width) * t,
        prominence: a.prominence + (b.prominence - a.prominence) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* zero init */
        let m = new_zygomatic_arch_morph();
        assert_eq!(m.width, 0.0);
        assert_eq!(m.prominence, 0.0);
    }

    #[test]
    fn test_set_width() {
        /* valid width */
        let mut m = new_zygomatic_arch_morph();
        zygomatic_set_width(&mut m, 0.6);
        assert!((m.width - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamp_high() {
        /* clamp high */
        let mut m = new_zygomatic_arch_morph();
        zygomatic_set_width(&mut m, 5.0);
        assert_eq!(m.width, 1.0);
    }

    #[test]
    fn test_set_prominence() {
        /* valid prominence */
        let mut m = new_zygomatic_arch_morph();
        zygomatic_set_prominence(&mut m, 0.3);
        assert!((m.prominence - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_prominence_clamp_low() {
        /* clamp low */
        let mut m = new_zygomatic_arch_morph();
        zygomatic_set_prominence(&mut m, -2.0);
        assert_eq!(m.prominence, 0.0);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = ZygomaticArchMorph {
            width: 0.4,
            prominence: 0.8,
        };
        assert!((zygomatic_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_prominent_false() {
        /* default not prominent */
        let m = new_zygomatic_arch_morph();
        assert!(!zygomatic_is_prominent(&m));
    }

    #[test]
    fn test_is_prominent_true() {
        /* above 0.5 */
        let m = ZygomaticArchMorph {
            width: 0.0,
            prominence: 0.8,
        };
        assert!(zygomatic_is_prominent(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint blend */
        let a = ZygomaticArchMorph {
            width: 0.0,
            prominence: 0.0,
        };
        let b = ZygomaticArchMorph {
            width: 1.0,
            prominence: 1.0,
        };
        let c = zygomatic_blend(&a, &b, 0.5);
        assert!((c.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let m = ZygomaticArchMorph {
            width: 0.3,
            prominence: 0.7,
        };
        let m2 = m.clone();
        assert_eq!(m.prominence, m2.prominence);
    }
}
