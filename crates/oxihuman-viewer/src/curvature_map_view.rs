// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub enum CurvatureMode {
    Mean,
    Gaussian,
    Principal,
}

#[derive(Debug, Clone)]
pub struct CurvatureMapView {
    pub enabled: bool,
    pub mode: CurvatureMode,
    pub scale: f32,
}

pub fn new_curvature_map_view() -> CurvatureMapView {
    CurvatureMapView {
        enabled: false,
        mode: CurvatureMode::Mean,
        scale: 1.0,
    }
}

pub fn cmv_set_mode(v: &mut CurvatureMapView, mode: CurvatureMode) {
    v.mode = mode;
}

pub fn cmv_set_scale(v: &mut CurvatureMapView, s: f32) {
    v.scale = s.max(0.0);
}

pub fn cmv_enable(v: &mut CurvatureMapView) {
    v.enabled = true;
}

pub fn cmv_curvature_color(v: &CurvatureMapView, curvature: f32) -> [f32; 3] {
    let t = ((curvature * v.scale).tanh() * 0.5 + 0.5).clamp(0.0, 1.0);
    [t, 1.0 - t, (0.5 - (t - 0.5).abs()) * 2.0]
}

pub fn cmv_is_enabled(v: &CurvatureMapView) -> bool {
    v.enabled
}

pub fn cmv_mode_name(v: &CurvatureMapView) -> &'static str {
    match v.mode {
        CurvatureMode::Mean => "mean",
        CurvatureMode::Gaussian => "gaussian",
        CurvatureMode::Principal => "principal",
    }
}

pub fn cmv_to_json(v: &CurvatureMapView) -> String {
    format!(
        r#"{{"enabled":{},"mode":"{}","scale":{:.4}}}"#,
        v.enabled,
        cmv_mode_name(v),
        v.scale
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* disabled, mean mode, scale=1 */
        let v = new_curvature_map_view();
        assert!(!v.enabled);
        assert_eq!(v.mode, CurvatureMode::Mean);
        assert!((v.scale - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_mode() {
        /* mode stored */
        let mut v = new_curvature_map_view();
        cmv_set_mode(&mut v, CurvatureMode::Gaussian);
        assert_eq!(v.mode, CurvatureMode::Gaussian);
    }

    #[test]
    fn test_set_scale() {
        /* valid scale */
        let mut v = new_curvature_map_view();
        cmv_set_scale(&mut v, 2.5);
        assert!((v.scale - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_scale_min() {
        /* minimum 0 */
        let mut v = new_curvature_map_view();
        cmv_set_scale(&mut v, -1.0);
        assert_eq!(v.scale, 0.0);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_curvature_map_view();
        cmv_enable(&mut v);
        assert!(cmv_is_enabled(&v));
    }

    #[test]
    fn test_curvature_color_range() {
        /* all channels in [0,1] */
        let v = new_curvature_map_view();
        let c = cmv_curvature_color(&v, 0.5);
        for ch in c {
            assert!((0.0..=1.0).contains(&ch));
        }
    }

    #[test]
    fn test_mode_name() {
        /* mode names correct */
        let mut v = new_curvature_map_view();
        assert_eq!(cmv_mode_name(&v), "mean");
        cmv_set_mode(&mut v, CurvatureMode::Principal);
        assert_eq!(cmv_mode_name(&v), "principal");
    }

    #[test]
    fn test_to_json() {
        /* JSON has mode */
        let v = new_curvature_map_view();
        assert!(cmv_to_json(&v).contains("mode"));
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let v = new_curvature_map_view();
        let v2 = v.clone();
        assert_eq!(v.mode, v2.mode);
    }
}
