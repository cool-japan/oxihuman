// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Screw thread helix mesh generator.

use std::f32::consts::PI;

/// Parameters for a screw thread mesh.
#[derive(Debug, Clone)]
pub struct ScrewParams {
    /// Major (outer) radius of the screw.
    pub major_radius: f32,
    /// Minor (root) radius.
    pub minor_radius: f32,
    /// Thread pitch (axial distance per turn).
    pub pitch: f32,
    /// Total number of turns.
    pub turns: f32,
    /// Segments per turn along the helix.
    pub segments_per_turn: usize,
    /// Points in the thread profile cross-section.
    pub profile_points: usize,
}

impl Default for ScrewParams {
    fn default() -> Self {
        Self {
            major_radius: 0.05,
            minor_radius: 0.04,
            pitch: 0.008,
            turns: 10.0,
            segments_per_turn: 32,
            profile_points: 4,
        }
    }
}

/// Generated screw thread mesh.
#[derive(Debug, Clone)]
pub struct ScrewMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

impl ScrewMesh {
    /// Triangle count.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Vertex count.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }
}

/// Build a screw thread mesh (simplified: helical band).
pub fn build_screw(params: &ScrewParams) -> ScrewMesh {
    let total_seg = (params.turns * params.segments_per_turn as f32).round() as usize;
    let total_seg = total_seg.max(8);
    let pp = params.profile_points.max(2);
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    for i in 0..=total_seg {
        let t = i as f32 / total_seg as f32;
        let angle = t * params.turns * 2.0 * PI;
        let y = t * params.turns * params.pitch;
        let (sa, ca) = angle.sin_cos();
        /* inner radius ring */
        for j in 0..pp {
            let tf = j as f32 / (pp - 1) as f32;
            let r = params.minor_radius + tf * (params.major_radius - params.minor_radius);
            positions.push([r * ca, y, r * sa]);
            normals.push([ca, 0.0, sa]);
        }
    }
    let mut indices = Vec::new();
    let pp32 = pp as u32;
    for i in 0..(total_seg as u32) {
        for j in 0..(pp32 - 1) {
            let a = i * pp32 + j;
            let b = i * pp32 + j + 1;
            let c = (i + 1) * pp32 + j;
            let d = (i + 1) * pp32 + j + 1;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }
    ScrewMesh {
        positions,
        normals,
        indices,
    }
}

/// Compute screw arc length along the helix centerline.
pub fn screw_arc_length(params: &ScrewParams) -> f32 {
    let cr = (params.major_radius + params.minor_radius) * 0.5;
    let circum = 2.0 * PI * cr;
    params.turns * (circum * circum + params.pitch * params.pitch).sqrt()
}

/// Compute thread depth.
pub fn thread_depth(params: &ScrewParams) -> f32 {
    params.major_radius - params.minor_radius
}

/// Validate screw parameters.
pub fn validate_screw_params(p: &ScrewParams) -> bool {
    p.major_radius > p.minor_radius
        && p.minor_radius > 0.0
        && p.pitch > 0.0
        && p.turns > 0.0
        && p.segments_per_turn >= 4
        && p.profile_points >= 2
}

/// Estimate total vertex count.
pub fn estimated_vertex_count(params: &ScrewParams) -> usize {
    let total_seg = (params.turns * params.segments_per_turn as f32).round() as usize;
    (total_seg + 1) * params.profile_points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screw_has_vertices() {
        let m = build_screw(&ScrewParams::default());
        assert!(m.vertex_count() > 0);
    }

    #[test]
    fn screw_has_triangles() {
        let m = build_screw(&ScrewParams::default());
        assert!(m.triangle_count() > 0);
    }

    #[test]
    fn indices_in_bounds() {
        let m = build_screw(&ScrewParams::default());
        let n = m.positions.len() as u32;
        assert!(m.indices.iter().all(|&i| i < n));
    }

    #[test]
    fn normals_match_positions() {
        let m = build_screw(&ScrewParams::default());
        assert_eq!(m.normals.len(), m.positions.len());
    }

    #[test]
    fn arc_length_positive() {
        assert!(screw_arc_length(&ScrewParams::default()) > 0.0);
    }

    #[test]
    fn thread_depth_positive() {
        assert!(thread_depth(&ScrewParams::default()) > 0.0);
    }

    #[test]
    fn validate_ok() {
        assert!(validate_screw_params(&ScrewParams::default()));
    }

    #[test]
    fn validate_bad_radii() {
        let mut p = ScrewParams::default();
        p.minor_radius = p.major_radius + 0.01;
        assert!(!validate_screw_params(&p));
    }

    #[test]
    fn estimate_vertex_count_nonzero() {
        assert!(estimated_vertex_count(&ScrewParams::default()) > 0);
    }
}
