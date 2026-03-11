// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Shading normals debug visualization.
#[derive(Debug, Clone)]
pub struct ShadingNormalView {
    pub enabled: bool,
    /// Arrow scale factor (length of each normal line in world units).
    pub scale: f32,
    /// Colour for outward normals.
    pub color: [f32; 3],
    /// Show face normals instead of vertex normals.
    pub show_face_normals: bool,
}

pub fn new_shading_normal_view() -> ShadingNormalView {
    ShadingNormalView {
        enabled: false,
        scale: 0.02,
        color: [0.0, 1.0, 0.5],
        show_face_normals: false,
    }
}

pub fn snv_enable(v: &mut ShadingNormalView) {
    v.enabled = true;
}

pub fn snv_set_scale(v: &mut ShadingNormalView, s: f32) {
    v.scale = s.max(1e-4);
}

pub fn snv_set_show_face_normals(v: &mut ShadingNormalView, show: bool) {
    v.show_face_normals = show;
}

/// Compute the endpoint of a normal line given origin and normal direction.
pub fn snv_normal_endpoint(origin: [f32; 3], normal: [f32; 3], scale: f32) -> [f32; 3] {
    [
        origin[0] + normal[0] * scale,
        origin[1] + normal[1] * scale,
        origin[2] + normal[2] * scale,
    ]
}

pub fn snv_normal_facing(normal: [f32; 3]) -> bool {
    /* facing camera if z-component > 0 in view space */
    normal[2] > 0.0
}

pub fn snv_to_json(v: &ShadingNormalView) -> String {
    format!(
        r#"{{"enabled":{},"scale":{:.4},"show_face_normals":{}}}"#,
        v.enabled, v.scale, v.show_face_normals
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* scale=0.02, enabled=false */
        let v = new_shading_normal_view();
        assert!((v.scale - 0.02).abs() < 1e-6);
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable() {
        /* enable flag */
        let mut v = new_shading_normal_view();
        snv_enable(&mut v);
        assert!(v.enabled);
    }

    #[test]
    fn test_set_scale() {
        /* valid scale */
        let mut v = new_shading_normal_view();
        snv_set_scale(&mut v, 0.1);
        assert!((v.scale - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_scale_min() {
        /* scale cannot be negative */
        let mut v = new_shading_normal_view();
        snv_set_scale(&mut v, -1.0);
        assert!(v.scale > 0.0);
    }

    #[test]
    fn test_endpoint() {
        /* endpoint offsets by scale in normal direction */
        let ep = snv_normal_endpoint([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.1);
        assert!((ep[1] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_facing_positive() {
        /* z > 0 is facing */
        assert!(snv_normal_facing([0.0, 0.0, 1.0]));
    }

    #[test]
    fn test_facing_negative() {
        /* z < 0 is not facing */
        assert!(!snv_normal_facing([0.0, 0.0, -1.0]));
    }

    #[test]
    fn test_show_face_normals() {
        /* toggle face normals */
        let mut v = new_shading_normal_view();
        snv_set_show_face_normals(&mut v, true);
        assert!(v.show_face_normals);
    }

    #[test]
    fn test_to_json() {
        /* JSON has scale */
        assert!(snv_to_json(&new_shading_normal_view()).contains("scale"));
    }
}
