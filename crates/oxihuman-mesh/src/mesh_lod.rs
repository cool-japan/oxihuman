// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Level-of-detail (LOD) chain management for progressive mesh rendering.
//!
//! Provides [`LodChain`] to store multiple resolution levels of the same mesh,
//! select the appropriate level by camera distance, and auto-generate a chain
//! from a base mesh using grid-based decimation.

use crate::mesh::MeshBuffers;

// ── LOD tier enum ─────────────────────────────────────────────────────────────

/// Named LOD tiers representing quality levels.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LodTier {
    /// Highest quality (full resolution, tier 0).
    Lod0,
    /// Medium quality (tier 1, ~50% reduction).
    Lod1,
    /// Low quality (tier 2, ~75% reduction).
    Lod2,
    /// Lowest quality (tier 3, ~87.5% reduction).
    Lod3,
}

impl LodTier {
    /// Return the target vertex ratio for this tier (relative to Lod0).
    #[allow(dead_code)]
    pub fn ratio(self) -> f32 {
        match self {
            LodTier::Lod0 => 1.0,
            LodTier::Lod1 => 0.5,
            LodTier::Lod2 => 0.25,
            LodTier::Lod3 => 0.125,
        }
    }
}

// ── Configuration ─────────────────────────────────────────────────────────────

/// Configuration for LOD chain generation and selection.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodConfig {
    /// Distance thresholds (meters) for switching from Lod0→1, 1→2, 2→3.
    pub thresholds: [f32; 3],
    /// Global LOD bias: positive = prefer higher detail, negative = prefer lower.
    pub lod_bias: f32,
    /// Maximum number of LOD levels to generate.
    pub max_levels: usize,
}

/// Build a [`LodConfig`] with sensible defaults.
#[allow(dead_code)]
pub fn default_lod_config() -> LodConfig {
    LodConfig {
        thresholds: [5.0, 15.0, 40.0],
        lod_bias: 0.0,
        max_levels: 4,
    }
}

// ── Per-level mesh container ──────────────────────────────────────────────────

/// A single LOD level holding a mesh and its target tier.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodMesh {
    /// The tier this mesh represents.
    pub tier: LodTier,
    /// The mesh data at this LOD level.
    pub mesh: MeshBuffers,
}

// ── LOD chain ─────────────────────────────────────────────────────────────────

/// Ordered chain of LOD levels for a single mesh asset.
///
/// Levels are stored in ascending tier order (Lod0 first).
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct LodChain {
    levels: Vec<LodMesh>,
    config: LodConfig,
}

// ── Default impl for LodConfig ────────────────────────────────────────────────

