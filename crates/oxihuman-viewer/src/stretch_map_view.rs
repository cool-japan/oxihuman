// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct StretchMapView {
    pub enabled: bool,
    pub warn_threshold: f32,
    pub error_threshold: f32,
}

pub fn new_stretch_map_view() -> StretchMapView {
    StretchMapView {
        enabled: false,
        warn_threshold: 0.2,
        error_threshold: 0.5,
    }
}

pub fn smv_set_thresholds(v: &mut StretchMapView, warn: f32, error: f32) {
    v.warn_threshold = warn.clamp(0.0, 1.0);
    v.error_threshold = error.clamp(0.0, 1.0);
}

pub fn smv_enable(v: &mut StretchMapView) {
    v.enabled = true;
}

pub fn smv_stretch_color(v: &StretchMapView, stretch: f32) -> [f32; 3] {
    if stretch > v.error_threshold {
        [1.0, 0.0, 0.0]
    } else if stretch > v.warn_threshold {
        [1.0, 1.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    }
}

pub fn smv_is_enabled(v: &StretchMapView) -> bool {
    v.enabled
}

pub fn smv_to_json(v: &StretchMapView) -> String {
    format!(
        r#"{{"enabled":{},"warn_threshold":{:.4},"error_threshold":{:.4}}}"#,
        v.enabled, v.warn_threshold, v.error_threshold
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* disabled with sensible thresholds */
        let v = new_stretch_map_view();
        assert!(!v.enabled);
        assert!((v.warn_threshold - 0.2).abs() < 1e-6);
        assert!((v.error_threshold - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_thresholds() {
        /* thresholds stored */
        let mut v = new_stretch_map_view();
        smv_set_thresholds(&mut v, 0.3, 0.7);
        assert!((v.warn_threshold - 0.3).abs() < 1e-6);
        assert!((v.error_threshold - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_thresholds_clamp() {
        /* clamp to [0,1] */
        let mut v = new_stretch_map_view();
        smv_set_thresholds(&mut v, -0.5, 2.0);
        assert_eq!(v.warn_threshold, 0.0);
        assert_eq!(v.error_threshold, 1.0);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_stretch_map_view();
        smv_enable(&mut v);
        assert!(smv_is_enabled(&v));
    }

    #[test]
    fn test_stretch_color_ok() {
        /* below warn -> green */
        let v = new_stretch_map_view();
        let c = smv_stretch_color(&v, 0.1);
        assert_eq!(c, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_stretch_color_warn() {
        /* above warn, below error -> yellow */
        let v = new_stretch_map_view();
        let c = smv_stretch_color(&v, 0.3);
        assert_eq!(c, [1.0, 1.0, 0.0]);
    }

    #[test]
    fn test_stretch_color_error() {
        /* above error -> red */
        let v = new_stretch_map_view();
        let c = smv_stretch_color(&v, 0.6);
        assert_eq!(c, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_to_json() {
        /* JSON has warn_threshold */
        let v = new_stretch_map_view();
        assert!(smv_to_json(&v).contains("warn_threshold"));
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let v = new_stretch_map_view();
        let v2 = v.clone();
        assert_eq!(v.error_threshold, v2.error_threshold);
    }
}
