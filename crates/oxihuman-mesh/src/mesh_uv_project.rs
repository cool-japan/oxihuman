// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! UV projection methods: planar, cylindrical, spherical, and box mapping.

// ── Enums ────────────────────────────────────────────────────────────────────

/// UV projection mode selector.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UvProjectMode {
    Planar,
    Cylindrical,
    Spherical,
    Box,
    Camera,
}

// ── Structs ──────────────────────────────────────────────────────────────────

/// Configuration for a UV projection operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvProjectConfig {
    pub mode: UvProjectMode,
    pub scale: [f32; 2],
    pub offset: [f32; 2],
    pub rotation_deg: f32,
}

/// Result of a UV projection operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvProjectResult {
    pub uvs: Vec<[f32; 2]>,
    pub coverage: f32,
    pub overlap_count: usize,
}

// ── Helpers ──────────────────────────────────────────────────────────────────

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn safe_norm3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-12 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

/// Compute a pair of tangent vectors orthogonal to `normal`.
fn make_tangent_frame(normal: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let n = safe_norm3(normal);
    let up = if n[1].abs() < 0.9 {
        [0.0_f32, 1.0, 0.0]
    } else {
        [1.0_f32, 0.0, 0.0]
    };
    // tangent = normalize(up - dot(up,n)*n)
    let d = dot3(up, n);
    let t_raw = [up[0] - d * n[0], up[1] - d * n[1], up[2] - d * n[2]];
    let t = safe_norm3(t_raw);
    // bitangent = n cross t
    let b = [
        n[1] * t[2] - n[2] * t[1],
        n[2] * t[0] - n[0] * t[2],
        n[0] * t[1] - n[1] * t[0],
    ];
    (t, b)
}

