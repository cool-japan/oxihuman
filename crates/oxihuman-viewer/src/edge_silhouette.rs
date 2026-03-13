// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Edge silhouette — detects and renders silhouette edges for toon/NPR rendering.

/// Silhouette edge detection configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SilhouetteConfig {
    pub thickness: f32,
    pub color: [f32; 4],
    pub depth_threshold: f32,
    pub normal_threshold: f32,
    pub enabled: bool,
}

/// An edge pair representing a silhouette candidate.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SilhouetteEdge {
    pub v0: u32,
    pub v1: u32,
    pub face_a: u32,
    pub face_b: u32,
    pub is_silhouette: bool,
}

#[allow(dead_code)]
pub fn default_silhouette_config() -> SilhouetteConfig {
    SilhouetteConfig {
        thickness: 1.5,
        color: [0.0, 0.0, 0.0, 1.0],
        depth_threshold: 0.01,
        normal_threshold: 0.5,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn sil_set_thickness(cfg: &mut SilhouetteConfig, v: f32) {
    cfg.thickness = v.clamp(0.1, 10.0);
}

#[allow(dead_code)]
pub fn sil_set_color(cfg: &mut SilhouetteConfig, color: [f32; 4]) {
    cfg.color = color;
}

#[allow(dead_code)]
pub fn sil_set_enabled(cfg: &mut SilhouetteConfig, enabled: bool) {
    cfg.enabled = enabled;
}

#[allow(dead_code)]
pub fn sil_is_silhouette_edge(
    cfg: &SilhouetteConfig,
    normal_a: [f32; 3],
    normal_b: [f32; 3],
    view_dir: [f32; 3],
) -> bool {
    if !cfg.enabled {
        return false;
    }
    let dot_a = normal_a[0] * view_dir[0] + normal_a[1] * view_dir[1] + normal_a[2] * view_dir[2];
    let dot_b = normal_b[0] * view_dir[0] + normal_b[1] * view_dir[1] + normal_b[2] * view_dir[2];
    (dot_a >= 0.0) != (dot_b >= 0.0)
}

#[allow(dead_code)]
pub fn sil_detect_edges(
    cfg: &SilhouetteConfig,
    edges: &mut [SilhouetteEdge],
    face_normals: &[[f32; 3]],
    view_dir: [f32; 3],
) {
    for edge in edges.iter_mut() {
        let fa = edge.face_a as usize;
        let fb = edge.face_b as usize;
        if fa < face_normals.len() && fb < face_normals.len() {
            edge.is_silhouette =
                sil_is_silhouette_edge(cfg, face_normals[fa], face_normals[fb], view_dir);
        } else {
            edge.is_silhouette = false;
        }
    }
}

#[allow(dead_code)]
pub fn sil_count_silhouettes(edges: &[SilhouetteEdge]) -> usize {
    edges.iter().filter(|e| e.is_silhouette).count()
}

#[allow(dead_code)]
pub fn sil_reset(cfg: &mut SilhouetteConfig) {
    *cfg = default_silhouette_config();
}

#[allow(dead_code)]
pub fn sil_to_json(cfg: &SilhouetteConfig) -> String {
    format!(
        r#"{{"thickness":{:.4},"depth_threshold":{:.4},"normal_threshold":{:.4},"enabled":{}}}"#,
        cfg.thickness, cfg.depth_threshold, cfg.normal_threshold, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_enabled() {
        let cfg = default_silhouette_config();
        assert!(cfg.enabled);
    }

    #[test]
    fn set_thickness_clamps() {
        let mut cfg = default_silhouette_config();
        sil_set_thickness(&mut cfg, 100.0);
        assert!((cfg.thickness - 10.0).abs() < 1e-6);
    }

    #[test]
    fn silhouette_when_normals_straddle_view() {
        let cfg = default_silhouette_config();
        let na = [0.0f32, 0.0, 1.0];
        let nb = [0.0f32, 0.0, -1.0];
        let view = [0.0f32, 0.0, 1.0];
        assert!(sil_is_silhouette_edge(&cfg, na, nb, view));
    }

    #[test]
    fn no_silhouette_same_side() {
        let cfg = default_silhouette_config();
        let na = [0.0f32, 0.0, 1.0];
        let nb = [0.0f32, 1.0, 0.5];
        let view = [0.0f32, 0.0, 1.0];
        assert!(!sil_is_silhouette_edge(&cfg, na, nb, view));
    }

    #[test]
    fn disabled_never_silhouette() {
        let mut cfg = default_silhouette_config();
        sil_set_enabled(&mut cfg, false);
        let na = [0.0f32, 0.0, 1.0];
        let nb = [0.0f32, 0.0, -1.0];
        let view = [0.0f32, 0.0, 1.0];
        assert!(!sil_is_silhouette_edge(&cfg, na, nb, view));
    }

    #[test]
    fn count_silhouettes() {
        let edges = vec![
            SilhouetteEdge {
                v0: 0,
                v1: 1,
                face_a: 0,
                face_b: 1,
                is_silhouette: true,
            },
            SilhouetteEdge {
                v0: 1,
                v1: 2,
                face_a: 1,
                face_b: 2,
                is_silhouette: false,
            },
        ];
        assert_eq!(sil_count_silhouettes(&edges), 1);
    }

    #[test]
    fn detect_edges_updates_flags() {
        let cfg = default_silhouette_config();
        let mut edges = vec![SilhouetteEdge {
            v0: 0,
            v1: 1,
            face_a: 0,
            face_b: 1,
            is_silhouette: false,
        }];
        let normals = [[0.0f32, 0.0, 1.0], [0.0f32, 0.0, -1.0]];
        let view = [0.0f32, 0.0, 1.0];
        sil_detect_edges(&cfg, &mut edges, &normals, view);
        assert!(edges[0].is_silhouette);
    }

    #[test]
    fn reset_restores_defaults() {
        let mut cfg = default_silhouette_config();
        sil_set_thickness(&mut cfg, 5.0);
        sil_reset(&mut cfg);
        assert!((cfg.thickness - 1.5).abs() < 1e-6);
    }

    #[test]
    fn to_json_fields() {
        let cfg = default_silhouette_config();
        let j = sil_to_json(&cfg);
        assert!(j.contains("thickness"));
        assert!(j.contains("enabled"));
    }

    #[test]
    fn set_color() {
        let mut cfg = default_silhouette_config();
        sil_set_color(&mut cfg, [1.0, 0.0, 0.0, 1.0]);
        assert!((cfg.color[0] - 1.0).abs() < 1e-6);
    }
}
