// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct VctView {
    pub voxel_resolution: u32,
    pub cone_angle_deg: f32,
    pub max_distance: f32,
    pub step_multiplier: f32,
}

pub fn new_vct_view() -> VctView {
    VctView {
        voxel_resolution: 128,
        cone_angle_deg: 30.0,
        max_distance: 10.0,
        step_multiplier: 1.0,
    }
}

pub fn vct_set_resolution(v: &mut VctView, r: u32) {
    v.voxel_resolution = r.clamp(16, 512);
}

pub fn vct_cone_half_angle_rad(v: &VctView) -> f32 {
    use std::f32::consts::PI;
    v.cone_angle_deg * 0.5 * PI / 180.0
}

pub fn vct_is_high_resolution(v: &VctView) -> bool {
    v.voxel_resolution >= 256
}

pub fn vct_blend(a: &VctView, b: &VctView, t: f32) -> VctView {
    let t = t.clamp(0.0, 1.0);
    let res = (a.voxel_resolution as f32
        + (b.voxel_resolution as f32 - a.voxel_resolution as f32) * t)
        .round() as u32;
    VctView {
        voxel_resolution: res.clamp(16, 512),
        cone_angle_deg: a.cone_angle_deg + (b.cone_angle_deg - a.cone_angle_deg) * t,
        max_distance: a.max_distance + (b.max_distance - a.max_distance) * t,
        step_multiplier: a.step_multiplier + (b.step_multiplier - a.step_multiplier) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* default resolution */
        let v = new_vct_view();
        assert_eq!(v.voxel_resolution, 128);
    }

    #[test]
    fn test_set_resolution_clamped() {
        /* clamped to 512 */
        let mut v = new_vct_view();
        vct_set_resolution(&mut v, 1024);
        assert_eq!(v.voxel_resolution, 512);
    }

    #[test]
    fn test_cone_half_angle_positive() {
        /* half angle is positive */
        let v = new_vct_view();
        assert!(vct_cone_half_angle_rad(&v) > 0.0);
    }

    #[test]
    fn test_not_high_resolution_by_default() {
        /* 128 is not high resolution */
        let v = new_vct_view();
        assert!(!vct_is_high_resolution(&v));
    }

    #[test]
    fn test_blend() {
        /* midpoint cone angle */
        let a = VctView {
            voxel_resolution: 64,
            cone_angle_deg: 0.0,
            max_distance: 0.0,
            step_multiplier: 0.0,
        };
        let b = VctView {
            voxel_resolution: 64,
            cone_angle_deg: 60.0,
            max_distance: 20.0,
            step_multiplier: 2.0,
        };
        let c = vct_blend(&a, &b, 0.5);
        assert!((c.cone_angle_deg - 30.0).abs() < 1e-4);
    }
}