fn compute_coverage(uvs: &[[f32; 2]]) -> f32 {
    if uvs.is_empty() {
        return 0.0;
    }
    let (mut umin, mut umax) = (f32::INFINITY, f32::NEG_INFINITY);
    let (mut vmin, mut vmax) = (f32::INFINITY, f32::NEG_INFINITY);
    for uv in uvs {
        umin = umin.min(uv[0]);
        umax = umax.max(uv[0]);
        vmin = vmin.min(uv[1]);
        vmax = vmax.max(uv[1]);
    }
    let du = (umax - umin).max(0.0);
    let dv = (vmax - vmin).max(0.0);
    (du * dv).clamp(0.0, 1.0)
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Build a default `UvProjectConfig` for the given mode.
#[allow(dead_code)]
pub fn default_uv_project_config(mode: UvProjectMode) -> UvProjectConfig {
    UvProjectConfig {
        mode,
        scale: [1.0, 1.0],
        offset: [0.0, 0.0],
        rotation_deg: 0.0,
    }
}

/// Project positions onto a plane defined by `normal`.
#[allow(dead_code)]
pub fn project_planar(
    positions: &[[f32; 3]],
    normal: [f32; 3],
    cfg: &UvProjectConfig,
) -> UvProjectResult {
    let (t, b) = make_tangent_frame(normal);
    let mut uvs: Vec<[f32; 2]> = positions
        .iter()
        .map(|p| {
            let u = dot3(*p, t) * cfg.scale[0] + cfg.offset[0];
            let v = dot3(*p, b) * cfg.scale[1] + cfg.offset[1];
            [u, v]
        })
        .collect();
    if cfg.rotation_deg.abs() > 1e-6 {
        apply_uv_transform(&mut uvs, [1.0, 1.0], [0.0, 0.0], cfg.rotation_deg);
    }
    let coverage = compute_coverage(&uvs);
    UvProjectResult {
        overlap_count: 0,
        coverage,
        uvs,
    }
}

/// Project positions cylindrically around `axis`.
#[allow(dead_code)]
pub fn project_cylindrical(
    positions: &[[f32; 3]],
    axis: [f32; 3],
    cfg: &UvProjectConfig,
) -> UvProjectResult {
    let ax = safe_norm3(axis);
    let mut uvs: Vec<[f32; 2]> = positions
        .iter()
        .map(|p| {
            // Project onto axis for V
            let along = dot3(*p, ax);
            // Radial component for U (angle)
            let radial = [
                p[0] - along * ax[0],
                p[1] - along * ax[1],
                p[2] - along * ax[2],
            ];
            let angle = radial[2].atan2(radial[0]); // atan2(z, x)
            let u = (angle / std::f32::consts::TAU + 0.5) * cfg.scale[0] + cfg.offset[0];
            let v = along * cfg.scale[1] + cfg.offset[1];
            [u, v]
        })
        .collect();
    if cfg.rotation_deg.abs() > 1e-6 {
        apply_uv_transform(&mut uvs, [1.0, 1.0], [0.0, 0.0], cfg.rotation_deg);
    }
    let coverage = compute_coverage(&uvs);
    UvProjectResult {
        overlap_count: 0,
        coverage,
        uvs,
    }
}

/// Project positions spherically from `center`.
#[allow(dead_code)]
pub fn project_spherical(
    positions: &[[f32; 3]],
    center: [f32; 3],
    cfg: &UvProjectConfig,
) -> UvProjectResult {
    let mut uvs: Vec<[f32; 2]> = positions
        .iter()
        .map(|p| {
            let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
            let dn = safe_norm3(d);
            let u = (dn[2].atan2(dn[0]) / std::f32::consts::TAU + 0.5) * cfg.scale[0]
                + cfg.offset[0];
            let v = (dn[1].asin() / std::f32::consts::PI + 0.5) * cfg.scale[1] + cfg.offset[1];
            [u, v]
        })
        .collect();
    if cfg.rotation_deg.abs() > 1e-6 {
        apply_uv_transform(&mut uvs, [1.0, 1.0], [0.0, 0.0], cfg.rotation_deg);
    }
    let coverage = compute_coverage(&uvs);
    UvProjectResult {
        overlap_count: 0,
        coverage,
        uvs,
    }
}

/// Project positions using box (tri-planar) mapping.
#[allow(dead_code)]
pub fn project_box_map(positions: &[[f32; 3]], cfg: &UvProjectConfig) -> UvProjectResult {
    let mut uvs: Vec<[f32; 2]> = positions
        .iter()
        .map(|p| {
            let ax = p[0].abs();
            let ay = p[1].abs();
            let az = p[2].abs();
            let (u, v) = if ax >= ay && ax >= az {
                (p[2] * cfg.scale[0], p[1] * cfg.scale[1])
            } else if ay >= ax && ay >= az {
                (p[0] * cfg.scale[0], p[2] * cfg.scale[1])
            } else {
                (p[0] * cfg.scale[0], p[1] * cfg.scale[1])
            };
            [u + cfg.offset[0], v + cfg.offset[1]]
        })
        .collect();
    if cfg.rotation_deg.abs() > 1e-6 {
        apply_uv_transform(&mut uvs, [1.0, 1.0], [0.0, 0.0], cfg.rotation_deg);
    }
    let coverage = compute_coverage(&uvs);
    UvProjectResult {
        overlap_count: 0,
        coverage,
        uvs,
    }
}

/// Dispatch projection based on `cfg.mode`.
#[allow(dead_code)]
pub fn project_vertices(positions: &[[f32; 3]], cfg: &UvProjectConfig) -> UvProjectResult {
    match cfg.mode {
        UvProjectMode::Planar => project_planar(positions, [0.0, 0.0, 1.0], cfg),
        UvProjectMode::Cylindrical => project_cylindrical(positions, [0.0, 1.0, 0.0], cfg),
        UvProjectMode::Spherical => project_spherical(positions, [0.0, 0.0, 0.0], cfg),
        UvProjectMode::Box | UvProjectMode::Camera => project_box_map(positions, cfg),
    }
}

/// Apply scale, offset, and rotation transform to a UV set in-place.
#[allow(dead_code)]
pub fn apply_uv_transform(
    uvs: &mut [[f32; 2]],
    scale: [f32; 2],
    offset: [f32; 2],
    rot_deg: f32,
) {
    let rad = rot_deg.to_radians();
    let (sin_r, cos_r) = rad.sin_cos();
    for uv in uvs.iter_mut() {
        // Apply scale and offset first
        let u = uv[0] * scale[0] + offset[0];
        let v = uv[1] * scale[1] + offset[1];
        // Then rotate around (0.5, 0.5)
        let du = u - 0.5;
        let dv = v - 0.5;
        uv[0] = cos_r * du - sin_r * dv + 0.5;
        uv[1] = sin_r * du + cos_r * dv + 0.5;
    }
}

/// Return a human-readable name for the projection mode in the config.
#[allow(dead_code)]
pub fn uv_project_mode_name(cfg: &UvProjectConfig) -> &'static str {
    match cfg.mode {
        UvProjectMode::Planar => "planar",
        UvProjectMode::Cylindrical => "cylindrical",
        UvProjectMode::Spherical => "spherical",
        UvProjectMode::Box => "box",
        UvProjectMode::Camera => "camera",
    }
}

/// Serialize a `UvProjectResult` to a JSON string.
#[allow(dead_code)]
pub fn uv_project_result_to_json(r: &UvProjectResult) -> String {
    format!(
        "{{\"uv_count\":{},\"coverage\":{:.4},\"overlap_count\":{}}}",
        r.uvs.len(),
        r.coverage,
        r.overlap_count
    )
}

