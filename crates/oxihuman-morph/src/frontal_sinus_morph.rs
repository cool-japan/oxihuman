// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Frontal sinus prominence morph.
#[derive(Debug, Clone)]
pub struct FrontalSinusMorph {
    /// Overall bossing prominence (0.0 = flat, 1.0 = maximum).
    pub bossing: f32,
    /// Lateral extent of the brow ridge (0.0 = narrow, 1.0 = wide).
    pub lateral_extent: f32,
    /// Superior slope steepness (0.0 = gradual, 1.0 = steep).
    pub slope: f32,
}

pub fn new_frontal_sinus_morph() -> FrontalSinusMorph {
    FrontalSinusMorph {
        bossing: 0.0,
        lateral_extent: 0.0,
        slope: 0.0,
    }
}

pub fn fsin_set_bossing(m: &mut FrontalSinusMorph, v: f32) {
    m.bossing = v.clamp(0.0, 1.0);
}

pub fn fsin_set_lateral_extent(m: &mut FrontalSinusMorph, v: f32) {
    m.lateral_extent = v.clamp(0.0, 1.0);
}

pub fn fsin_set_slope(m: &mut FrontalSinusMorph, v: f32) {
    m.slope = v.clamp(0.0, 1.0);
}

pub fn fsin_overall_weight(m: &FrontalSinusMorph) -> f32 {
    (m.bossing + m.lateral_extent + m.slope) / 3.0
}

pub fn fsin_blend(a: &FrontalSinusMorph, b: &FrontalSinusMorph, t: f32) -> FrontalSinusMorph {
    let t = t.clamp(0.0, 1.0);
    FrontalSinusMorph {
        bossing: a.bossing + (b.bossing - a.bossing) * t,
        lateral_extent: a.lateral_extent + (b.lateral_extent - a.lateral_extent) * t,
        slope: a.slope + (b.slope - a.slope) * t,
    }
}

pub fn fsin_is_neutral(m: &FrontalSinusMorph) -> bool {
    m.bossing < 1e-5 && m.lateral_extent < 1e-5 && m.slope < 1e-5
}

pub fn fsin_to_json(m: &FrontalSinusMorph) -> String {
    format!(
        r#"{{"bossing":{:.4},"lateral_extent":{:.4},"slope":{:.4}}}"#,
        m.bossing, m.lateral_extent, m.slope
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* all zero */
        let m = new_frontal_sinus_morph();
        assert_eq!(m.bossing, 0.0);
    }

    #[test]
    fn test_set_bossing() {
        /* valid value */
        let mut m = new_frontal_sinus_morph();
        fsin_set_bossing(&mut m, 0.7);
        assert!((m.bossing - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_high() {
        /* clamp above 1 */
        let mut m = new_frontal_sinus_morph();
        fsin_set_bossing(&mut m, 5.0);
        assert_eq!(m.bossing, 1.0);
    }

    #[test]
    fn test_clamp_low() {
        /* clamp below 0 */
        let mut m = new_frontal_sinus_morph();
        fsin_set_slope(&mut m, -1.0);
        assert_eq!(m.slope, 0.0);
    }

    #[test]
    fn test_is_neutral_true() {
        /* default neutral */
        assert!(fsin_is_neutral(&new_frontal_sinus_morph()));
    }

    #[test]
    fn test_is_neutral_false() {
        /* after setting bossing */
        let mut m = new_frontal_sinus_morph();
        fsin_set_bossing(&mut m, 0.1);
        assert!(!fsin_is_neutral(&m));
    }

    #[test]
    fn test_overall_weight() {
        /* 0.6+0.6+0.6 / 3 = 0.6 */
        let m = FrontalSinusMorph {
            bossing: 0.6,
            lateral_extent: 0.6,
            slope: 0.6,
        };
        assert!((fsin_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_blend_midpoint() {
        /* midpoint */
        let a = new_frontal_sinus_morph();
        let b = FrontalSinusMorph {
            bossing: 1.0,
            lateral_extent: 1.0,
            slope: 1.0,
        };
        let c = fsin_blend(&a, &b, 0.5);
        assert!((c.bossing - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        /* JSON has bossing */
        assert!(fsin_to_json(&new_frontal_sinus_morph()).contains("bossing"));
    }

    #[test]
    fn test_clone() {
        /* clone matches */
        let m = FrontalSinusMorph {
            bossing: 0.3,
            lateral_extent: 0.4,
            slope: 0.5,
        };
        let m2 = m.clone();
        assert_eq!(m.slope, m2.slope);
    }
}
