// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cable / rope mesh generator.

/// Parameters for cable mesh generation.
#[derive(Debug, Clone)]
pub struct CableParams {
    /// Outer radius of the cable sheath.
    pub radius: f32,
    /// Number of cross-section segments (≥ 3).
    pub radial_segments: usize,
    /// Number of segments along the cable length.
    pub length_segments: usize,
    /// Total cable length.
    pub length: f32,
}

impl Default for CableParams {
    fn default() -> Self {
        Self {
            radius: 0.02,
            radial_segments: 8,
            length_segments: 20,
            length: 1.0,
        }
    }
}

/// Result of cable mesh generation.
#[derive(Debug, Clone)]
pub struct CableMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

impl CableMesh {
    /// Number of triangles.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }
}

/// Build a straight cylindrical cable mesh along the Y axis.
pub fn build_cable(params: &CableParams) -> CableMesh {
    let r = params.radial_segments.max(3);
    let l = params.length_segments.max(1);
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for i in 0..=(l) {
        let y = i as f32 / l as f32 * params.length;
        let v = i as f32 / l as f32;
        for j in 0..=r {
            let angle = 2.0 * std::f32::consts::PI * j as f32 / r as f32;
            let (s, c) = angle.sin_cos();
            positions.push([c * params.radius, y, s * params.radius]);
            normals.push([c, 0.0, s]);
            uvs.push([j as f32 / r as f32, v]);
        }
    }
    let mut indices = Vec::new();
    for i in 0..(l as u32) {
        for j in 0..(r as u32) {
            let row = (r as u32 + 1) * i;
            let a = row + j;
            let b = row + j + 1;
            let c = row + r as u32 + 1 + j;
            let d = row + r as u32 + 1 + j + 1;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }
    CableMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}

/// Compute expected vertex count.
pub fn expected_vertex_count(radial: usize, length: usize) -> usize {
    (radial + 1) * (length + 1)
}

/// Compute expected triangle count.
pub fn expected_triangle_count(radial: usize, length: usize) -> usize {
    radial * length * 2
}

/// Validate cable params.
pub fn validate_cable_params(p: &CableParams) -> bool {
    p.radius > 0.0 && p.radial_segments >= 3 && p.length_segments >= 1 && p.length > 0.0
}

/// Surface area of a cylindrical cable (lateral only).
pub fn cable_lateral_area(params: &CableParams) -> f32 {
    2.0 * std::f32::consts::PI * params.radius * params.length
}

/// Mass of a cable given linear density (kg/m).
pub fn cable_mass(params: &CableParams, linear_density: f32) -> f32 {
    params.length * linear_density
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_count_correct() {
        /* 8 radial × 20 length × 2 = 320 */
        let m = build_cable(&CableParams::default());
        assert_eq!(m.triangle_count(), 320);
    }

    #[test]
    fn vertex_count_correct() {
        /* (8+1) × (20+1) = 189 */
        let m = build_cable(&CableParams::default());
        assert_eq!(m.vertex_count(), expected_vertex_count(8, 20));
    }

    #[test]
    fn normals_length_one() {
        /* all normals should be unit length */
        let m = build_cable(&CableParams::default());
        for n in &m.normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn validate_ok() {
        assert!(validate_cable_params(&CableParams::default()));
    }

    #[test]
    fn validate_bad_radius() {
        let p = CableParams {
            radius: 0.0,
            ..CableParams::default()
        };
        assert!(!validate_cable_params(&p));
    }

    #[test]
    fn lateral_area_positive() {
        assert!(cable_lateral_area(&CableParams::default()) > 0.0);
    }

    #[test]
    fn mass_calculation() {
        /* 1 m cable, 0.5 kg/m → 0.5 kg */
        let p = CableParams {
            length: 1.0,
            ..CableParams::default()
        };
        assert!((cable_mass(&p, 0.5) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn expected_formula_matches_build() {
        let p = CableParams::default();
        let m = build_cable(&p);
        assert_eq!(m.triangle_count(), expected_triangle_count(8, 20));
    }

    #[test]
    fn indices_in_bounds() {
        let m = build_cable(&CableParams::default());
        let n = m.positions.len() as u32;
        assert!(m.indices.iter().all(|&i| i < n));
    }
}