/// Normalize UVs to [0, 1] range in-place.
#[allow(dead_code)]
pub fn normalize_uvs_proj(uvs: &mut [[f32; 2]]) {
    if uvs.is_empty() {
        return;
    }
    let (mut umin, mut umax) = (f32::INFINITY, f32::NEG_INFINITY);
    let (mut vmin, mut vmax) = (f32::INFINITY, f32::NEG_INFINITY);
    for uv in uvs.iter() {
        umin = umin.min(uv[0]);
        umax = umax.max(uv[0]);
        vmin = vmin.min(uv[1]);
        vmax = vmax.max(uv[1]);
    }
    let du = (umax - umin).max(1e-12);
    let dv = (vmax - vmin).max(1e-12);
    for uv in uvs.iter_mut() {
        uv[0] = (uv[0] - umin) / du;
        uv[1] = (uv[1] - vmin) / dv;
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_square() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    #[test]
    fn default_config_has_identity_scale() {
        let cfg = default_uv_project_config(UvProjectMode::Planar);
        assert!((cfg.scale[0] - 1.0).abs() < 1e-6);
        assert!((cfg.scale[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn planar_projection_produces_correct_count() {
        let pos = unit_square();
        let cfg = default_uv_project_config(UvProjectMode::Planar);
        let result = project_planar(&pos, [0.0, 0.0, 1.0], &cfg);
        assert_eq!(result.uvs.len(), pos.len());
    }

    #[test]
    fn cylindrical_projection_u_in_range() {
        let pos = vec![
            [1.0_f32, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let cfg = default_uv_project_config(UvProjectMode::Cylindrical);
        let result = project_cylindrical(&pos, [0.0, 1.0, 0.0], &cfg);
        for uv in &result.uvs {
            assert!(uv[0] >= 0.0 && uv[0] <= 1.0, "U out of range: {}", uv[0]);
        }
    }

    #[test]
    fn spherical_projection_returns_all_finite() {
        let pos = vec![[1.0_f32, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let cfg = default_uv_project_config(UvProjectMode::Spherical);
        let result = project_spherical(&pos, [0.0, 0.0, 0.0], &cfg);
        for uv in &result.uvs {
            assert!(uv[0].is_finite() && uv[1].is_finite());
        }
    }

    #[test]
    fn box_map_returns_correct_count() {
        let pos = unit_square();
        let cfg = default_uv_project_config(UvProjectMode::Box);
        let result = project_box_map(&pos, &cfg);
        assert_eq!(result.uvs.len(), pos.len());
    }

    #[test]
    fn normalize_uvs_proj_clamps_to_unit() {
        let mut uvs = vec![[2.0_f32, 3.0], [4.0, 5.0], [6.0, 7.0]];
        normalize_uvs_proj(&mut uvs);
        for uv in &uvs {
            assert!(uv[0] >= 0.0 && uv[0] <= 1.0);
            assert!(uv[1] >= 0.0 && uv[1] <= 1.0);
        }
    }

    #[test]
    fn project_vertices_dispatch_works() {
        let pos = unit_square();
        for mode in [
            UvProjectMode::Planar,
            UvProjectMode::Cylindrical,
            UvProjectMode::Spherical,
            UvProjectMode::Box,
            UvProjectMode::Camera,
        ] {
            let cfg = default_uv_project_config(mode);
            let result = project_vertices(&pos, &cfg);
            assert_eq!(result.uvs.len(), pos.len());
        }
    }

    #[test]
    fn uv_project_mode_name_all_modes() {
        let cases = [
            (UvProjectMode::Planar, "planar"),
            (UvProjectMode::Cylindrical, "cylindrical"),
            (UvProjectMode::Spherical, "spherical"),
            (UvProjectMode::Box, "box"),
            (UvProjectMode::Camera, "camera"),
        ];
        for (mode, expected) in cases {
            let cfg = default_uv_project_config(mode);
            assert_eq!(uv_project_mode_name(&cfg), expected);
        }
    }

    #[test]
    fn uv_project_result_to_json_contains_fields() {
        let r = UvProjectResult {
            uvs: vec![[0.0, 0.0], [1.0, 1.0]],
            coverage: 0.5,
            overlap_count: 0,
        };
        let json = uv_project_result_to_json(&r);
        assert!(json.contains("uv_count"));
        assert!(json.contains("coverage"));
    }

    #[test]
    fn apply_uv_transform_identity_no_change() {
        let mut uvs = vec![[0.25_f32, 0.75]];
        let orig = uvs.clone();
        apply_uv_transform(&mut uvs, [1.0, 1.0], [0.0, 0.0], 0.0);
        assert!((uvs[0][0] - orig[0][0]).abs() < 1e-5);
        assert!((uvs[0][1] - orig[0][1]).abs() < 1e-5);
    }
}
