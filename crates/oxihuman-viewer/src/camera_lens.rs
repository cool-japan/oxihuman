// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Camera lens simulation (focal length, aperture, distortion).

use std::f32::consts::FRAC_PI_4;

/// Lens preset.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LensPreset {
    Wide,
    Normal,
    Telephoto,
    Macro,
}

/// Lens state.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct CameraLens {
    pub focal_length_mm: f32,
    pub aperture_fstop: f32,
    /// Barrel/pincushion distortion coefficient (-1..1, 0 = none).
    pub distortion_k1: f32,
    pub distortion_k2: f32,
    pub focus_distance_m: f32,
}

impl Default for CameraLens {
    fn default() -> Self {
        Self {
            focal_length_mm: 50.0,
            aperture_fstop: 8.0,
            distortion_k1: 0.0,
            distortion_k2: 0.0,
            focus_distance_m: 2.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_camera_lens() -> CameraLens {
    CameraLens::default()
}

#[allow(dead_code)]
pub fn cl_from_preset(preset: LensPreset) -> CameraLens {
    let mut l = CameraLens::default();
    match preset {
        LensPreset::Wide => {
            l.focal_length_mm = 24.0;
            l.distortion_k1 = -0.05;
        }
        LensPreset::Normal => {}
        LensPreset::Telephoto => {
            l.focal_length_mm = 200.0;
        }
        LensPreset::Macro => {
            l.focal_length_mm = 100.0;
            l.focus_distance_m = 0.3;
        }
    }
    l
}

#[allow(dead_code)]
pub fn cl_set_focal(lens: &mut CameraLens, mm: f32) {
    lens.focal_length_mm = mm.clamp(8.0, 800.0);
}

#[allow(dead_code)]
pub fn cl_set_aperture(lens: &mut CameraLens, fstop: f32) {
    lens.aperture_fstop = fstop.clamp(0.95, 22.0);
}

#[allow(dead_code)]
pub fn cl_set_focus(lens: &mut CameraLens, m: f32) {
    lens.focus_distance_m = m.max(0.1);
}

/// Vertical FOV from focal length (assuming 35mm equivalent, sensor height 24mm).
#[allow(dead_code)]
pub fn cl_fov_vertical_rad(lens: &CameraLens) -> f32 {
    2.0 * (12.0 / lens.focal_length_mm).atan()
}

/// Depth of field using thin lens formula (returns near/far distances in metres).
#[allow(dead_code)]
pub fn cl_dof_range(lens: &CameraLens, coc_mm: f32) -> (f32, f32) {
    let f = lens.focal_length_mm * 0.001;
    let n = lens.aperture_fstop;
    let d = lens.focus_distance_m;
    let c = coc_mm * 0.001;
    let hyp = f * f / (n * c);
    let near = (hyp * d - f * f) / (hyp - (d - f));
    let far = (hyp * d + f * f) / (hyp + (d - f));
    (near.max(0.01), far.max(near + 0.001))
}

/// Radial distortion correction factor at normalised radius r.
#[allow(dead_code)]
pub fn cl_distortion_factor(lens: &CameraLens, r: f32) -> f32 {
    let r2 = r * r;
    1.0 + lens.distortion_k1 * r2 + lens.distortion_k2 * r2 * r2
}

/// Reference angle (45 degrees) used in FOV calculations.
#[allow(dead_code)]
pub fn cl_reference_angle() -> f32 {
    FRAC_PI_4
}

#[allow(dead_code)]
pub fn cl_to_json(lens: &CameraLens) -> String {
    format!(
        "{{\"focal_mm\":{:.1},\"fstop\":{:.2},\"focus_m\":{:.3}}}",
        lens.focal_length_mm, lens.aperture_fstop, lens.focus_distance_m
    )
}

#[allow(dead_code)]
pub fn cl_blend(a: &CameraLens, b: &CameraLens, t: f32) -> CameraLens {
    let t = t.clamp(0.0, 1.0);
    CameraLens {
        focal_length_mm: a.focal_length_mm + (b.focal_length_mm - a.focal_length_mm) * t,
        aperture_fstop: a.aperture_fstop + (b.aperture_fstop - a.aperture_fstop) * t,
        distortion_k1: a.distortion_k1 + (b.distortion_k1 - a.distortion_k1) * t,
        distortion_k2: a.distortion_k2 + (b.distortion_k2 - a.distortion_k2) * t,
        focus_distance_m: a.focus_distance_m + (b.focus_distance_m - a.focus_distance_m) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_50mm() {
        assert!((new_camera_lens().focal_length_mm - 50.0).abs() < 1e-5);
    }

    #[test]
    fn focal_clamp_min() {
        let mut l = new_camera_lens();
        cl_set_focal(&mut l, 1.0);
        assert!(l.focal_length_mm >= 8.0);
    }

    #[test]
    fn focal_clamp_max() {
        let mut l = new_camera_lens();
        cl_set_focal(&mut l, 9999.0);
        assert!(l.focal_length_mm <= 800.0);
    }

    #[test]
    fn fov_positive() {
        assert!(cl_fov_vertical_rad(&new_camera_lens()) > 0.0);
    }

    #[test]
    fn dof_near_less_than_far() {
        let l = new_camera_lens();
        let (near, far) = cl_dof_range(&l, 0.03);
        assert!(near < far);
    }

    #[test]
    fn distortion_factor_one_at_zero_radius() {
        let l = new_camera_lens();
        assert!((cl_distortion_factor(&l, 0.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn preset_wide_focal() {
        let l = cl_from_preset(LensPreset::Wide);
        assert!(l.focal_length_mm < 35.0);
    }

    #[test]
    fn preset_telephoto_focal() {
        let l = cl_from_preset(LensPreset::Telephoto);
        assert!(l.focal_length_mm >= 100.0);
    }

    #[test]
    fn blend_midpoint() {
        let a = new_camera_lens();
        let mut b = new_camera_lens();
        cl_set_focal(&mut b, 100.0);
        let m = cl_blend(&a, &b, 0.5);
        assert!((m.focal_length_mm - 75.0).abs() < 1e-3);
    }

    #[test]
    fn json_has_focal() {
        assert!(cl_to_json(&new_camera_lens()).contains("focal_mm"));
    }
}
