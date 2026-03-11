#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Screw modifier: rotate profile and offset per step.

#[allow(dead_code)]
pub struct ScrewResult {
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
}

#[allow(dead_code)]
pub fn screw_vert_count(profile_len: usize, steps: u32) -> usize {
    profile_len * (steps as usize + 1)
}

#[allow(dead_code)]
pub fn screw_mesh(
    profile: &[[f32; 3]],
    steps: u32,
    angle_rad: f32,
    screw_offset: f32,
    axis: u8,
) -> ScrewResult {
    let pl = profile.len();
    let total_steps = steps + 1;
    let mut verts = Vec::with_capacity(pl * total_steps as usize);
    let mut tris = Vec::new();

    for s in 0..total_steps {
        let t = s as f32 / steps.max(1) as f32;
        let angle = angle_rad * t;
        let (sin_a, cos_a) = angle.sin_cos();
        let z_off = screw_offset * t;
        for p in profile {
            let v = match axis {
                1 => {
                    // rotate around Y
                    let x = p[0] * cos_a - p[2] * sin_a;
                    let z = p[0] * sin_a + p[2] * cos_a;
                    [x, p[1] + z_off, z]
                }
                2 => {
                    // rotate around Z
                    let x = p[0] * cos_a - p[1] * sin_a;
                    let y = p[0] * sin_a + p[1] * cos_a;
                    [x, y, p[2] + z_off]
                }
                _ => {
                    // default: rotate around X
                    let y = p[1] * cos_a - p[2] * sin_a;
                    let z = p[1] * sin_a + p[2] * cos_a;
                    [p[0], y, z + z_off]
                }
            };
            verts.push(v);
        }
        if s > 0 {
            let base0 = ((s - 1) as usize * pl) as u32;
            let base1 = (s as usize * pl) as u32;
            for i in 0..pl.saturating_sub(1) {
                let i0 = base0 + i as u32;
                let i1 = base0 + i as u32 + 1;
                let i2 = base1 + i as u32;
                let i3 = base1 + i as u32 + 1;
                tris.push([i0, i2, i1]);
                tris.push([i1, i2, i3]);
            }
        }
    }
    ScrewResult { verts, tris }
}

// ---- New API required by lib.rs ----

use std::f32::consts::PI;

/// Screw modifier parameters (new API).
pub struct ScrewParams {
    pub angle_deg: f32,
    pub screw_offset: f32,
    pub axis: [f32; 3],
    pub steps: usize,
    pub iterations: usize,
}

pub fn new_screw_params(angle_deg: f32, offset: f32, steps: usize) -> ScrewParams {
    ScrewParams {
        angle_deg,
        screw_offset: offset,
        axis: [0.0, 1.0, 0.0],
        steps: steps.max(1),
        iterations: 1,
    }
}

pub fn screw_transform_vertex(p: [f32; 3], params: &ScrewParams, step: usize) -> [f32; 3] {
    let total_steps = params.steps * params.iterations;
    let t = step as f32 / total_steps.max(1) as f32;
    let angle_rad = params.angle_deg * PI / 180.0 * t;
    let (sin_a, cos_a) = angle_rad.sin_cos();
    let ax = params.axis;
    // Rodrigues rotation around axis
    let dot = p[0] * ax[0] + p[1] * ax[1] + p[2] * ax[2];
    let cross = [
        ax[1] * p[2] - ax[2] * p[1],
        ax[2] * p[0] - ax[0] * p[2],
        ax[0] * p[1] - ax[1] * p[0],
    ];
    let rx = p[0] * cos_a + cross[0] * sin_a + ax[0] * dot * (1.0 - cos_a);
    let ry = p[1] * cos_a + cross[1] * sin_a + ax[1] * dot * (1.0 - cos_a);
    let rz = p[2] * cos_a + cross[2] * sin_a + ax[2] * dot * (1.0 - cos_a);
    // translate along axis
    let translate = params.screw_offset * t;
    [
        rx + ax[0] * translate,
        ry + ax[1] * translate,
        rz + ax[2] * translate,
    ]
}

pub fn screw_total_height(params: &ScrewParams) -> f32 {
    params.screw_offset * params.iterations as f32
}

pub fn screw_total_angle_deg(params: &ScrewParams) -> f32 {
    params.angle_deg * params.iterations as f32
}

pub fn screw_output_vertex_count(input: usize, params: &ScrewParams) -> usize {
    input * (params.steps * params.iterations + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_profile() -> Vec<[f32; 3]> {
        vec![[1.0, 0.0, 0.0], [1.0, 1.0, 0.0]]
    }

    #[test]
    fn vert_count_formula() {
        assert_eq!(screw_vert_count(2, 4), 10);
    }

    #[test]
    fn screw_verts_count_matches() {
        let r = screw_mesh(&line_profile(), 4, PI, 1.0, 0);
        assert_eq!(r.verts.len(), screw_vert_count(2, 4));
    }

    #[test]
    fn zero_angle_first_vert_unchanged() {
        let profile = vec![[1.0, 0.0, 0.0]];
        let r = screw_mesh(&profile, 4, 0.0, 0.0, 0);
        assert!((r.verts[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn screw_generates_tris() {
        let r = screw_mesh(&line_profile(), 4, PI, 0.0, 0);
        assert!(!r.tris.is_empty());
    }

    #[test]
    fn axis_y_rotation() {
        let profile = vec![[1.0, 0.0, 0.0]];
        let r = screw_mesh(&profile, 1, PI / 2.0, 0.0, 1);
        // after 90 deg around Y: x -> z
        assert!(r.verts[1][2].abs() > 0.5);
    }

    #[test]
    fn axis_z_rotation() {
        let profile = vec![[1.0, 0.0, 0.0]];
        let r = screw_mesh(&profile, 1, PI / 2.0, 0.0, 2);
        // after 90 deg around Z: x -> y
        assert!(r.verts[1][1].abs() > 0.5);
    }

    #[test]
    fn screw_offset_applied() {
        let profile = vec![[1.0, 0.0, 0.0]];
        let r = screw_mesh(&profile, 2, 0.0, 3.0, 0);
        // last vert should have z_off = 3.0 (axis 0 offsets z)
        assert!((r.verts[2][2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn full_rotation_uses_tau() {
        // tau should be 2*PI
        let tau = std::f32::consts::TAU;
        assert!((tau - 2.0 * PI).abs() < 1e-6);
    }

    #[test]
    fn single_step_two_tris_per_segment() {
        let r = screw_mesh(&line_profile(), 1, PI, 0.0, 0);
        assert_eq!(r.tris.len(), 2);
    }
}
