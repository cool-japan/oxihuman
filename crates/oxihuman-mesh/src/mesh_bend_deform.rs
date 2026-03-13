// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Bend deformation for mesh vertices along an axis.

use std::f32::consts::PI;

/// Bend deformation configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BendDeformConfig {
    pub axis: usize,
    pub angle_rad: f32,
    pub lower_bound: f32,
    pub upper_bound: f32,
}

/// Result of bend deformation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BendDeformResult {
    pub positions: Vec<[f32; 3]>,
    pub max_displacement: f32,
}

/// Create default bend config along Y axis.
#[allow(dead_code)]
pub fn default_bend_config() -> BendDeformConfig {
    BendDeformConfig {
        axis: 1,
        angle_rad: PI / 4.0,
        lower_bound: 0.0,
        upper_bound: 1.0,
    }
}

/// Compute the bend parameter t for a vertex along the bend axis.
#[allow(dead_code)]
pub fn bend_param(v: [f32; 3], cfg: &BendDeformConfig) -> f32 {
    let val = v[cfg.axis];
    let range = cfg.upper_bound - cfg.lower_bound;
    if range.abs() < 1e-12 {
        return 0.0;
    }
    ((val - cfg.lower_bound) / range).clamp(0.0, 1.0)
}

/// Apply bend deformation to a single vertex.
#[allow(dead_code)]
pub fn bend_vertex(v: [f32; 3], cfg: &BendDeformConfig) -> [f32; 3] {
    let t = bend_param(v, cfg);
    let theta = t * cfg.angle_rad;
    let (sin_t, cos_t) = theta.sin_cos();
    let axis = cfg.axis;
    let radial = (axis + 2) % 3;
    let range = cfg.upper_bound - cfg.lower_bound;
    if range.abs() < 1e-12 {
        return v;
    }
    let radius = range / cfg.angle_rad.abs().max(1e-12);
    let offset = v[radial];
    let r = radius + offset;
    let mut result = v;
    result[axis] = cfg.lower_bound + r * sin_t;
    result[radial] = r * cos_t - radius;
    result
}

/// Apply bend deformation to an entire mesh.
#[allow(dead_code)]
pub fn apply_bend_deform(positions: &[[f32; 3]], cfg: &BendDeformConfig) -> BendDeformResult {
    let mut max_disp = 0.0_f32;
    let new_pos: Vec<[f32; 3]> = positions
        .iter()
        .map(|&v| {
            let bent = bend_vertex(v, cfg);
            let d =
                ((bent[0] - v[0]).powi(2) + (bent[1] - v[1]).powi(2) + (bent[2] - v[2]).powi(2))
                    .sqrt();
            max_disp = max_disp.max(d);
            bent
        })
        .collect();
    BendDeformResult {
        positions: new_pos,
        max_displacement: max_disp,
    }
}

/// Vertex count in result.
#[allow(dead_code)]
pub fn bend_vertex_count(r: &BendDeformResult) -> usize {
    r.positions.len()
}

/// Check if deformation is within a tolerance.
#[allow(dead_code)]
pub fn bend_within_tolerance(r: &BendDeformResult, tol: f32) -> bool {
    r.max_displacement <= tol
}

/// Compute angle in degrees from radians.
#[allow(dead_code)]
pub fn rad_to_deg(rad: f32) -> f32 {
    rad * 180.0 / PI
}

/// Compute angle in radians from degrees.
#[allow(dead_code)]
pub fn deg_to_rad(deg: f32) -> f32 {
    deg * PI / 180.0
}

/// Export to JSON.
#[allow(dead_code)]
pub fn bend_deform_to_json(r: &BendDeformResult) -> String {
    format!(
        "{{\"vertices\":{},\"max_displacement\":{:.6}}}",
        bend_vertex_count(r),
        r.max_displacement
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_bend_config();
        assert_eq!(c.axis, 1);
        assert!((c.angle_rad - PI / 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_bend_param() {
        let cfg = default_bend_config();
        let t = bend_param([0.0, 0.5, 0.0], &cfg);
        assert!((t - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_bend_param_clamped() {
        let cfg = default_bend_config();
        let t = bend_param([0.0, 2.0, 0.0], &cfg);
        assert!((t - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_bend_vertex_zero_angle() {
        let mut cfg = default_bend_config();
        cfg.angle_rad = 0.0;
        let v = [0.5, 0.5, 0.5];
        let b = bend_vertex(v, &cfg);
        // Zero angle means minimal deformation
        assert!(b[0].is_finite());
    }

    #[test]
    fn test_apply_bend_deform() {
        let positions = vec![[0.0, 0.0, 0.0], [0.0, 0.5, 0.0], [0.0, 1.0, 0.0]];
        let cfg = default_bend_config();
        let r = apply_bend_deform(&positions, &cfg);
        assert_eq!(bend_vertex_count(&r), 3);
    }

    #[test]
    fn test_bend_within_tolerance() {
        let positions = vec![[0.0, 0.0, 0.0]];
        let cfg = default_bend_config();
        let r = apply_bend_deform(&positions, &cfg);
        assert!(bend_within_tolerance(&r, 100.0));
    }

    #[test]
    fn test_rad_to_deg() {
        assert!((rad_to_deg(PI) - 180.0).abs() < 1e-4);
    }

    #[test]
    fn test_deg_to_rad() {
        assert!((deg_to_rad(180.0) - PI).abs() < 1e-4);
    }

    #[test]
    fn test_to_json() {
        let r = BendDeformResult {
            positions: vec![[0.0; 3]],
            max_displacement: 0.5,
        };
        let j = bend_deform_to_json(&r);
        assert!(j.contains("\"vertices\":1"));
    }

    #[test]
    fn test_empty() {
        let r = apply_bend_deform(&[], &default_bend_config());
        assert_eq!(bend_vertex_count(&r), 0);
    }
}
