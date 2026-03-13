// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Face normal direction visualization.

/// Face normal view configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceNormalConfig {
    pub line_length: f32,
    pub color: [f32; 3],
    pub show_backfaces: bool,
    pub backface_color: [f32; 3],
    pub enabled: bool,
}

impl Default for FaceNormalConfig {
    fn default() -> Self {
        FaceNormalConfig {
            line_length: 0.05,
            color: [0.0, 0.7, 1.0],
            show_backfaces: true,
            backface_color: [1.0, 0.3, 0.0],
            enabled: false,
        }
    }
}

#[allow(dead_code)]
pub fn default_face_normal_config() -> FaceNormalConfig {
    FaceNormalConfig::default()
}

#[allow(dead_code)]
pub fn fn_enable(cfg: &mut FaceNormalConfig) {
    cfg.enabled = true;
}

#[allow(dead_code)]
pub fn fn_disable(cfg: &mut FaceNormalConfig) {
    cfg.enabled = false;
}

#[allow(dead_code)]
pub fn fn_set_line_length(cfg: &mut FaceNormalConfig, v: f32) {
    cfg.line_length = v.clamp(0.001, 10.0);
}

#[allow(dead_code)]
pub fn fn_set_color(cfg: &mut FaceNormalConfig, rgb: [f32; 3]) {
    cfg.color = rgb;
}

/// Compute face normal from three vertices.
#[allow(dead_code)]
pub fn compute_face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    if len < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        [cross[0] / len, cross[1] / len, cross[2] / len]
    }
}

/// Compute face centroid (average of three vertices).
#[allow(dead_code)]
pub fn face_centroid(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    [
        (a[0] + b[0] + c[0]) / 3.0,
        (a[1] + b[1] + c[1]) / 3.0,
        (a[2] + b[2] + c[2]) / 3.0,
    ]
}

/// Check if face is back-facing relative to view direction.
#[allow(dead_code)]
pub fn is_backface(normal: [f32; 3], view_dir: [f32; 3]) -> bool {
    let dot = normal[0] * view_dir[0] + normal[1] * view_dir[1] + normal[2] * view_dir[2];
    dot > 0.0
}

#[allow(dead_code)]
pub fn fn_to_json(cfg: &FaceNormalConfig) -> String {
    format!(
        r#"{{"line_length":{:.4},"show_backfaces":{},"enabled":{}}}"#,
        cfg.line_length, cfg.show_backfaces, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        assert!(!default_face_normal_config().enabled);
    }

    #[test]
    fn compute_normal_xy_plane() {
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0f32, 0.0, 0.0];
        let c = [0.0f32, 1.0, 0.0];
        let n = compute_face_normal(a, b, c);
        assert!(n[2].abs() > 0.99);
    }

    #[test]
    fn compute_normal_unit_length() {
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0f32, 0.0, 0.0];
        let c = [0.0f32, 1.0, 0.0];
        let n = compute_face_normal(a, b, c);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn centroid_of_equilateral() {
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0f32, 0.0, 0.0];
        let c = [0.5f32, 1.0, 0.0];
        let cen = face_centroid(a, b, c);
        assert!((cen[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn backface_detection() {
        let n = [0.0f32, 0.0, 1.0];
        let view = [0.0f32, 0.0, 1.0];
        assert!(is_backface(n, view));
    }

    #[test]
    fn frontface_detection() {
        let n = [0.0f32, 0.0, 1.0];
        let view = [0.0f32, 0.0, -1.0];
        assert!(!is_backface(n, view));
    }

    #[test]
    fn set_line_length_clamps() {
        let mut cfg = default_face_normal_config();
        fn_set_line_length(&mut cfg, 0.0);
        assert!((cfg.line_length - 0.001).abs() < 1e-6);
    }

    #[test]
    fn enable_disable() {
        let mut cfg = default_face_normal_config();
        fn_enable(&mut cfg);
        assert!(cfg.enabled);
        fn_disable(&mut cfg);
        assert!(!cfg.enabled);
    }

    #[test]
    fn to_json_has_line_length() {
        assert!(fn_to_json(&default_face_normal_config()).contains("line_length"));
    }
}
