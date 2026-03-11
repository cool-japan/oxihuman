// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Vertex cache optimization analysis and export metadata.

// ── Types ─────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum VcacheAlgorithm {
    TomForsyth,
    NvTristrip,
    Linear,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VcacheConfig {
    pub cache_size: usize,
    pub acmr_target: f32,
    pub algorithm: VcacheAlgorithm,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VcacheResult {
    pub acmr: f32,
    pub atvr: f32,
    pub cache_miss_count: usize,
    pub reordered_indices: Vec<u32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VcacheStats {
    pub before_acmr: f32,
    pub after_acmr: f32,
    pub improvement_pct: f32,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_vcache_config() -> VcacheConfig {
    VcacheConfig {
        cache_size: 32,
        acmr_target: 0.5,
        algorithm: VcacheAlgorithm::TomForsyth,
    }
}

#[allow(dead_code)]
pub fn optimize_vertex_cache(
    indices: &[u32],
    vertex_count: usize,
    cfg: &VcacheConfig,
) -> VcacheResult {
    let reordered = match cfg.algorithm {
        VcacheAlgorithm::Linear => linear_reorder(indices),
        VcacheAlgorithm::TomForsyth | VcacheAlgorithm::NvTristrip => {
            tom_forsyth_reorder(indices, vertex_count, cfg.cache_size)
        }
    };
    let acmr = compute_acmr(&reordered, cfg.cache_size);
    let atvr = compute_atvr(&reordered, vertex_count);
    let misses = count_cache_misses(&reordered, cfg.cache_size);
    VcacheResult {
        acmr,
        atvr,
        cache_miss_count: misses,
        reordered_indices: reordered,
    }
}

/// Compute Average Cache Miss Ratio for the given index stream.
#[allow(dead_code)]
pub fn compute_acmr(indices: &[u32], cache_size: usize) -> f32 {
    let misses = count_cache_misses(indices, cache_size);
    let triangle_count = indices.len() / 3;
    if triangle_count == 0 {
        return 0.0;
    }
    misses as f32 / triangle_count as f32
}

/// Compute Average Transform-to-Vertex Ratio.
#[allow(dead_code)]
pub fn compute_atvr(indices: &[u32], vertex_count: usize) -> f32 {
    let unique = unique_vertex_count(indices);
    if unique == 0 {
        return 0.0;
    }
    vertex_count as f32 / unique as f32
}

#[allow(dead_code)]
pub fn vcache_result_to_json(r: &VcacheResult) -> String {
    format!(
        r#"{{"acmr":{:.6},"atvr":{:.6},"cache_miss_count":{},"reordered_index_count":{}}}"#,
        r.acmr,
        r.atvr,
        r.cache_miss_count,
        r.reordered_indices.len()
    )
}

#[allow(dead_code)]
pub fn vcache_stats_to_json(s: &VcacheStats) -> String {
    format!(
        r#"{{"before_acmr":{:.6},"after_acmr":{:.6},"improvement_pct":{:.4}}}"#,
        s.before_acmr, s.after_acmr, s.improvement_pct
    )
}

#[allow(dead_code)]
pub fn algorithm_name(cfg: &VcacheConfig) -> &'static str {
    match cfg.algorithm {
        VcacheAlgorithm::TomForsyth => "TomForsyth",
        VcacheAlgorithm::NvTristrip => "NvTristrip",
        VcacheAlgorithm::Linear => "Linear",
    }
}

#[allow(dead_code)]
pub fn compare_vcache(before: &VcacheResult, after: &VcacheResult) -> VcacheStats {
    let improvement_pct = if before.acmr > 1e-9 {
        (before.acmr - after.acmr) / before.acmr * 100.0
    } else {
        0.0
    };
    VcacheStats {
        before_acmr: before.acmr,
        after_acmr: after.acmr,
        improvement_pct,
    }
}

/// Identity reorder — returns a copy of the index slice unchanged.
#[allow(dead_code)]
pub fn linear_reorder(indices: &[u32]) -> Vec<u32> {
    indices.to_vec()
}

#[allow(dead_code)]
pub fn unique_vertex_count(indices: &[u32]) -> usize {
    let mut seen = std::collections::HashSet::new();
    for &i in indices {
        seen.insert(i);
    }
    seen.len()
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn count_cache_misses(indices: &[u32], cache_size: usize) -> usize {
    let mut cache: std::collections::VecDeque<u32> = std::collections::VecDeque::new();
    let mut misses = 0usize;
    for &idx in indices {
        if !cache.contains(&idx) {
            misses += 1;
            cache.push_back(idx);
            if cache.len() > cache_size {
                cache.pop_front();
            }
        }
    }
    misses
}

/// Very simple Tom-Forsyth-inspired greedy reorder (for stub purposes).
fn tom_forsyth_reorder(indices: &[u32], _vertex_count: usize, _cache_size: usize) -> Vec<u32> {
    // Minimal stub: sort triangles by first index to improve locality.
    let mut triangles: Vec<[u32; 3]> = indices
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();
    triangles.sort_by_key(|t| t[0]);
    triangles.iter().flat_map(|t| t.iter().copied()).collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_cache_size_32() {
        let cfg = default_vcache_config();
        assert_eq!(cfg.cache_size, 32);
    }

    #[test]
    fn algorithm_name_matches() {
        let mut cfg = default_vcache_config();
        assert_eq!(algorithm_name(&cfg), "TomForsyth");
        cfg.algorithm = VcacheAlgorithm::NvTristrip;
        assert_eq!(algorithm_name(&cfg), "NvTristrip");
        cfg.algorithm = VcacheAlgorithm::Linear;
        assert_eq!(algorithm_name(&cfg), "Linear");
    }

    #[test]
    fn linear_reorder_is_identity() {
        let idx = vec![0u32, 1, 2, 3, 4, 5];
        assert_eq!(linear_reorder(&idx), idx);
    }

    #[test]
    fn unique_vertex_count_works() {
        let idx = vec![0u32, 1, 2, 1, 2, 3];
        assert_eq!(unique_vertex_count(&idx), 4);
    }

    #[test]
    fn compute_acmr_empty() {
        assert!((compute_acmr(&[], 32) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn compute_acmr_single_triangle() {
        let idx = vec![0u32, 1, 2];
        let acmr = compute_acmr(&idx, 32);
        // 3 unique verts, 1 triangle → acmr = 3.0
        assert!((acmr - 3.0).abs() < 1e-6);
    }

    #[test]
    fn optimize_vertex_cache_linear() {
        let idx = vec![2u32, 1, 0, 3, 1, 2];
        let cfg = VcacheConfig {
            cache_size: 4,
            acmr_target: 0.5,
            algorithm: VcacheAlgorithm::Linear,
        };
        let result = optimize_vertex_cache(&idx, 4, &cfg);
        // Linear just copies
        assert_eq!(result.reordered_indices, idx);
    }

    #[test]
    fn compare_vcache_improvement() {
        let before = VcacheResult {
            acmr: 2.0,
            atvr: 1.0,
            cache_miss_count: 10,
            reordered_indices: vec![],
        };
        let after = VcacheResult {
            acmr: 1.0,
            atvr: 1.0,
            cache_miss_count: 5,
            reordered_indices: vec![],
        };
        let stats = compare_vcache(&before, &after);
        assert!((stats.improvement_pct - 50.0).abs() < 1e-4);
    }

    #[test]
    fn vcache_result_to_json_contains_acmr() {
        let r = VcacheResult {
            acmr: 1.5,
            atvr: 2.0,
            cache_miss_count: 42,
            reordered_indices: vec![0, 1, 2],
        };
        let json = vcache_result_to_json(&r);
        assert!(json.contains("acmr"));
        assert!(json.contains("42"));
    }

    #[test]
    fn vcache_stats_to_json_contains_improvement() {
        let s = VcacheStats {
            before_acmr: 2.0,
            after_acmr: 1.0,
            improvement_pct: 50.0,
        };
        let json = vcache_stats_to_json(&s);
        assert!(json.contains("improvement_pct"));
    }
}
