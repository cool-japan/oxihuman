// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Strand-to-mesh conversion: turns hair strands into renderable triangle meshes.

/// Parameters for converting hair strands to mesh ribbons.
#[derive(Debug, Clone)]
pub struct StrandToMeshParams {
    /// Width of each ribbon in world units.
    pub ribbon_width: f32,
    /// Whether to generate UV coordinates.
    pub gen_uvs: bool,
    /// Up vector for ribbon orientation.
    pub up: [f32; 3],
}

impl Default for StrandToMeshParams {
    fn default() -> Self {
        Self {
            ribbon_width: 0.004,
            gen_uvs: true,
            up: [0.0, 1.0, 0.0],
        }
    }
}

/// A ribbon mesh generated from a single strand.
#[derive(Debug, Clone)]
pub struct StrandRibbon {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub uvs: Vec<[f32; 2]>,
}

impl StrandRibbon {
    /// Number of triangles in this ribbon.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

/// Convert a strand (list of spine points) to a flat ribbon mesh.
pub fn strand_to_ribbon(spine: &[[f32; 3]], params: &StrandToMeshParams) -> StrandRibbon {
    if spine.len() < 2 {
        return StrandRibbon {
            positions: vec![],
            indices: vec![],
            uvs: vec![],
        };
    }
    let half = params.ribbon_width * 0.5;
    let right = perp_to_forward(spine, params.up);
    let n = spine.len();
    let mut positions = Vec::with_capacity(n * 2);
    let mut uvs = Vec::with_capacity(n * 2);
    for (i, &p) in spine.iter().enumerate() {
        let t = i as f32 / (n - 1) as f32;
        let r = right[i];
        positions.push([p[0] - r[0] * half, p[1] - r[1] * half, p[2] - r[2] * half]);
        positions.push([p[0] + r[0] * half, p[1] + r[1] * half, p[2] + r[2] * half]);
        uvs.push([0.0, t]);
        uvs.push([1.0, t]);
    }
    let mut indices = Vec::new();
    for i in 0..(n as u32 - 1) {
        let base = i * 2;
        indices.extend_from_slice(&[base, base + 1, base + 2, base + 1, base + 3, base + 2]);
    }
    StrandRibbon {
        positions,
        indices,
        uvs,
    }
}

/// Compute per-point right vectors perpendicular to the strand direction.
fn perp_to_forward(spine: &[[f32; 3]], up: [f32; 3]) -> Vec<[f32; 3]> {
    spine
        .windows(2)
        .map(|w| {
            let fwd = normalize3([w[1][0] - w[0][0], w[1][1] - w[0][1], w[1][2] - w[0][2]]);
            normalize3(cross3(fwd, up))
        })
        .chain(std::iter::once({
            let last = spine.len() - 1;
            let fwd = normalize3([
                spine[last][0] - spine[last - 1][0],
                spine[last][1] - spine[last - 1][1],
                spine[last][2] - spine[last - 1][2],
            ]);
            normalize3(cross3(fwd, up))
        }))
        .collect()
}

/// Merge multiple strand ribbons into a single mesh.
pub fn merge_ribbons(ribbons: &[StrandRibbon]) -> StrandRibbon {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut uvs = Vec::new();
    for ribbon in ribbons {
        let offset = positions.len() as u32;
        positions.extend_from_slice(&ribbon.positions);
        uvs.extend_from_slice(&ribbon.uvs);
        indices.extend(ribbon.indices.iter().map(|&i| i + offset));
    }
    StrandRibbon {
        positions,
        indices,
        uvs,
    }
}

/// Estimate total triangle count before generation.
pub fn estimate_triangle_count(strand_count: usize, segments_per_strand: usize) -> usize {
    strand_count * (segments_per_strand.saturating_sub(1)) * 2
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        return [1.0, 0.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_spine() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [0.0, 0.5, 0.0], [0.0, 1.0, 0.0]]
    }

    #[test]
    fn ribbon_vertex_count() {
        /* 3 spine points → 6 verts */
        let r = strand_to_ribbon(&simple_spine(), &StrandToMeshParams::default());
        assert_eq!(r.positions.len(), 6);
    }

    #[test]
    fn ribbon_triangle_count() {
        /* 3 spine points → 4 triangles (2 quads) */
        let r = strand_to_ribbon(&simple_spine(), &StrandToMeshParams::default());
        assert_eq!(r.triangle_count(), 4);
    }

    #[test]
    fn short_spine_returns_empty() {
        /* single point → empty ribbon */
        let r = strand_to_ribbon(&[[0.0; 3]], &StrandToMeshParams::default());
        assert!(r.positions.is_empty());
    }

    #[test]
    fn uv_count_matches_positions() {
        /* uvs and positions should have same len */
        let r = strand_to_ribbon(&simple_spine(), &StrandToMeshParams::default());
        assert_eq!(r.uvs.len(), r.positions.len());
    }

    #[test]
    fn merge_ribbons_combines() {
        /* two ribbons → double vertex count */
        let r1 = strand_to_ribbon(&simple_spine(), &StrandToMeshParams::default());
        let r2 = strand_to_ribbon(&simple_spine(), &StrandToMeshParams::default());
        let merged = merge_ribbons(&[r1.clone(), r2]);
        assert_eq!(merged.positions.len(), r1.positions.len() * 2);
    }

    #[test]
    fn merge_ribbons_index_offset() {
        /* merged indices should be offset properly */
        let r1 = strand_to_ribbon(&simple_spine(), &StrandToMeshParams::default());
        let r2 = strand_to_ribbon(&simple_spine(), &StrandToMeshParams::default());
        let n = r1.positions.len() as u32;
        let merged = merge_ribbons(&[r1, r2]);
        assert!(merged
            .indices
            .iter()
            .all(|&i| i < merged.positions.len() as u32));
        /* second ribbon indices should reference offset range */
        assert!(merged.indices.contains(&n));
    }

    #[test]
    fn estimate_triangle_count_formula() {
        /* 10 strands × 5 segments → 80 tris */
        assert_eq!(estimate_triangle_count(10, 5), 80);
    }

    #[test]
    fn default_ribbon_width() {
        /* default width is 0.004 */
        let p = StrandToMeshParams::default();
        assert!((p.ribbon_width - 0.004).abs() < 1e-7);
    }

    #[test]
    fn default_gen_uvs() {
        /* gen_uvs is true by default */
        assert!(StrandToMeshParams::default().gen_uvs);
    }
}
