// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct DensityView {
    pub enabled: bool,
    pub low_density_threshold: f32,
    pub high_density_threshold: f32,
}

pub fn new_density_view() -> DensityView {
    DensityView {
        enabled: false,
        low_density_threshold: 0.25,
        high_density_threshold: 0.75,
    }
}

pub fn dv_set_thresholds(v: &mut DensityView, low: f32, high: f32) {
    v.low_density_threshold = low.clamp(0.0, 1.0);
    v.high_density_threshold = high.clamp(0.0, 1.0);
}

pub fn dv_enable(v: &mut DensityView) {
    v.enabled = true;
}

pub fn dv_density_color(v: &DensityView, density: f32) -> [f32; 3] {
    if density < v.low_density_threshold {
        [0.0, 0.5, 1.0]
    } else if density > v.high_density_threshold {
        [1.0, 0.2, 0.0]
    } else {
        [0.2, 0.9, 0.2]
    }
}

pub fn dv_is_enabled(v: &DensityView) -> bool {
    v.enabled
}

pub fn dv_to_json(v: &DensityView) -> String {
    format!(
        r#"{{"enabled":{},"low":{:.4},"high":{:.4}}}"#,
        v.enabled, v.low_density_threshold, v.high_density_threshold
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* disabled with sensible thresholds */
        let v = new_density_view();
        assert!(!v.enabled);
        assert!((v.low_density_threshold - 0.25).abs() < 1e-6);
        assert!((v.high_density_threshold - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_thresholds() {
        /* thresholds stored */
        let mut v = new_density_view();
        dv_set_thresholds(&mut v, 0.2, 0.8);
        assert!((v.low_density_threshold - 0.2).abs() < 1e-6);
        assert!((v.high_density_threshold - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_thresholds_clamp() {
        /* clamp to [0,1] */
        let mut v = new_density_view();
        dv_set_thresholds(&mut v, -1.0, 5.0);
        assert_eq!(v.low_density_threshold, 0.0);
        assert_eq!(v.high_density_threshold, 1.0);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_density_view();
        dv_enable(&mut v);
        assert!(dv_is_enabled(&v));
    }

    #[test]
    fn test_density_color_low() {
        /* low density -> blue */
        let v = new_density_view();
        let c = dv_density_color(&v, 0.1);
        assert_eq!(c, [0.0, 0.5, 1.0]);
    }

    #[test]
    fn test_density_color_high() {
        /* high density -> orange */
        let v = new_density_view();
        let c = dv_density_color(&v, 0.9);
        assert_eq!(c, [1.0, 0.2, 0.0]);
    }

    #[test]
    fn test_density_color_mid() {
        /* mid density -> green */
        let v = new_density_view();
        let c = dv_density_color(&v, 0.5);
        assert_eq!(c, [0.2, 0.9, 0.2]);
    }

    #[test]
    fn test_to_json() {
        /* JSON has low threshold */
        let v = new_density_view();
        assert!(dv_to_json(&v).contains("low"));
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let v = new_density_view();
        let v2 = v.clone();
        assert_eq!(v.high_density_threshold, v2.high_density_threshold);
    }
}
