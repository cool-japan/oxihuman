// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Chain link mesh generator.

/// Parameters for a single chain link.
#[derive(Debug, Clone)]
pub struct ChainLinkParams {
    /// Outer radius of the link ring.
    pub outer_radius: f32,
    /// Wire radius (tube radius of the torus).
    pub wire_radius: f32,
    /// Segments around the link circumference.
    pub major_segments: usize,
    /// Segments around the wire cross-section.
    pub minor_segments: usize,
    /// Number of chain links to generate.
    pub link_count: usize,
}

impl Default for ChainLinkParams {
    fn default() -> Self {
        Self {
            outer_radius: 0.1,
            wire_radius: 0.02,
            major_segments: 16,
            minor_segments: 8,
            link_count: 5,
        }
    }
}

/// A single torus-ring link mesh.
#[derive(Debug, Clone)]
pub struct LinkMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

impl LinkMesh {
    /// Triangle count.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Vertex count.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }
}

/// Build one torus-link mesh (stub uses approximate torus tessellation).
pub fn build_link(params: &ChainLinkParams) -> LinkMesh {
    let maj = params.major_segments.max(4);
    let min = params.minor_segments.max(3);
    let r_maj = params.outer_radius - params.wire_radius;
    let r_min = params.wire_radius;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    for i in 0..maj {
        let phi = 2.0 * std::f32::consts::PI * i as f32 / maj as f32;
        let (sp, cp) = phi.sin_cos();
        for j in 0..min {
            let theta = 2.0 * std::f32::consts::PI * j as f32 / min as f32;
            let (st, ct) = theta.sin_cos();
            let x = (r_maj + r_min * ct) * cp;
            let y = r_min * st;
            let z = (r_maj + r_min * ct) * sp;
            positions.push([x, y, z]);
            normals.push([ct * cp, st, ct * sp]);
        }
    }
    let mut indices = Vec::new();
    for i in 0..maj {
        for j in 0..min {
            let a = (i * min + j) as u32;
            let b = (i * min + (j + 1) % min) as u32;
            let c = (((i + 1) % maj) * min + j) as u32;
            let d = (((i + 1) % maj) * min + (j + 1) % min) as u32;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }
    LinkMesh {
        positions,
        normals,
        indices,
    }
}

/// Build an entire chain: alternating links rotated 90 degrees.
pub fn build_chain(params: &ChainLinkParams) -> Vec<LinkMesh> {
    let step = params.outer_radius * 2.0 - params.wire_radius;
    (0..params.link_count)
        .map(|i| {
            let mut link = build_link(params);
            let offset_y = i as f32 * step;
            let rot = if i % 2 == 0 {
                0.0f32
            } else {
                std::f32::consts::FRAC_PI_2
            };
            for p in &mut link.positions {
                let x = p[0] * rot.cos() - p[2] * rot.sin();
                let z = p[0] * rot.sin() + p[2] * rot.cos();
                p[0] = x;
                p[1] += offset_y;
                p[2] = z;
            }
            link
        })
        .collect()
}

/// Compute expected triangles per link.
pub fn triangles_per_link(maj: usize, min: usize) -> usize {
    maj * min * 2
}

/// Validate chain link params.
pub fn validate_link_params(p: &ChainLinkParams) -> bool {
    p.outer_radius > p.wire_radius
        && p.wire_radius > 0.0
        && p.major_segments >= 4
        && p.minor_segments >= 3
        && p.link_count >= 1
}

/// Estimate total triangles in the chain.
pub fn chain_triangle_count(params: &ChainLinkParams) -> usize {
    params.link_count * triangles_per_link(params.major_segments, params.minor_segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_triangle_count() {
        /* 16 major × 8 minor × 2 = 256 */
        let link = build_link(&ChainLinkParams::default());
        assert_eq!(link.triangle_count(), 256);
    }

    #[test]
    fn link_vertex_count() {
        /* 16 × 8 = 128 */
        let link = build_link(&ChainLinkParams::default());
        assert_eq!(link.vertex_count(), 128);
    }

    #[test]
    fn chain_link_count() {
        /* build_chain returns 5 links */
        let chain = build_chain(&ChainLinkParams::default());
        assert_eq!(chain.len(), 5);
    }

    #[test]
    fn validate_params_ok() {
        assert!(validate_link_params(&ChainLinkParams::default()));
    }

    #[test]
    fn validate_bad_radii() {
        let p = ChainLinkParams {
            outer_radius: 0.01,
            wire_radius: 0.05,
            ..ChainLinkParams::default()
        };
        assert!(!validate_link_params(&p));
    }

    #[test]
    fn triangles_per_link_formula() {
        /* 16 × 8 × 2 = 256 */
        assert_eq!(triangles_per_link(16, 8), 256);
    }

    #[test]
    fn chain_total_triangles() {
        /* 5 links × 256 = 1280 */
        assert_eq!(chain_triangle_count(&ChainLinkParams::default()), 1280);
    }

    #[test]
    fn link_indices_in_bounds() {
        let link = build_link(&ChainLinkParams::default());
        let n = link.positions.len() as u32;
        assert!(link.indices.iter().all(|&i| i < n));
    }

    #[test]
    fn normals_count_matches() {
        let link = build_link(&ChainLinkParams::default());
        assert_eq!(link.normals.len(), link.positions.len());
    }
}
