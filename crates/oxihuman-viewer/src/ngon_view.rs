// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! N-gon (non-quad/tri face) highlight visualization.

/// N-gon view configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NgonViewConfig {
    pub tri_color: [f32; 3],
    pub quad_color: [f32; 3],
    pub ngon_color: [f32; 3],
    pub highlight_ngons_only: bool,
    pub enabled: bool,
}

impl Default for NgonViewConfig {
    fn default() -> Self {
        NgonViewConfig {
            tri_color: [0.3, 0.6, 1.0],
            quad_color: [0.2, 0.8, 0.2],
            ngon_color: [1.0, 0.0, 0.5],
            highlight_ngons_only: true,
            enabled: false,
        }
    }
}

#[allow(dead_code)]
pub fn default_ngon_view_config() -> NgonViewConfig {
    NgonViewConfig::default()
}

#[allow(dead_code)]
pub fn ngv_enable(cfg: &mut NgonViewConfig) {
    cfg.enabled = true;
}

#[allow(dead_code)]
pub fn ngv_disable(cfg: &mut NgonViewConfig) {
    cfg.enabled = false;
}

#[allow(dead_code)]
pub fn ngv_set_highlight_ngons_only(cfg: &mut NgonViewConfig, v: bool) {
    cfg.highlight_ngons_only = v;
}

/// Face polygon type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FacePolyType {
    Triangle,
    Quad,
    Ngon,
    Degenerate,
}

/// Classify a face by vertex count.
#[allow(dead_code)]
pub fn ngv_classify(vertex_count: usize) -> FacePolyType {
    match vertex_count {
        0..=2 => FacePolyType::Degenerate,
        3 => FacePolyType::Triangle,
        4 => FacePolyType::Quad,
        _ => FacePolyType::Ngon,
    }
}

/// Get display color for a face.
#[allow(dead_code)]
pub fn ngv_face_color(cfg: &NgonViewConfig, poly_type: FacePolyType) -> [f32; 3] {
    match poly_type {
        FacePolyType::Triangle => cfg.tri_color,
        FacePolyType::Quad => cfg.quad_color,
        FacePolyType::Ngon => cfg.ngon_color,
        FacePolyType::Degenerate => [1.0, 0.0, 1.0],
    }
}

/// Count n-gon faces in a list of per-face vertex counts.
#[allow(dead_code)]
pub fn ngv_count_ngons(vert_counts: &[usize]) -> usize {
    vert_counts.iter().filter(|&&n| n > 4).count()
}

/// Count tri, quad, ngon faces.
#[allow(dead_code)]
pub fn ngv_face_stats(vert_counts: &[usize]) -> (usize, usize, usize) {
    let tris = vert_counts.iter().filter(|&&n| n == 3).count();
    let quads = vert_counts.iter().filter(|&&n| n == 4).count();
    let ngons = vert_counts.iter().filter(|&&n| n > 4).count();
    (tris, quads, ngons)
}

#[allow(dead_code)]
pub fn ngv_to_json(cfg: &NgonViewConfig) -> String {
    format!(
        r#"{{"highlight_ngons_only":{},"enabled":{}}}"#,
        cfg.highlight_ngons_only, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        assert!(!default_ngon_view_config().enabled);
    }

    #[test]
    fn classify_tri() {
        assert_eq!(ngv_classify(3), FacePolyType::Triangle);
    }

    #[test]
    fn classify_quad() {
        assert_eq!(ngv_classify(4), FacePolyType::Quad);
    }

    #[test]
    fn classify_ngon() {
        assert_eq!(ngv_classify(5), FacePolyType::Ngon);
    }

    #[test]
    fn classify_degenerate() {
        assert_eq!(ngv_classify(2), FacePolyType::Degenerate);
    }

    #[test]
    fn ngon_color_distinct() {
        let cfg = default_ngon_view_config();
        let c = ngv_face_color(&cfg, FacePolyType::Ngon);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn count_ngons() {
        let counts = vec![3, 4, 5, 6, 3, 4];
        assert_eq!(ngv_count_ngons(&counts), 2);
    }

    #[test]
    fn face_stats() {
        let counts = vec![3, 4, 5, 3, 4, 6];
        let (tris, quads, ngons) = ngv_face_stats(&counts);
        assert_eq!(tris, 2);
        assert_eq!(quads, 2);
        assert_eq!(ngons, 2);
    }

    #[test]
    fn enable_disable() {
        let mut cfg = default_ngon_view_config();
        ngv_enable(&mut cfg);
        assert!(cfg.enabled);
        ngv_disable(&mut cfg);
        assert!(!cfg.enabled);
    }

    #[test]
    fn to_json_has_enabled() {
        assert!(ngv_to_json(&default_ngon_view_config()).contains("enabled"));
    }
}
