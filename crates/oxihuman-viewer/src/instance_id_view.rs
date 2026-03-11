// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Instance ID colour coding visualization.
#[derive(Debug, Clone)]
pub struct InstanceIdView {
    pub enabled: bool,
    pub palette_size: u32,
    pub opacity: f32,
    /// If true, overlay the instance index as a tinted colour instead of solid.
    pub tint_mode: bool,
}

pub fn new_instance_id_view() -> InstanceIdView {
    InstanceIdView {
        enabled: false,
        palette_size: 64,
        opacity: 0.8,
        tint_mode: false,
    }
}

pub fn iidv_enable(v: &mut InstanceIdView) {
    v.enabled = true;
}

pub fn iidv_set_opacity(v: &mut InstanceIdView, o: f32) {
    v.opacity = o.clamp(0.0, 1.0);
}

pub fn iidv_set_palette_size(v: &mut InstanceIdView, n: u32) {
    v.palette_size = n.max(1);
}

pub fn iidv_set_tint_mode(v: &mut InstanceIdView, tint: bool) {
    v.tint_mode = tint;
}

/// Returns colour for the given instance ID.
pub fn iidv_color_for_id(v: &InstanceIdView, id: u32) -> [f32; 3] {
    let idx = id % v.palette_size;
    let hue = (idx as f32 / v.palette_size as f32) * 360.0;
    hsv_to_rgb_iid(hue, 0.7, 0.9)
}

fn hsv_to_rgb_iid(h: f32, s: f32, v_val: f32) -> [f32; 3] {
    let h = h % 360.0;
    let i = (h / 60.0) as u32;
    let f = h / 60.0 - i as f32;
    let p = v_val * (1.0 - s);
    let q = v_val * (1.0 - s * f);
    let t = v_val * (1.0 - s * (1.0 - f));
    match i {
        0 => [v_val, t, p],
        1 => [q, v_val, p],
        2 => [p, v_val, t],
        3 => [p, q, v_val],
        4 => [t, p, v_val],
        _ => [v_val, p, q],
    }
}

pub fn iidv_to_json(v: &InstanceIdView) -> String {
    format!(
        r#"{{"enabled":{},"palette_size":{},"opacity":{:.4},"tint_mode":{}}}"#,
        v.enabled, v.palette_size, v.opacity, v.tint_mode
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* palette=64, tint=false */
        let v = new_instance_id_view();
        assert_eq!(v.palette_size, 64);
        assert!(!v.tint_mode);
    }

    #[test]
    fn test_enable() {
        /* enable */
        let mut v = new_instance_id_view();
        iidv_enable(&mut v);
        assert!(v.enabled);
    }

    #[test]
    fn test_opacity() {
        /* opacity stored */
        let mut v = new_instance_id_view();
        iidv_set_opacity(&mut v, 0.6);
        assert!((v.opacity - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_opacity_clamp() {
        /* clamped */
        let mut v = new_instance_id_view();
        iidv_set_opacity(&mut v, 2.0);
        assert_eq!(v.opacity, 1.0);
    }

    #[test]
    fn test_tint_mode() {
        /* tint_mode toggle */
        let mut v = new_instance_id_view();
        iidv_set_tint_mode(&mut v, true);
        assert!(v.tint_mode);
    }

    #[test]
    fn test_color_range() {
        /* colour components in [0,1] */
        let v = new_instance_id_view();
        let c = iidv_color_for_id(&v, 10);
        for ch in c {
            assert!((0.0..=1.0).contains(&ch));
        }
    }

    #[test]
    fn test_color_wraps() {
        /* palette wraps */
        let v = new_instance_id_view();
        assert_eq!(
            iidv_color_for_id(&v, 0),
            iidv_color_for_id(&v, v.palette_size)
        );
    }

    #[test]
    fn test_palette_min() {
        /* palette min=1 */
        let mut v = new_instance_id_view();
        iidv_set_palette_size(&mut v, 0);
        assert_eq!(v.palette_size, 1);
    }

    #[test]
    fn test_to_json() {
        /* JSON has tint_mode */
        assert!(iidv_to_json(&new_instance_id_view()).contains("tint_mode"));
    }
}
