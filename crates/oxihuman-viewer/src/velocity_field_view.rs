// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct VelocityFieldView {
    pub enabled: bool,
    pub arrow_scale: f32,
    pub max_speed: f32,
    pub clamp_arrows: bool,
    pub grid_stride: usize,
}

pub fn new_velocity_field_view() -> VelocityFieldView {
    VelocityFieldView {
        enabled: false,
        arrow_scale: 1.0,
        max_speed: 10.0,
        clamp_arrows: true,
        grid_stride: 4,
    }
}

pub fn vfv_set_enabled(v: &mut VelocityFieldView, e: bool) {
    v.enabled = e;
}

pub fn vfv_set_arrow_scale(v: &mut VelocityFieldView, s: f32) {
    v.arrow_scale = s.max(0.0);
}

pub fn vfv_set_max_speed(v: &mut VelocityFieldView, s: f32) {
    v.max_speed = s.max(1e-6);
}

pub fn vfv_set_clamp_arrows(v: &mut VelocityFieldView, c: bool) {
    v.clamp_arrows = c;
}

pub fn vfv_set_grid_stride(v: &mut VelocityFieldView, n: usize) {
    v.grid_stride = n.max(1);
}

pub fn vfv_arrow_length(v: &VelocityFieldView, speed: f32) -> f32 {
    let ratio = speed / v.max_speed;
    let ratio = if v.clamp_arrows {
        ratio.clamp(0.0, 1.0)
    } else {
        ratio.max(0.0)
    };
    ratio * v.arrow_scale
}

pub fn velocity_field_view_to_json(v: &VelocityFieldView) -> String {
    format!(
        r#"{{"enabled":{},"arrow_scale":{:.4},"max_speed":{:.4},"clamp_arrows":{},"grid_stride":{}}}"#,
        v.enabled, v.arrow_scale, v.max_speed, v.clamp_arrows, v.grid_stride
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* disabled, scale=1, max_speed=10, stride=4 */
        let v = new_velocity_field_view();
        assert!(!v.enabled);
        assert!((v.arrow_scale - 1.0).abs() < 1e-6);
        assert!((v.max_speed - 10.0).abs() < 1e-6);
        assert_eq!(v.grid_stride, 4);
    }

    #[test]
    fn test_set_enabled() {
        /* enable and disable */
        let mut v = new_velocity_field_view();
        vfv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_set_arrow_scale() {
        /* valid scale */
        let mut v = new_velocity_field_view();
        vfv_set_arrow_scale(&mut v, 2.0);
        assert!((v.arrow_scale - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_arrow_scale_min() {
        /* minimum 0 */
        let mut v = new_velocity_field_view();
        vfv_set_arrow_scale(&mut v, -1.0);
        assert_eq!(v.arrow_scale, 0.0);
    }

    #[test]
    fn test_set_max_speed() {
        /* valid max speed */
        let mut v = new_velocity_field_view();
        vfv_set_max_speed(&mut v, 20.0);
        assert!((v.max_speed - 20.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_clamp_arrows() {
        /* flag set */
        let mut v = new_velocity_field_view();
        vfv_set_clamp_arrows(&mut v, false);
        assert!(!v.clamp_arrows);
    }

    #[test]
    fn test_set_grid_stride() {
        /* valid stride */
        let mut v = new_velocity_field_view();
        vfv_set_grid_stride(&mut v, 8);
        assert_eq!(v.grid_stride, 8);
    }

    #[test]
    fn test_arrow_length_at_max() {
        /* at max speed, length = arrow_scale */
        let v = new_velocity_field_view();
        let len = vfv_arrow_length(&v, 10.0);
        assert!((len - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_arrow_length_zero() {
        /* zero speed -> zero length */
        let v = new_velocity_field_view();
        let len = vfv_arrow_length(&v, 0.0);
        assert_eq!(len, 0.0);
    }

    #[test]
    fn test_to_json() {
        /* JSON has arrow_scale */
        let v = new_velocity_field_view();
        assert!(velocity_field_view_to_json(&v).contains("arrow_scale"));
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let v = new_velocity_field_view();
        let v2 = v.clone();
        assert_eq!(v.grid_stride, v2.grid_stride);
    }
}
