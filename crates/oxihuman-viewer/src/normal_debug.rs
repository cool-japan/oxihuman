// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Normal vector debug visualization: colored normals, face vs vertex normals, line display.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum NormalDisplayMode {
    FaceNormals,
    VertexNormals,
    TangentSpace,
    FlatColor,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NormalDebugConfig {
    pub mode: NormalDisplayMode,
    pub line_length: f32,
    pub line_color: [f32; 3],
    pub show_lines: bool,
    pub show_color_map: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NormalDebugLine {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub color: [f32; 3],
}

#[allow(dead_code)]
pub fn default_normal_debug_config() -> NormalDebugConfig {
    NormalDebugConfig {
        mode: NormalDisplayMode::VertexNormals,
        line_length: 0.05,
        line_color: [0.0, 0.5, 1.0],
        show_lines: true,
        show_color_map: true,
    }
}

#[allow(dead_code)]
pub fn normal_to_color(normal: [f32; 3]) -> [f32; 3] {
    [
        normal[0] * 0.5 + 0.5,
        normal[1] * 0.5 + 0.5,
        normal[2] * 0.5 + 0.5,
    ]
}

#[allow(dead_code)]
pub fn generate_normal_lines(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    length: f32,
) -> Vec<NormalDebugLine> {
    positions
        .iter()
        .zip(normals.iter())
        .map(|(p, n)| NormalDebugLine {
            start: *p,
            end: [
                p[0] + n[0] * length,
                p[1] + n[1] * length,
                p[2] + n[2] * length,
            ],
            color: normal_to_color(*n),
        })
        .collect()
}

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
    if len < 1e-8 {
        [0.0, 1.0, 0.0]
    } else {
        [cross[0] / len, cross[1] / len, cross[2] / len]
    }
}

#[allow(dead_code)]
pub fn validate_normal(n: [f32; 3]) -> bool {
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    (len - 1.0).abs() < 0.01
}

#[allow(dead_code)]
pub fn count_invalid_normals(normals: &[[f32; 3]]) -> usize {
    normals.iter().filter(|n| !validate_normal(**n)).count()
}

#[allow(dead_code)]
pub fn normal_debug_to_json(cfg: &NormalDebugConfig) -> String {
    let m = match &cfg.mode {
        NormalDisplayMode::FaceNormals => "face",
        NormalDisplayMode::VertexNormals => "vertex",
        NormalDisplayMode::TangentSpace => "tangent",
        NormalDisplayMode::FlatColor => "flat",
    };
    format!(
        r#"{{"mode":"{}","length":{},"lines":{},"color_map":{}}}"#,
        m, cfg.line_length, cfg.show_lines, cfg.show_color_map
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_normal_debug_config();
        assert_eq!(c.mode, NormalDisplayMode::VertexNormals);
    }

    #[test]
    fn test_normal_to_color() {
        let c = normal_to_color([0.0, 1.0, 0.0]);
        assert!((c[0] - 0.5).abs() < 1e-6);
        assert!((c[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_generate_lines() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let nrm = vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
        let lines = generate_normal_lines(&pos, &nrm, 0.1);
        assert_eq!(lines.len(), 2);
        assert!((lines[0].end[1] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_face_normal() {
        let n = compute_face_normal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        assert!((n[1] - (-1.0)).abs() < 1e-4 || (n[1] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_face_normal_degenerate() {
        let n = compute_face_normal([0.0; 3], [0.0; 3], [0.0; 3]);
        assert!((n[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_validate_normal_good() {
        assert!(validate_normal([0.0, 1.0, 0.0]));
    }

    #[test]
    fn test_validate_normal_bad() {
        assert!(!validate_normal([0.0, 0.0, 0.0]));
    }

    #[test]
    fn test_count_invalid() {
        let normals = vec![[0.0, 1.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        assert_eq!(count_invalid_normals(&normals), 1);
    }

    #[test]
    fn test_to_json() {
        let c = default_normal_debug_config();
        let j = normal_debug_to_json(&c);
        assert!(j.contains("vertex"));
    }

    #[test]
    fn test_generate_empty() {
        let lines = generate_normal_lines(&[], &[], 0.1);
        assert!(lines.is_empty());
    }
}