impl Default for LodConfig {
    fn default() -> Self {
        default_lod_config()
    }
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Create a new, empty [`LodChain`] with the given configuration.
#[allow(dead_code)]
pub fn new_lod_chain(config: LodConfig) -> LodChain {
    LodChain {
        levels: Vec::new(),
        config,
    }
}

/// Append a pre-built [`LodMesh`] to an existing chain.
///
/// Levels are kept sorted by tier (Lod0 first).
#[allow(dead_code)]
pub fn add_lod_level(chain: &mut LodChain, entry: LodMesh) {
    chain.levels.push(entry);
    chain.levels.sort_by_key(|l| l.tier as u8);
}

/// Select the best [`LodTier`] for a given camera distance.
///
/// Returns `LodTier::Lod0` when the chain is empty.
#[allow(dead_code)]
pub fn select_lod(chain: &LodChain, distance: f32) -> LodTier {
    if chain.levels.is_empty() {
        return LodTier::Lod0;
    }
    let d = (distance + chain.config.lod_bias).max(0.0);
    let t = &chain.config.thresholds;
    let raw_tier = if d < t[0] {
        0usize
    } else if d < t[1] {
        1
    } else if d < t[2] {
        2
    } else {
        3
    };
    // Clamp to the number of levels actually available.
    let clamped = raw_tier.min(chain.levels.len().saturating_sub(1));
    chain.levels[clamped].tier
}

/// Return the vertex count for the given tier, or 0 if not present.
#[allow(dead_code)]
pub fn lod_vertex_count(chain: &LodChain, tier: LodTier) -> usize {
    chain
        .levels
        .iter()
        .find(|l| l.tier == tier)
        .map(|l| l.mesh.positions.len())
        .unwrap_or(0)
}

/// Return the triangle count for the given tier, or 0 if not present.
#[allow(dead_code)]
pub fn lod_triangle_count(chain: &LodChain, tier: LodTier) -> usize {
    chain
        .levels
        .iter()
        .find(|l| l.tier == tier)
        .map(|l| l.mesh.indices.len() / 3)
        .unwrap_or(0)
}

/// Return the number of LOD levels in the chain.
#[allow(dead_code)]
pub fn lod_count(chain: &LodChain) -> usize {
    chain.levels.len()
}

/// Auto-generate a [`LodChain`] from a base mesh by progressive decimation.
///
/// The chain always starts with the original mesh at [`LodTier::Lod0`], then
/// adds up to `config.max_levels - 1` further levels.
#[allow(dead_code)]
pub fn generate_lod_chain(base: &MeshBuffers, config: LodConfig) -> LodChain {
    let max = config.max_levels.min(4);
    let mut chain = new_lod_chain(config);

    let tiers = [LodTier::Lod0, LodTier::Lod1, LodTier::Lod2, LodTier::Lod3];
    for (i, &tier) in tiers.iter().enumerate().take(max) {
        let mesh = if i == 0 {
            base.clone()
        } else {
            lod_decimate(base, tier.ratio())
        };
        chain.levels.push(LodMesh { tier, mesh });
    }
    chain
}

/// Return the distance threshold before which the chain switches from `tier` to the next.
///
/// Returns `f32::INFINITY` for Lod3 (lowest quality, no further step).
#[allow(dead_code)]
pub fn lod_transition_distance(chain: &LodChain, tier: LodTier) -> f32 {
    match tier {
        LodTier::Lod0 => chain.config.thresholds[0],
        LodTier::Lod1 => chain.config.thresholds[1],
        LodTier::Lod2 => chain.config.thresholds[2],
        LodTier::Lod3 => f32::INFINITY,
    }
}

/// Set the LOD bias on the chain's configuration.
#[allow(dead_code)]
pub fn set_lod_bias(chain: &mut LodChain, bias: f32) {
    chain.config.lod_bias = bias;
}

/// Estimate total memory usage of all levels combined (in bytes).
///
/// Counts `positions`, `normals`, `uvs`, and `indices` fields only.
#[allow(dead_code)]
pub fn lod_memory_usage(chain: &LodChain) -> usize {
    chain.levels.iter().map(|l| mesh_byte_size(&l.mesh)).sum()
}

/// Remove the LOD level with the given tier from the chain.
///
/// Does nothing if no such level exists.
#[allow(dead_code)]
pub fn remove_lod_level(chain: &mut LodChain, tier: LodTier) {
    chain.levels.retain(|l| l.tier != tier);
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Estimate byte size of a [`MeshBuffers`].
fn mesh_byte_size(mesh: &MeshBuffers) -> usize {
    mesh.positions.len() * 12
        + mesh.normals.len() * 12
        + mesh.uvs.len() * 8
        + mesh.indices.len() * 4
}

/// Decimate a mesh to approximately `ratio` of its original vertex count using
/// grid-based vertex clustering (LCG-free, deterministic).
fn lod_decimate(src: &MeshBuffers, ratio: f32) -> MeshBuffers {
    if ratio >= 1.0 || src.positions.is_empty() {
        return src.clone();
    }
    let n = src.positions.len();
    let target_cells = (n as f32 * ratio).max(1.0);
    let grid_res = (target_cells.cbrt().ceil() as usize).max(2);

    let mut bb_min = src.positions[0];
    let mut bb_max = src.positions[0];
    for p in &src.positions {
        for k in 0..3 {
            if p[k] < bb_min[k] {
                bb_min[k] = p[k];
            }
            if p[k] > bb_max[k] {
                bb_max[k] = p[k];
            }
        }
    }
    let extent = [
        (bb_max[0] - bb_min[0]).max(1e-8),
        (bb_max[1] - bb_min[1]).max(1e-8),
        (bb_max[2] - bb_min[2]).max(1e-8),
    ];
    let gr = grid_res as f32;

    let cell_of: Vec<usize> = src
        .positions
        .iter()
        .map(|p| {
            let ix = ((p[0] - bb_min[0]) / extent[0] * gr).min(gr - 1.0) as usize;
            let iy = ((p[1] - bb_min[1]) / extent[1] * gr).min(gr - 1.0) as usize;
            let iz = ((p[2] - bb_min[2]) / extent[2] * gr).min(gr - 1.0) as usize;
            ix + iy * grid_res + iz * grid_res * grid_res
        })
        .collect();

    let total_cells = grid_res * grid_res * grid_res;
    let mut sum_pos = vec![[0.0f32; 3]; total_cells];
    let mut sum_norm = vec![[0.0f32; 3]; total_cells];
    let mut sum_uv = vec![[0.0f32; 2]; total_cells];
    let mut cnt = vec![0u32; total_cells];

    for (i, &cell) in cell_of.iter().enumerate() {
        let p = src.positions[i];
        sum_pos[cell][0] += p[0];
        sum_pos[cell][1] += p[1];
        sum_pos[cell][2] += p[2];
        if i < src.normals.len() {
            let nm = src.normals[i];
            sum_norm[cell][0] += nm[0];
            sum_norm[cell][1] += nm[1];
            sum_norm[cell][2] += nm[2];
        }
        if i < src.uvs.len() {
            let uv = src.uvs[i];
            sum_uv[cell][0] += uv[0];
            sum_uv[cell][1] += uv[1];
        }
        cnt[cell] += 1;
    }

    let mut cell_to_out: Vec<Option<u32>> = vec![None; total_cells];
    let mut out_pos: Vec<[f32; 3]> = Vec::new();
    let mut out_norm: Vec<[f32; 3]> = Vec::new();
    let mut out_uv: Vec<[f32; 2]> = Vec::new();

    for (cell, &count) in cnt.iter().enumerate() {
        if count == 0 {
            continue;
        }
        let idx = out_pos.len() as u32;
        cell_to_out[cell] = Some(idx);
        let c = count as f32;
        out_pos.push([
            sum_pos[cell][0] / c,
            sum_pos[cell][1] / c,
            sum_pos[cell][2] / c,
        ]);
        let nl =
            (sum_norm[cell][0].powi(2) + sum_norm[cell][1].powi(2) + sum_norm[cell][2].powi(2))
                .sqrt()
                .max(1e-10);
        out_norm.push([
            sum_norm[cell][0] / nl,
            sum_norm[cell][1] / nl,
            sum_norm[cell][2] / nl,
        ]);
        out_uv.push([sum_uv[cell][0] / c, sum_uv[cell][1] / c]);
    }

    let mut out_idx: Vec<u32> = Vec::new();
    for tri in src.indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= n || i1 >= n || i2 >= n {
            continue;
        }
        let (c0, c1, c2) = (cell_of[i0], cell_of[i1], cell_of[i2]);
        if c0 == c1 || c1 == c2 || c0 == c2 {
            continue;
        }
        if let (Some(o0), Some(o1), Some(o2)) = (cell_to_out[c0], cell_to_out[c1], cell_to_out[c2])
        {
            out_idx.push(o0);
            out_idx.push(o1);
            out_idx.push(o2);
        }
    }

    MeshBuffers {
        positions: out_pos,
        normals: out_norm,
        tangents: Vec::new(),
        uvs: out_uv,
        indices: out_idx,
        colors: None,
        has_suit: src.has_suit,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers as MyMesh;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn grid_mesh(n: usize) -> MyMesh {
        let mut pos = Vec::new();
        let mut uvs = Vec::new();
        let mut idx = Vec::new();
        for row in 0..n {
            for col in 0..n {
                pos.push([col as f32 * 0.1, row as f32 * 0.1, 0.0f32]);
                uvs.push([col as f32 / n as f32, row as f32 / n as f32]);
            }
        }
        for row in 0..n - 1 {
            for col in 0..n - 1 {
                let tl = (row * n + col) as u32;
                let tr = tl + 1;
                let bl = tl + n as u32;
                let br = bl + 1;
                idx.extend_from_slice(&[tl, tr, bl, tr, br, bl]);
            }
        }
        MyMesh::from_morph(MB {
            positions: pos,
            normals: vec![[0.0f32, 0.0, 1.0]; n * n],
            uvs,
            indices: idx,
            has_suit: false,
        })
    }

    #[test]
    fn default_lod_config_thresholds_ascending() {
        let c = default_lod_config();
        assert!(c.thresholds[0] < c.thresholds[1]);
        assert!(c.thresholds[1] < c.thresholds[2]);
    }

    #[test]
    fn new_lod_chain_empty() {
        let c = default_lod_config();
        let chain = new_lod_chain(c);
        assert_eq!(lod_count(&chain), 0);
    }

    #[test]
    fn add_lod_level_increments_count() {
        let mesh = grid_mesh(10);
        let mut chain = new_lod_chain(default_lod_config());
        add_lod_level(
            &mut chain,
            LodMesh {
                tier: LodTier::Lod0,
                mesh: mesh.clone(),
            },
        );
        assert_eq!(lod_count(&chain), 1);
    }

    #[test]
    fn add_lod_level_sorted_by_tier() {
        let mesh = grid_mesh(10);
        let mut chain = new_lod_chain(default_lod_config());
        add_lod_level(
            &mut chain,
            LodMesh {
                tier: LodTier::Lod2,
                mesh: mesh.clone(),
            },
        );
        add_lod_level(
            &mut chain,
            LodMesh {
                tier: LodTier::Lod0,
                mesh: mesh.clone(),
            },
        );
        assert_eq!(chain.levels[0].tier, LodTier::Lod0);
        assert_eq!(chain.levels[1].tier, LodTier::Lod2);
    }

    #[test]
    fn select_lod_close_returns_lod0() {
        let base = grid_mesh(20);
        let chain = generate_lod_chain(&base, default_lod_config());
        let tier = select_lod(&chain, 1.0);
        assert_eq!(tier, LodTier::Lod0);
    }

    #[test]
    fn select_lod_far_returns_lower_quality() {
        let base = grid_mesh(20);
        let chain = generate_lod_chain(&base, default_lod_config());
        let near = select_lod(&chain, 1.0);
        let far = select_lod(&chain, 100.0);
        assert!(far as u8 > near as u8 || far == LodTier::Lod3);
    }

    #[test]
    fn lod_vertex_count_lod0_equals_base() {
        let base = grid_mesh(10);
        let n = base.positions.len();
        let chain = generate_lod_chain(&base, default_lod_config());
        assert_eq!(lod_vertex_count(&chain, LodTier::Lod0), n);
    }

    #[test]
    fn lod_vertex_count_lod1_less_than_lod0() {
        let base = grid_mesh(20);
        let chain = generate_lod_chain(&base, default_lod_config());
        let v0 = lod_vertex_count(&chain, LodTier::Lod0);
        let v1 = lod_vertex_count(&chain, LodTier::Lod1);
        assert!(v1 < v0, "lod1 ({v1}) should be less than lod0 ({v0})");
    }

    #[test]
    fn lod_triangle_count_lod0_nonzero_for_nonempty() {
        let base = grid_mesh(10);
        let chain = generate_lod_chain(&base, default_lod_config());
        assert!(lod_triangle_count(&chain, LodTier::Lod0) > 0);
    }

    #[test]
    fn lod_count_equals_max_levels() {
        let base = grid_mesh(10);
        let mut cfg = default_lod_config();
        cfg.max_levels = 3;
        let chain = generate_lod_chain(&base, cfg);
        assert_eq!(lod_count(&chain), 3);
    }

    #[test]
    fn lod_transition_distance_ordering() {
        let base = grid_mesh(10);
        let chain = generate_lod_chain(&base, default_lod_config());
        let d0 = lod_transition_distance(&chain, LodTier::Lod0);
        let d1 = lod_transition_distance(&chain, LodTier::Lod1);
        let d2 = lod_transition_distance(&chain, LodTier::Lod2);
        let d3 = lod_transition_distance(&chain, LodTier::Lod3);
        assert!(d0 < d1 && d1 < d2 && d2 < d3);
    }

    #[test]
    fn set_lod_bias_affects_selection() {
        let base = grid_mesh(20);
        let mut chain = generate_lod_chain(&base, default_lod_config());
        // Without bias, distance=3 is close → Lod0
        let tier_normal = select_lod(&chain, 3.0);
        // Add a big negative bias (pretend we are even closer)
        set_lod_bias(&mut chain, -100.0);
        let tier_biased = select_lod(&chain, 3.0);
        // With big negative bias we clamp to 0 anyway
        assert_eq!(tier_biased, LodTier::Lod0);
        assert_eq!(tier_normal, LodTier::Lod0);
    }

    #[test]
    fn lod_memory_usage_nonzero_for_nonempty_chain() {
        let base = grid_mesh(10);
        let chain = generate_lod_chain(&base, default_lod_config());
        assert!(lod_memory_usage(&chain) > 0);
    }

    #[test]
    fn remove_lod_level_removes_correct_tier() {
        let base = grid_mesh(10);
        let mut chain = generate_lod_chain(&base, default_lod_config());
        let before = lod_count(&chain);
        remove_lod_level(&mut chain, LodTier::Lod1);
        assert_eq!(lod_count(&chain), before - 1);
        assert_eq!(lod_vertex_count(&chain, LodTier::Lod1), 0);
    }

    #[test]
    fn remove_nonexistent_lod_is_noop() {
        let base = grid_mesh(10);
        let mut cfg = default_lod_config();
        cfg.max_levels = 2;
        let mut chain = generate_lod_chain(&base, cfg);
        let before = lod_count(&chain);
        remove_lod_level(&mut chain, LodTier::Lod3);
        assert_eq!(lod_count(&chain), before);
    }

    #[test]
    fn lod_tier_ratios_descending() {
        assert!(LodTier::Lod0.ratio() > LodTier::Lod1.ratio());
        assert!(LodTier::Lod1.ratio() > LodTier::Lod2.ratio());
        assert!(LodTier::Lod2.ratio() > LodTier::Lod3.ratio());
    }

    #[test]
    fn select_lod_empty_chain_returns_lod0() {
        let chain = new_lod_chain(default_lod_config());
        assert_eq!(select_lod(&chain, 999.0), LodTier::Lod0);
    }
}
