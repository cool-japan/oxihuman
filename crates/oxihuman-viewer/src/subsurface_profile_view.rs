// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Subsurface scattering profile debug view.

/// SSS profile debug state.
#[derive(Debug, Clone)]
pub struct SubsurfaceProfileView {
    pub radius: [f32; 3],
    pub scale: f32,
    pub ior: f32,
    pub anisotropy: f32,
    pub show_profile: bool,
}

impl Default for SubsurfaceProfileView {
    fn default() -> Self {
        Self {
            radius: [1.0, 0.2, 0.1],
            scale: 0.1,
            ior: 1.4,
            anisotropy: 0.0,
            show_profile: false,
        }
    }
}

/// Create a new SubsurfaceProfileView.
pub fn new_subsurface_profile_view() -> SubsurfaceProfileView {
    SubsurfaceProfileView::default()
}

/// Set SSS radius per RGB channel.
pub fn sss_profile_set_radius(view: &mut SubsurfaceProfileView, r: f32, g: f32, b: f32) {
    view.radius[0] = r.max(0.0);
    view.radius[1] = g.max(0.0);
    view.radius[2] = b.max(0.0);
}

/// Set SSS scale factor.
pub fn sss_profile_set_scale(view: &mut SubsurfaceProfileView, v: f32) {
    view.scale = v.clamp(0.0, 10.0);
}

/// Set index of refraction.
pub fn sss_profile_set_ior(view: &mut SubsurfaceProfileView, v: f32) {
    view.ior = v.clamp(1.0, 3.0);
}

/// Toggle profile visualization overlay.
pub fn sss_profile_show(view: &mut SubsurfaceProfileView, show: bool) {
    view.show_profile = show;
}

/// Serialize to JSON.
pub fn sss_profile_to_json(view: &SubsurfaceProfileView) -> String {
    format!(
        r#"{{"scale":{},"ior":{},"anisotropy":{},"show_profile":{}}}"#,
        view.scale, view.ior, view.anisotropy, view.show_profile,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let v = new_subsurface_profile_view();
        assert!((v.scale - 0.1).abs() < 1e-6 /* default scale */);
    }

    #[test]
    fn test_radius_set() {
        let mut v = new_subsurface_profile_view();
        sss_profile_set_radius(&mut v, 2.0, 0.5, 0.3);
        assert!((v.radius[0] - 2.0).abs() < 1e-6 /* R stored */);
    }

    #[test]
    fn test_radius_negative_clamp() {
        let mut v = new_subsurface_profile_view();
        sss_profile_set_radius(&mut v, -1.0, 0.5, 0.3);
        assert!((v.radius[0] - 0.0).abs() < 1e-6 /* clamped to 0 */);
    }

    #[test]
    fn test_scale_clamp_high() {
        let mut v = new_subsurface_profile_view();
        sss_profile_set_scale(&mut v, 100.0);
        assert!((v.scale - 10.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_ior_clamp() {
        let mut v = new_subsurface_profile_view();
        sss_profile_set_ior(&mut v, 5.0);
        assert!((v.ior - 3.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_ior_min() {
        let mut v = new_subsurface_profile_view();
        sss_profile_set_ior(&mut v, 0.5);
        assert!((v.ior - 1.0).abs() < 1e-6 /* min 1.0 */);
    }

    #[test]
    fn test_show_profile() {
        let mut v = new_subsurface_profile_view();
        sss_profile_show(&mut v, true);
        assert!(v.show_profile /* enabled */);
    }

    #[test]
    fn test_json_keys() {
        let v = new_subsurface_profile_view();
        let j = sss_profile_to_json(&v);
        assert!(j.contains("ior") /* key */);
    }

    #[test]
    fn test_default_anisotropy() {
        let v = SubsurfaceProfileView::default();
        assert!((v.anisotropy - 0.0).abs() < 1e-6 /* zero */);
    }
}
