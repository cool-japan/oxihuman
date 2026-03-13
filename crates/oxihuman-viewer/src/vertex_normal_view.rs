// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Per-vertex normal visualization.

/// Per-vertex normal visualization config.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexNormalConfig {
    pub line_length: f32,
    pub color: [f32; 3],
    pub max_verts: usize,
    pub enabled: bool,
}

impl Default for VertexNormalConfig {
    fn default() -> Self {
        VertexNormalConfig {
            line_length: 0.05,
            color: [0.4, 1.0, 0.4],
            max_verts: 10_000,
            enabled: false,
        }
    }
}

#[allow(dead_code)]
pub fn default_vertex_normal_config() -> VertexNormalConfig {
    VertexNormalConfig::default()
}

#[allow(dead_code)]
pub fn vn_enable(cfg: &mut VertexNormalConfig) {
    cfg.enabled = true;
}

#[allow(dead_code)]
pub fn vn_disable(cfg: &mut VertexNormalConfig) {
    cfg.enabled = false;
}

#[allow(dead_code)]
pub fn vn_set_line_length(cfg: &mut VertexNormalConfig, v: f32) {
    cfg.line_length = v.clamp(0.001, 10.0);
}

#[allow(dead_code)]
pub fn vn_set_max_verts(cfg: &mut VertexNormalConfig, n: usize) {
    cfg.max_verts = n.max(1);
}

/// Normalize a 3D vector. Returns (0,1,0) for zero-length.
#[allow(dead_code)]
pub fn vn_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

/// Compute the end point of a normal line starting at position p.
#[allow(dead_code)]
pub fn vn_normal_endpoint(pos: [f32; 3], normal: [f32; 3], length: f32) -> [f32; 3] {
    let n = vn_normalize(normal);
    [
        pos[0] + n[0] * length,
        pos[1] + n[1] * length,
        pos[2] + n[2] * length,
    ]
}

/// Map normal direction to RGB color.
#[allow(dead_code)]
pub fn vn_normal_to_color(normal: [f32; 3]) -> [f32; 3] {
    let n = vn_normalize(normal);
    [(n[0] + 1.0) * 0.5, (n[1] + 1.0) * 0.5, (n[2] + 1.0) * 0.5]
}

/// Build line segment list for a set of vertices.
#[allow(dead_code)]
pub fn vn_build_lines(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    length: f32,
) -> Vec<[[f32; 3]; 2]> {
    positions
        .iter()
        .zip(normals.iter())
        .map(|(&pos, &n)| [pos, vn_normal_endpoint(pos, n, length)])
        .collect()
}

#[allow(dead_code)]
pub fn vn_to_json(cfg: &VertexNormalConfig) -> String {
    format!(
        r#"{{"line_length":{:.4},"max_verts":{},"enabled":{}}}"#,
        cfg.line_length, cfg.max_verts, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        assert!(!default_vertex_normal_config().enabled);
    }

    #[test]
    fn normalize_unit_vector() {
        let v = [1.0f32, 0.0, 0.0];
        let n = vn_normalize(v);
        assert!((n[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_zero_returns_up() {
        let v = [0.0f32, 0.0, 0.0];
        let n = vn_normalize(v);
        assert!((n[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn normal_endpoint_along_z() {
        let pos = [0.0f32, 0.0, 0.0];
        let n = [0.0f32, 0.0, 1.0];
        let end = vn_normal_endpoint(pos, n, 1.0);
        assert!((end[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn normal_to_color_up() {
        let c = vn_normal_to_color([0.0, 1.0, 0.0]);
        assert!((c[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn build_lines_count() {
        let pos = vec![[0.0f32; 3], [1.0f32, 0.0, 0.0]];
        let normals = vec![[0.0f32, 1.0, 0.0]; 2];
        let lines = vn_build_lines(&pos, &normals, 0.1);
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn set_line_length_clamps() {
        let mut cfg = default_vertex_normal_config();
        vn_set_line_length(&mut cfg, 0.0);
        assert!((cfg.line_length - 0.001).abs() < 1e-6);
    }

    #[test]
    fn enable_disable() {
        let mut cfg = default_vertex_normal_config();
        vn_enable(&mut cfg);
        assert!(cfg.enabled);
        vn_disable(&mut cfg);
        assert!(!cfg.enabled);
    }

    #[test]
    fn to_json_has_max_verts() {
        assert!(vn_to_json(&default_vertex_normal_config()).contains("max_verts"));
    }
}
