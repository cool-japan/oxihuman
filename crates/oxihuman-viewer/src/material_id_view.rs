// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Material ID colour coding visualization.
#[derive(Debug, Clone)]
pub struct MaterialIdView {
    pub enabled: bool,
    /// Number of distinct material IDs to colour-cycle through.
    pub palette_size: u32,
    /// Overall opacity.
    pub opacity: f32,
}

pub fn new_material_id_view() -> MaterialIdView {
    MaterialIdView {
        enabled: false,
        palette_size: 16,
        opacity: 1.0,
    }
}

pub fn midv_enable(v: &mut MaterialIdView) {
    v.enabled = true;
}

pub fn midv_set_opacity(v: &mut MaterialIdView, o: f32) {
    v.opacity = o.clamp(0.0, 1.0);
}

pub fn midv_set_palette_size(v: &mut MaterialIdView, n: u32) {
    v.palette_size = n.max(1);
}

/// Returns a deterministic colour for the given material ID.
pub fn midv_color_for_id(v: &MaterialIdView, id: u32) -> [f32; 3] {
    let idx = id % v.palette_size;
    let h = (idx as f32 / v.palette_size as f32) * 360.0;
    hsv_to_rgb(h, 0.8, 0.9)
}

fn hsv_to_rgb(h: f32, s: f32, v_val: f32) -> [f32; 3] {
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

pub fn midv_to_json(v: &MaterialIdView) -> String {
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
        /* palette_size=16, opacity=1 */
        let v = new_material_id_view();
        assert_eq!(v.palette_size, 16);
        assert!((v.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_enable() {
        /* enable */
        let mut v = new_material_id_view();
        midv_enable(&mut v);
        assert!(v.enabled);
    }

    #[test]
    fn test_set_opacity() {
        /* valid opacity */
        let mut v = new_material_id_view();
        midv_set_opacity(&mut v, 0.5);
        assert!((v.opacity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_opacity_clamp() {
        /* clamped */
        let mut v = new_material_id_view();
        midv_set_opacity(&mut v, 2.0);
        assert_eq!(v.opacity, 1.0);
    }

    #[test]
    fn test_color_id_0() {
        /* id 0 has valid colour */
        let v = new_material_id_view();
        let c = midv_color_for_id(&v, 0);
        for ch in c {
            assert!((0.0..=1.0).contains(&ch));
        }
    }

    #[test]
    fn test_color_ids_differ() {
        /* different IDs give different colours */
        let v = new_material_id_view();
        let c0 = midv_color_for_id(&v, 0);
        let c1 = midv_color_for_id(&v, 1);
        assert_ne!(c0, c1);
    }

    #[test]
    fn test_color_wraps_palette() {
        /* ID >= palette_size wraps */
        let v = new_material_id_view();
        let c0 = midv_color_for_id(&v, 0);
        let c16 = midv_color_for_id(&v, v.palette_size);
        assert_eq!(c0, c16);
    }

    #[test]
    fn test_palette_size_min() {
        /* palette_size min=1 */
        let mut v = new_material_id_view();
        midv_set_palette_size(&mut v, 0);
        assert_eq!(v.palette_size, 1);
    }

    #[test]
    fn test_to_json() {
        /* JSON has palette_size */
        assert!(midv_to_json(&new_material_id_view()).contains("palette_size"));
    }
}
