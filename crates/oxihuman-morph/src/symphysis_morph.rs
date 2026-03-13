// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct SymphysisMorph {
    pub width: f32,
    pub height: f32,
    pub curvature: f32,
}

pub fn new_symphysis_morph() -> SymphysisMorph {
    SymphysisMorph {
        width: 0.0,
        height: 0.0,
        curvature: 0.0,
    }
}

pub fn sy_set_width(m: &mut SymphysisMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

pub fn sy_set_height(m: &mut SymphysisMorph, v: f32) {
    m.height = v.clamp(0.0, 1.0);
}

pub fn sy_set_curvature(m: &mut SymphysisMorph, v: f32) {
    m.curvature = v.clamp(0.0, 1.0);
}

pub fn sy_overall_weight(m: &SymphysisMorph) -> f32 {
    (m.width + m.height + m.curvature) / 3.0
}

pub fn sy_blend(a: &SymphysisMorph, b: &SymphysisMorph, t: f32) -> SymphysisMorph {
    let t = t.clamp(0.0, 1.0);
    SymphysisMorph {
        width: a.width + (b.width - a.width) * t,
        height: a.height + (b.height - a.height) * t,
        curvature: a.curvature + (b.curvature - a.curvature) * t,
    }
}

pub fn sy_is_wide(m: &SymphysisMorph) -> bool {
    m.width > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* zero init */
        let m = new_symphysis_morph();
        assert_eq!(m.width, 0.0);
        assert_eq!(m.height, 0.0);
        assert_eq!(m.curvature, 0.0);
    }

    #[test]
    fn test_set_width() {
        /* valid width */
        let mut m = new_symphysis_morph();
        sy_set_width(&mut m, 0.6);
        assert!((m.width - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamp_high() {
        /* clamp high */
        let mut m = new_symphysis_morph();
        sy_set_width(&mut m, 5.0);
        assert_eq!(m.width, 1.0);
    }

    #[test]
    fn test_set_height_clamp_low() {
        /* clamp low */
        let mut m = new_symphysis_morph();
        sy_set_height(&mut m, -0.5);
        assert_eq!(m.height, 0.0);
    }

    #[test]
    fn test_set_curvature() {
        /* valid curvature */
        let mut m = new_symphysis_morph();
        sy_set_curvature(&mut m, 0.7);
        assert!((m.curvature - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = SymphysisMorph {
            width: 0.3,
            height: 0.6,
            curvature: 0.9,
        };
        assert!((sy_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_wide_false() {
        /* default not wide */
        let m = new_symphysis_morph();
        assert!(!sy_is_wide(&m));
    }

    #[test]
    fn test_is_wide_true() {
        /* above 0.5 */
        let m = SymphysisMorph {
            width: 0.8,
            height: 0.0,
            curvature: 0.0,
        };
        assert!(sy_is_wide(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint blend */
        let a = SymphysisMorph {
            width: 0.0,
            height: 0.0,
            curvature: 0.0,
        };
        let b = SymphysisMorph {
            width: 1.0,
            height: 1.0,
            curvature: 1.0,
        };
        let c = sy_blend(&a, &b, 0.5);
        assert!((c.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let m = SymphysisMorph {
            width: 0.2,
            height: 0.5,
            curvature: 0.8,
        };
        let m2 = m.clone();
        assert_eq!(m.curvature, m2.curvature);
    }
}
