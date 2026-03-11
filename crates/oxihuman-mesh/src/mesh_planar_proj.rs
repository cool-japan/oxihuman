// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Planar UV projection onto a user-defined plane.
#[allow(dead_code)]
pub struct PlanarProjConfig {
    pub origin: [f32; 3],
    pub normal: [f32; 3],
    pub u_axis: [f32; 3],
    pub scale: f32,
}

#[allow(dead_code)]
pub struct PlanarProjResult {
    pub uvs: Vec<[f32; 2]>,
    pub config: PlanarProjConfig,
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        return [0.0, 1.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

#[allow(dead_code)]
pub fn default_planar_proj_config() -> PlanarProjConfig {
    PlanarProjConfig {
        origin: [0.0; 3],
        normal: [0.0, 1.0, 0.0],
        u_axis: [1.0, 0.0, 0.0],
        scale: 1.0,
    }
}

#[allow(dead_code)]
pub fn project_planar(positions: &[[f32; 3]], cfg: PlanarProjConfig) -> PlanarProjResult {
    let n = normalize3(cfg.normal);
    let u = normalize3(cfg.u_axis);
    let v = normalize3(cross3(n, u));

    let uvs: Vec<[f32; 2]> = positions
        .iter()
        .map(|&p| {
            let rel = [
                p[0] - cfg.origin[0],
                p[1] - cfg.origin[1],
                p[2] - cfg.origin[2],
            ];
            let u_coord = dot3(rel, u) / cfg.scale;
            let v_coord = dot3(rel, v) / cfg.scale;
            [u_coord, v_coord]
        })
        .collect();

    PlanarProjResult { uvs, config: cfg }
}

#[allow(dead_code)]
pub fn planar_proj_vertex_count(r: &PlanarProjResult) -> usize {
    r.uvs.len()
}

#[allow(dead_code)]
pub fn planar_uv_bounds(r: &PlanarProjResult) -> ([f32; 2], [f32; 2]) {
    if r.uvs.is_empty() {
        return ([0.0; 2], [0.0; 2]);
    }
    let mut mn = r.uvs[0];
    let mut mx = r.uvs[0];
    for &uv in &r.uvs {
        if uv[0] < mn[0] {
            mn[0] = uv[0];
        }
        if uv[1] < mn[1] {
            mn[1] = uv[1];
        }
        if uv[0] > mx[0] {
            mx[0] = uv[0];
        }
        if uv[1] > mx[1] {
            mx[1] = uv[1];
        }
    }
    (mn, mx)
}

#[allow(dead_code)]
pub fn planar_proj_to_json(r: &PlanarProjResult) -> String {
    format!("{{\"vertex_count\":{}}}", r.uvs.len())
}

#[allow(dead_code)]
pub fn normalize_planar_uvs(r: &mut PlanarProjResult) {
    let (mn, mx) = planar_uv_bounds(r);
    let rw = (mx[0] - mn[0]).max(1e-10);
    let rh = (mx[1] - mn[1]).max(1e-10);
    for uv in &mut r.uvs {
        uv[0] = (uv[0] - mn[0]) / rw;
        uv[1] = (uv[1] - mn[1]) / rh;
    }
}

#[allow(dead_code)]
pub fn planar_proj_tilted(positions: &[[f32; 3]], angle_deg: f32) -> PlanarProjResult {
    let angle = angle_deg * PI / 180.0;
    let cfg = PlanarProjConfig {
        origin: [0.0; 3],
        normal: [angle.sin(), angle.cos(), 0.0],
        u_axis: [1.0, 0.0, 0.0],
        scale: 1.0,
    };
    project_planar(positions, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_quad() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ]
    }

    #[test]
    fn test_xz_plane_uv_count() {
        let cfg = PlanarProjConfig {
            origin: [0.0; 3],
            normal: [0.0, 1.0, 0.0],
            u_axis: [1.0, 0.0, 0.0],
            scale: 1.0,
        };
        let r = project_planar(&flat_quad(), cfg);
        assert_eq!(r.uvs.len(), 4);
    }

    #[test]
    fn test_scale_affects_uvs() {
        let pos = vec![[1.0, 0.0, 0.0]];
        let cfg1 = PlanarProjConfig {
            origin: [0.0; 3],
            normal: [0.0, 1.0, 0.0],
            u_axis: [1.0, 0.0, 0.0],
            scale: 1.0,
        };
        let cfg2 = PlanarProjConfig {
            origin: [0.0; 3],
            normal: [0.0, 1.0, 0.0],
            u_axis: [1.0, 0.0, 0.0],
            scale: 2.0,
        };
        let r1 = project_planar(&pos, cfg1);
        let r2 = project_planar(&pos, cfg2);
        assert!((r1.uvs[0][0] - 2.0 * r2.uvs[0][0]).abs() < 1e-5);
    }

    #[test]
    fn test_empty_positions() {
        let cfg = default_planar_proj_config();
        let r = project_planar(&[], cfg);
        assert_eq!(r.uvs.len(), 0);
    }

    #[test]
    fn test_uv_bounds() {
        let cfg = default_planar_proj_config();
        let r = project_planar(&flat_quad(), cfg);
        let (mn, mx) = planar_uv_bounds(&r);
        assert!(mx[0] > mn[0]);
    }

    #[test]
    fn test_normalize_uvs() {
        let cfg = default_planar_proj_config();
        let mut r = project_planar(&flat_quad(), cfg);
        normalize_planar_uvs(&mut r);
        let (mn, mx) = planar_uv_bounds(&r);
        assert!((mn[0]).abs() < 1e-5);
        assert!((mx[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_tilted_projection() {
        let pos = flat_quad();
        let r = planar_proj_tilted(&pos, 45.0);
        assert_eq!(r.uvs.len(), 4);
    }

    #[test]
    fn test_to_json() {
        let cfg = default_planar_proj_config();
        let r = project_planar(&flat_quad(), cfg);
        let j = planar_proj_to_json(&r);
        assert!(j.contains("vertex_count"));
    }

    #[test]
    fn test_vertex_count_fn() {
        let cfg = default_planar_proj_config();
        let r = project_planar(&flat_quad(), cfg);
        assert_eq!(planar_proj_vertex_count(&r), 4);
    }

    #[test]
    fn test_uvs_finite() {
        let cfg = default_planar_proj_config();
        let r = project_planar(&flat_quad(), cfg);
        for &uv in &r.uvs {
            assert!(uv[0].is_finite());
            assert!(uv[1].is_finite());
        }
    }

    #[test]
    fn test_default_config() {
        let cfg = default_planar_proj_config();
        assert!((cfg.scale - 1.0).abs() < 1e-6);
    }
}
