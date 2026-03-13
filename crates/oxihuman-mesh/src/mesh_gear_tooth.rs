// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Involute gear tooth mesh generator.

use std::f32::consts::PI;

/// Parameters for an involute gear.
#[derive(Debug, Clone)]
pub struct GearParams {
    /// Number of teeth.
    pub tooth_count: usize,
    /// Module (tooth size parameter).
    pub module: f32,
    /// Pressure angle in degrees.
    pub pressure_angle_deg: f32,
    /// Gear thickness (extrusion depth).
    pub thickness: f32,
    /// Points per involute flank.
    pub flank_points: usize,
}

impl Default for GearParams {
    fn default() -> Self {
        Self {
            tooth_count: 20,
            module: 0.01,
            pressure_angle_deg: 20.0,
            thickness: 0.02,
            flank_points: 8,
        }
    }
}

/// A gear mesh result.
#[derive(Debug, Clone)]
pub struct GearMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
}

impl GearMesh {
    /// Triangle count.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Vertex count.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }
}

/// Compute pitch circle radius.
pub fn pitch_radius(params: &GearParams) -> f32 {
    params.module * params.tooth_count as f32 / 2.0
}

/// Compute addendum circle radius.
pub fn addendum_radius(params: &GearParams) -> f32 {
    pitch_radius(params) + params.module
}

/// Compute dedendum circle radius.
pub fn dedendum_radius(params: &GearParams) -> f32 {
    pitch_radius(params) - 1.25 * params.module
}

/// Compute base circle radius for involute.
pub fn base_radius(params: &GearParams) -> f32 {
    pitch_radius(params) * (params.pressure_angle_deg.to_radians()).cos()
}

/// Sample an involute curve point at parameter `t` (0 = start).
pub fn involute_point(base_r: f32, t: f32) -> [f32; 2] {
    let x = base_r * (t.cos() + t * t.sin());
    let y = base_r * (t.sin() - t * t.cos());
    [x, y]
}

/// Build a simplified flat gear outline (top-down 2-D, then extruded stub).
pub fn build_gear(params: &GearParams) -> GearMesh {
    let n = params.tooth_count.max(3);
    let pr = pitch_radius(params);
    let ar = addendum_radius(params);
    let dr = dedendum_radius(params);
    let tooth_angle = 2.0 * PI / n as f32;
    let half_tooth = tooth_angle * 0.25;
    let mut verts: Vec<[f32; 2]> = Vec::new();
    for i in 0..n {
        let base_a = i as f32 * tooth_angle;
        /* root left */
        let ra_l = base_a - half_tooth;
        verts.push([dr * ra_l.cos(), dr * ra_l.sin()]);
        /* addendum left */
        let aa_l = base_a - half_tooth * 0.5;
        verts.push([ar * aa_l.cos(), ar * aa_l.sin()]);
        /* addendum right */
        let aa_r = base_a + half_tooth * 0.5;
        verts.push([ar * aa_r.cos(), ar * aa_r.sin()]);
        /* root right */
        let ra_r = base_a + half_tooth;
        verts.push([pr * ra_r.cos(), pr * ra_r.sin()]);
    }
    /* extrude: bottom face (y=0) and top face (y=thickness) */
    let bottom: Vec<[f32; 3]> = verts.iter().map(|v| [v[0], 0.0, v[1]]).collect();
    let top: Vec<[f32; 3]> = verts
        .iter()
        .map(|v| [v[0], params.thickness, v[1]])
        .collect();
    let mut positions = bottom;
    positions.extend(top);
    /* fan triangles from center for each face */
    let nv = verts.len() as u32;
    /* center bottom = separate vertex */
    positions.push([0.0, 0.0, 0.0]);
    positions.push([0.0, params.thickness, 0.0]);
    let c_bot = nv * 2;
    let c_top = nv * 2 + 1;
    let mut indices: Vec<u32> = Vec::new();
    for i in 0..nv {
        let next = (i + 1) % nv;
        /* bottom */
        indices.extend_from_slice(&[c_bot, i, next]);
        /* top */
        indices.extend_from_slice(&[c_top, nv + next, nv + i]);
        /* side quad */
        indices.extend_from_slice(&[i, nv + i, next, next, nv + i, nv + next]);
    }
    let normals = vec![[0.0f32, 1.0, 0.0]; positions.len()];
    GearMesh {
        positions,
        normals,
        indices,
    }
}

/// Validate gear parameters.
pub fn validate_gear_params(p: &GearParams) -> bool {
    p.tooth_count >= 3
        && p.module > 0.0
        && (0.0..=45.0).contains(&p.pressure_angle_deg)
        && p.thickness > 0.0
        && p.flank_points >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pitch_radius_correct() {
        /* 20 teeth, module 0.01 → pitch radius 0.1 */
        let p = GearParams::default();
        assert!((pitch_radius(&p) - 0.1).abs() < 1e-5);
    }

    #[test]
    fn addendum_larger_than_pitch() {
        let p = GearParams::default();
        assert!(addendum_radius(&p) > pitch_radius(&p));
    }

    #[test]
    fn dedendum_smaller_than_pitch() {
        let p = GearParams::default();
        assert!(dedendum_radius(&p) < pitch_radius(&p));
    }

    #[test]
    fn base_radius_positive() {
        let p = GearParams::default();
        assert!(base_radius(&p) > 0.0);
    }

    #[test]
    fn involute_origin() {
        /* at t=0, involute starts at (base_r, 0) */
        let pt = involute_point(1.0, 0.0);
        assert!((pt[0] - 1.0).abs() < 1e-5);
        assert!(pt[1].abs() < 1e-5);
    }

    #[test]
    fn build_gear_has_vertices() {
        let m = build_gear(&GearParams::default());
        assert!(m.vertex_count() > 0);
    }

    #[test]
    fn build_gear_has_triangles() {
        let m = build_gear(&GearParams::default());
        assert!(m.triangle_count() > 0);
    }

    #[test]
    fn indices_in_bounds() {
        let m = build_gear(&GearParams::default());
        let n = m.positions.len() as u32;
        assert!(m.indices.iter().all(|&i| i < n));
    }

    #[test]
    fn validate_ok() {
        assert!(validate_gear_params(&GearParams::default()));
    }

    #[test]
    fn validate_bad_teeth() {
        let p = GearParams {
            tooth_count: 2,
            ..GearParams::default()
        };
        assert!(!validate_gear_params(&p));
    }
}
