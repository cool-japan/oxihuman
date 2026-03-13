// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct TangentFrameView {
    pub scale: f32,
    pub show_tangent: bool,
    pub show_bitangent: bool,
    pub show_normal: bool,
    pub enabled: bool,
}

pub fn new_tangent_frame_view() -> TangentFrameView {
    TangentFrameView {
        scale: 0.1,
        show_tangent: true,
        show_bitangent: true,
        show_normal: true,
        enabled: false,
    }
}

pub fn tfv_set_scale(v: &mut TangentFrameView, s: f32) {
    v.scale = s.clamp(0.001, 10.0);
}

pub fn tfv_enable(v: &mut TangentFrameView) {
    v.enabled = true;
}

pub fn tfv_toggle_tangent(v: &mut TangentFrameView) {
    v.show_tangent = !v.show_tangent;
}

pub fn tfv_toggle_normal(v: &mut TangentFrameView) {
    v.show_normal = !v.show_normal;
}

pub fn tfv_active_channel_count(v: &TangentFrameView) -> u32 {
    let mut n = 0u32;
    if v.show_tangent {
        n += 1;
    }
    if v.show_bitangent {
        n += 1;
    }
    if v.show_normal {
        n += 1;
    }
    n
}

pub fn tfv_to_json(v: &TangentFrameView) -> String {
    format!(
        r#"{{"scale":{:.4},"show_tangent":{},"show_bitangent":{},"show_normal":{},"enabled":{}}}"#,
        v.scale, v.show_tangent, v.show_bitangent, v.show_normal, v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* new view has all channels active */
        let v = new_tangent_frame_view();
        assert_eq!(tfv_active_channel_count(&v), 3);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_scale() {
        /* valid scale stored */
        let mut v = new_tangent_frame_view();
        tfv_set_scale(&mut v, 0.5);
        assert!((v.scale - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_scale_clamp_low() {
        /* below minimum clamped */
        let mut v = new_tangent_frame_view();
        tfv_set_scale(&mut v, -1.0);
        assert!(v.scale >= 0.001);
    }

    #[test]
    fn test_set_scale_clamp_high() {
        /* above maximum clamped */
        let mut v = new_tangent_frame_view();
        tfv_set_scale(&mut v, 100.0);
        assert!(v.scale <= 10.0);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_tangent_frame_view();
        tfv_enable(&mut v);
        assert!(v.enabled);
    }

    #[test]
    fn test_toggle_tangent() {
        /* toggle changes state */
        let mut v = new_tangent_frame_view();
        let initial = v.show_tangent;
        tfv_toggle_tangent(&mut v);
        assert_ne!(v.show_tangent, initial);
    }

    #[test]
    fn test_toggle_normal() {
        /* toggle changes state */
        let mut v = new_tangent_frame_view();
        let initial = v.show_normal;
        tfv_toggle_normal(&mut v);
        assert_ne!(v.show_normal, initial);
    }

    #[test]
    fn test_active_channel_count_after_toggle() {
        /* disabling tangent reduces count by 1 */
        let mut v = new_tangent_frame_view();
        tfv_toggle_tangent(&mut v);
        assert_eq!(tfv_active_channel_count(&v), 2);
    }

    #[test]
    fn test_to_json() {
        /* JSON has scale key */
        let v = new_tangent_frame_view();
        assert!(tfv_to_json(&v).contains("scale"));
    }
}
