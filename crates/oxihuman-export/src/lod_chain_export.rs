// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Level-of-detail mesh export with automatic LOD chain generation.

// ── LodExportConfig ───────────────────────────────────────────────────────────

/// Configuration for LOD chain generation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodExportConfig {
    /// Number of LOD levels to generate (including the base level 0).
    pub lod_count: usize,
    /// Per-level reduction ratios (0.0 = fully reduced, 1.0 = original).
    /// Length should equal `lod_count`.
    pub reduction_ratios: Vec<f32>,
    /// Vertices closer than this threshold are merged during reduction.
    pub merge_threshold: f32,
}

// ── LodChainLevel ─────────────────────────────────────────────────────────────

/// Description of one level in a LOD chain.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodChainLevel {
    /// Zero-based LOD index (0 = highest detail).
    pub level: usize,
    /// Vertex count after reduction.
    pub vertex_count: usize,
    /// Triangle count after reduction.
    pub triangle_count: usize,
    /// Reduction ratio applied (1.0 = no reduction).
    pub reduction: f32,
}

// ── LodChain ──────────────────────────────────────────────────────────────────

/// A complete LOD chain for one mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodChain {
    /// Ordered levels, level 0 first (highest quality).
    pub levels: Vec<LodChainLevel>,
    /// Vertex count of the original (unreduced) mesh.
    pub base_vertex_count: usize,
    /// Triangle count of the original mesh.
    pub base_triangle_count: usize,
}

// ── LodExportResult ───────────────────────────────────────────────────────────

/// Result of a LOD chain export operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodExportResult {
    /// The generated LOD chain.
    pub chain: LodChain,
    /// Total number of LOD levels generated.
    pub total_levels: usize,
    /// Whether the export completed without error.
    pub success: bool,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return a sensible default [`LodExportConfig`] with 4 LOD levels.
#[allow(dead_code)]
pub fn default_lod_export_config() -> LodExportConfig {
    LodExportConfig {
        lod_count: 4,
        reduction_ratios: vec![1.0, 0.5, 0.25, 0.125],
        merge_threshold: 1e-4,
    }
}

/// Generate a [`LodExportResult`] from positions and triangle indices.
///
/// Each LOD level reduces the triangle count by applying the corresponding
/// `reduction_ratios` entry.  For LOD level 0 (`ratio == 1.0`) the mesh is
/// kept as-is.
#[allow(dead_code)]
pub fn generate_lod_chain(
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    cfg: &LodExportConfig,
) -> LodExportResult {
    let base_vertex_count = positions.len();
    let base_triangle_count = triangles.len();

    let mut levels = Vec::with_capacity(cfg.lod_count);
    let ratios = &cfg.reduction_ratios;

    for i in 0..cfg.lod_count {
        let ratio = if i < ratios.len() { ratios[i] } else { 0.125 };
        let ratio_clamped = ratio.clamp(0.0, 1.0);

        // Simple proportional reduction: keep `ratio` fraction of triangles,
        // approximate vertex count accordingly.
        let tri_count = ((base_triangle_count as f32 * ratio_clamped).ceil() as usize).max(1);
        // Vertices scale roughly as sqrt of triangle ratio for a typical mesh.
        let vtx_count =
            ((base_vertex_count as f32 * ratio_clamped.sqrt()).ceil() as usize).max(1);

        levels.push(LodChainLevel {
            level: i,
            vertex_count: vtx_count,
            triangle_count: tri_count,
            reduction: ratio_clamped,
        });
    }

    let total_levels = levels.len();
    let chain = LodChain {
        levels,
        base_vertex_count,
        base_triangle_count,
    };

    LodExportResult {
        chain,
        total_levels,
        success: true,
    }
}

/// Serialise a [`LodChainLevel`] to a JSON string.
#[allow(dead_code)]
pub fn lod_chain_level_to_json(l: &LodChainLevel) -> String {
    format!(
        r#"{{"level":{},"vertex_count":{},"triangle_count":{},"reduction":{:.4}}}"#,
        l.level, l.vertex_count, l.triangle_count, l.reduction
    )
}

/// Serialise a [`LodChain`] to a JSON string.
#[allow(dead_code)]
pub fn lod_chain_to_json(chain: &LodChain) -> String {
    let levels_json: Vec<String> = chain.levels.iter().map(lod_chain_level_to_json).collect();
    format!(
        r#"{{"base_vertex_count":{},"base_triangle_count":{},"levels":[{}]}}"#,
        chain.base_vertex_count,
        chain.base_triangle_count,
        levels_json.join(",")
    )
}

/// Serialise a [`LodExportResult`] to a JSON string.
#[allow(dead_code)]
pub fn lod_export_result_to_json(r: &LodExportResult) -> String {
    format!(
        r#"{{"total_levels":{},"success":{},"chain":{}}}"#,
        r.total_levels,
        r.success,
        lod_chain_to_json(&r.chain)
    )
}

