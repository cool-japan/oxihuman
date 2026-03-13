// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct NasalSpineMorph {
    pub projection: f32,
    pub angulation: f32,
    pub prominence: f32,
    pub base_width: f32,
}

pub fn new_nasal_spine_morph() -> NasalSpineMorph {
    NasalSpineMorph {
        projection: 0.0,
        angulation: 0.0,
        prominence: 0.0,
        base_width: 0.0,
    }
}

pub fn nspine_set_projection(m: &mut NasalSpineMorph, v: f32) {
    m.projection = v.clamp(0.0, 1.0);
}

pub fn nspine_set_angulation(m: &mut NasalSpineMorph, v: f32) {
    m.angulation = v.clamp(0.0, 1.0);
}

pub fn nspine_set_prominence(m: &mut NasalSpineMorph, v: f32) {
    m.prominence = v.clamp(0.0, 1.0);
}

pub fn nspine_set_base_width(m: &mut NasalSpineMorph, v: f32) {
    m.base_width = v.clamp(0.0, 1.0);
}

pub fn nspine_volume_estimate(m: &NasalSpineMorph) -> f32 {
    m.projection * m.base_width
}

pub fn nasal_spine_morph_to_json(m: &NasalSpineMorph) -> String {
    format!(
        r#"{{"projection":{:.4},"angulation":{:.4},"prominence":{:.4},"base_width":{:.4}}}"#,
        m.projection, m.angulation, m.prominence, m.base_width
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all zero */
        let m = new_nasal_spine_morph();
        assert_eq!(m.projection, 0.0);
        assert_eq!(m.angulation, 0.0);
    }

    #[test]
    fn test_set_projection() {
        /* valid projection */
        let mut m = new_nasal_spine_morph();
        nspine_set_projection(&mut m, 0.6);
        assert!((m.projection - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_projection_clamp_high() {
        /* clamp high */
        let mut m = new_nasal_spine_morph();
        nspine_set_projection(&mut m, 5.0);
        assert_eq!(m.projection, 1.0);
    }

    #[test]
    fn test_set_angulation() {
        /* valid angulation */
        let mut m = new_nasal_spine_morph();
        nspine_set_angulation(&mut m, 0.4);
        assert!((m.angulation - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_set_prominence_clamp_low() {
        /* clamp low */
        let mut m = new_nasal_spine_morph();
        nspine_set_prominence(&mut m, -1.0);
        assert_eq!(m.prominence, 0.0);
    }

    #[test]
    fn test_set_base_width() {
        /* valid base width */
        let mut m = new_nasal_spine_morph();
        nspine_set_base_width(&mut m, 0.5);
        assert!((m.base_width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_volume_estimate() {
        /* volume is projection * base_width */
        let m = NasalSpineMorph {
            projection: 0.5,
            angulation: 0.0,
            prominence: 0.0,
            base_width: 0.4,
        };
        assert!((nspine_volume_estimate(&m) - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_volume_estimate_zero() {
        /* default volume is zero */
        let m = new_nasal_spine_morph();
        assert_eq!(nspine_volume_estimate(&m), 0.0);
    }

    #[test]
    fn test_to_json() {
        /* JSON has projection */
        let m = new_nasal_spine_morph();
        assert!(nasal_spine_morph_to_json(&m).contains("projection"));
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let m = NasalSpineMorph {
            projection: 0.3,
            angulation: 0.5,
            prominence: 0.7,
            base_width: 0.2,
        };
        let m2 = m.clone();
        assert_eq!(m.base_width, m2.base_width);
    }
}
