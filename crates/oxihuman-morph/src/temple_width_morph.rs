// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct TempleWidthMorph {
    pub width: f32,
    pub prominence: f32,
}

pub fn new_temple_width_morph() -> TempleWidthMorph {
    TempleWidthMorph {
        width: 0.0,
        prominence: 0.0,
    }
}

pub fn temple_set_width(m: &mut TempleWidthMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

pub fn temple_set_prominence(m: &mut TempleWidthMorph, v: f32) {
    m.prominence = v.clamp(0.0, 1.0);
}

pub fn temple_is_wide(m: &TempleWidthMorph) -> bool {
    m.width > 0.5
}

pub fn temple_overall_weight(m: &TempleWidthMorph) -> f32 {
    (m.width + m.prominence) / 2.0
}

pub fn temple_blend(a: &TempleWidthMorph, b: &TempleWidthMorph, t: f32) -> TempleWidthMorph {
    let t = t.clamp(0.0, 1.0);
    TempleWidthMorph {
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
        let m = new_temple_width_morph();
        assert_eq!(m.width, 0.0);
        assert_eq!(m.prominence, 0.0);
    }

    #[test]
    fn test_set_width() {
        /* valid width */
        let mut m = new_temple_width_morph();
        temple_set_width(&mut m, 0.7);
        assert!((m.width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamp_high() {
        /* clamp high */
        let mut m = new_temple_width_morph();
        temple_set_width(&mut m, 3.0);
        assert_eq!(m.width, 1.0);
    }

    #[test]
    fn test_set_prominence() {
        /* valid prominence */
        let mut m = new_temple_width_morph();
        temple_set_prominence(&mut m, 0.4);
        assert!((m.prominence - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_set_prominence_clamp_low() {
        /* clamp low */
        let mut m = new_temple_width_morph();
        temple_set_prominence(&mut m, -1.0);
        assert_eq!(m.prominence, 0.0);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = TempleWidthMorph {
            width: 0.4,
            prominence: 0.8,
        };
        assert!((temple_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_wide_false() {
        /* default not wide */
        let m = new_temple_width_morph();
        assert!(!temple_is_wide(&m));
    }

    #[test]
    fn test_is_wide_true() {
        /* above 0.5 */
        let m = TempleWidthMorph {
            width: 0.9,
            prominence: 0.0,
        };
        assert!(temple_is_wide(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint blend */
        let a = TempleWidthMorph {
            width: 0.0,
            prominence: 0.0,
        };
        let b = TempleWidthMorph {
            width: 1.0,
            prominence: 1.0,
        };
        let c = temple_blend(&a, &b, 0.5);
        assert!((c.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let m = TempleWidthMorph {
            width: 0.3,
            prominence: 0.7,
        };
        let m2 = m.clone();
        assert_eq!(m.prominence, m2.prominence);
    }
}