/// Return the number of LOD levels in `result`.
#[allow(dead_code)]
pub fn lod_level_count(result: &LodExportResult) -> usize {
    result.total_levels
}

/// Return the reduction ratio at `level`, or 0.0 if the level is out of range.
#[allow(dead_code)]
pub fn lod_reduction_at(result: &LodExportResult, level: usize) -> f32 {
    result
        .chain
        .levels
        .get(level)
        .map(|l| l.reduction)
        .unwrap_or(0.0)
}

/// Sum the triangle counts across all LOD levels.
#[allow(dead_code)]
pub fn total_triangle_budget(result: &LodExportResult) -> usize {
    result
        .chain
        .levels
        .iter()
        .map(|l| l.triangle_count)
        .sum()
}

/// Serialise a [`LodExportConfig`] to a JSON string.
#[allow(dead_code)]
pub fn lod_config_to_json(cfg: &LodExportConfig) -> String {
    let ratios: Vec<String> = cfg
        .reduction_ratios
        .iter()
        .map(|r| format!("{:.4}", r))
        .collect();
    format!(
        r#"{{"lod_count":{},"merge_threshold":{:.6},"reduction_ratios":[{}]}}"#,
        cfg.lod_count,
        cfg.merge_threshold,
        ratios.join(",")
    )
}

/// Return `true` if the LOD chain has at least one level and level 0 has the
/// highest triangle count.
#[allow(dead_code)]
pub fn validate_lod_chain(chain: &LodChain) -> bool {
    if chain.levels.is_empty() {
        return false;
    }
    let first = &chain.levels[0];
    chain
        .levels
        .iter()
        .all(|l| l.triangle_count <= first.triangle_count)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]
    }

    fn sample_triangles() -> Vec<[u32; 3]> {
        vec![[0, 1, 2], [1, 3, 2]]
    }

    #[test]
    fn default_config_has_four_levels() {
        let cfg = default_lod_export_config();
        assert_eq!(cfg.lod_count, 4);
        assert_eq!(cfg.reduction_ratios.len(), 4);
    }

    #[test]
    fn generate_lod_chain_level_count_matches_config() {
        let pos = sample_positions();
        let tri = sample_triangles();
        let cfg = default_lod_export_config();
        let result = generate_lod_chain(&pos, &tri, &cfg);
        assert_eq!(result.total_levels, cfg.lod_count);
        assert_eq!(result.chain.levels.len(), cfg.lod_count);
    }

    #[test]
    fn generate_lod_chain_level0_full_resolution() {
        let pos = sample_positions();
        let tri = sample_triangles();
        let cfg = default_lod_export_config();
        let result = generate_lod_chain(&pos, &tri, &cfg);
        // LOD 0 has ratio 1.0 → triangle_count == base
        assert_eq!(
            result.chain.levels[0].triangle_count,
            result.chain.base_triangle_count
        );
    }

    #[test]
    fn lod_level_count_matches_total_levels() {
        let pos = sample_positions();
        let tri = sample_triangles();
        let cfg = default_lod_export_config();
        let result = generate_lod_chain(&pos, &tri, &cfg);
        assert_eq!(lod_level_count(&result), result.total_levels);
    }

    #[test]
    fn lod_reduction_at_out_of_range_returns_zero() {
        let pos = sample_positions();
        let tri = sample_triangles();
        let cfg = default_lod_export_config();
        let result = generate_lod_chain(&pos, &tri, &cfg);
        assert!((lod_reduction_at(&result, 999) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn total_triangle_budget_positive() {
        let pos = sample_positions();
        let tri = sample_triangles();
        let cfg = default_lod_export_config();
        let result = generate_lod_chain(&pos, &tri, &cfg);
        assert!(total_triangle_budget(&result) > 0);
    }

    #[test]
    fn lod_chain_to_json_contains_base_vertex_count() {
        let pos = sample_positions();
        let tri = sample_triangles();
        let cfg = default_lod_export_config();
        let result = generate_lod_chain(&pos, &tri, &cfg);
        let json = lod_chain_to_json(&result.chain);
        assert!(
            json.contains(&format!(
                "\"base_vertex_count\":{}",
                result.chain.base_vertex_count
            ))
        );
    }

    #[test]
    fn validate_lod_chain_valid() {
        let pos = sample_positions();
        let tri = sample_triangles();
        let cfg = default_lod_export_config();
        let result = generate_lod_chain(&pos, &tri, &cfg);
        assert!(validate_lod_chain(&result.chain));
    }

    #[test]
    fn lod_config_to_json_contains_lod_count() {
        let cfg = default_lod_export_config();
        let json = lod_config_to_json(&cfg);
        assert!(json.contains("\"lod_count\":4"));
    }

    #[test]
    fn lod_export_result_to_json_contains_success() {
        let pos = sample_positions();
        let tri = sample_triangles();
        let cfg = default_lod_export_config();
        let result = generate_lod_chain(&pos, &tri, &cfg);
        let json = lod_export_result_to_json(&result);
        assert!(json.contains("\"success\":true"));
    }
}
