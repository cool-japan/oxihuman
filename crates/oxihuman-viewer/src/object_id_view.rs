// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Object / mesh ID colour coding visualization.
#[derive(Debug, Clone)]
pub struct ObjectIdView {
    pub enabled: bool,
    pub palette_size: u32,
    pub opacity: f32,
}

pub fn new_object_id_view() -> ObjectIdView {
    ObjectIdView {
        enabled: false,
        palette_size: 32,
        opacity: 1.0,
    }
}

pub fn oidv_enable(v: &mut ObjectIdView) {
    v.enabled = true;
}

pub fn oidv_set_opacity(v: &mut ObjectIdView, o: f32) {
    v.opacity = o.clamp(0.0, 1.0);
}

pub fn oidv_set_palette_size(v: &mut ObjectIdView, n: u32) {
    v.palette_size = n.max(1);
}

/// Returns colour for the given object ID (cycles the palette).
pub fn oidv_color_for_id(v: &ObjectIdView, id: u32) -> [f32; 3] {
    let idx = id % v.palette_size;
    let hue = (idx as f32 / v.palette_size as f32) * 360.0;
    hsv_to_rgb_simple(hue, 0.75, 0.95)
}

fn hsv_to_rgb_simple(h: f32, s: f32, v_val: f32) -> [f32; 3] {
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

pub fn oidv_to_json(v: &ObjectIdView) -> String {
    format!(
        r#"{{"enabled":{},"palette_size":{},"opacity":{:.4}}}"#,
        v.enabled, v.palette_size, v.opacity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* palette=32 */
        let v = new_object_id_view();
        assert_eq!(v.palette_size, 32);
    }

    #[test]
    fn test_enable() {
        /* enable */
        let mut v = new_object_id_view();
        oidv_enable(&mut v);
        assert!(v.enabled);
    }

    #[test]
    fn test_opacity_set() {
        /* opacity stored */
        let mut v = new_object_id_view();
        oidv_set_opacity(&mut v, 0.8);
        assert!((v.opacity - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_opacity_clamp() {
        /* clamp */
        let mut v = new_object_id_view();
        oidv_set_opacity(&mut v, 5.0);
        assert_eq!(v.opacity, 1.0);
    }

    #[test]
    fn test_color_valid_range() {
        /* colour components in [0,1] */
        let v = new_object_id_view();
        let c = oidv_color_for_id(&v, 5);
        for ch in c {
            assert!((0.0..=1.0).contains(&ch));
        }
    }

    #[test]
    fn test_color_different_ids() {
        /* different IDs different colours */
        let v = new_object_id_view();
        assert_ne!(oidv_color_for_id(&v, 0), oidv_color_for_id(&v, 1));
    }

    #[test]
    fn test_color_wraps() {
        /* wraps palette */
        let v = new_object_id_view();
        assert_eq!(
            oidv_color_for_id(&v, 0),
            oidv_color_for_id(&v, v.palette_size)
        );
    }

    #[test]
    fn test_palette_min() {
        /* min palette=1 */
        let mut v = new_object_id_view();
        oidv_set_palette_size(&mut v, 0);
        assert_eq!(v.palette_size, 1);
    }

    #[test]
    fn test_to_json() {
        /* JSON has palette_size */
        assert!(oidv_to_json(&new_object_id_view()).contains("palette_size"));
    }
}
