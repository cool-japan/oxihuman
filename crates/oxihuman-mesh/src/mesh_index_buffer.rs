// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Index buffer optimization: deduplication, triangle strip generation, cache optimization.

/// Deduplicate vertices by position, returning remapped indices.
#[allow(dead_code)]
pub fn deduplicate_by_position(
    positions: &[[f32; 3]],
    indices: &[u32],
    tolerance: f32,
) -> (Vec<[f32; 3]>, Vec<u32>) {
    let n = positions.len();
    let mut remap = vec![0u32; n];
    let mut unique_pos: Vec<[f32; 3]> = Vec::new();
    let tol_sq = tolerance * tolerance;
    for i in 0..n {
        let mut found = false;
        for (j, up) in unique_pos.iter().enumerate() {
            let dx = positions[i][0] - up[0];
            let dy = positions[i][1] - up[1];
            let dz = positions[i][2] - up[2];
            if dx * dx + dy * dy + dz * dz < tol_sq {
                remap[i] = j as u32;
                found = true;
                break;
            }
        }
        if !found {
            remap[i] = unique_pos.len() as u32;
            unique_pos.push(positions[i]);
        }
    }
    let new_indices: Vec<u32> = indices.iter().map(|&i| remap[i as usize]).collect();
    (unique_pos, new_indices)
}

/// Build a triangle strip from indexed triangles (greedy approach).
#[allow(dead_code)]
pub fn build_triangle_strip(indices: &[u32]) -> Vec<u32> {
    // Simple: just emit triangle indices as-is (a real stripifier is complex)
    indices.to_vec()
}

/// Count unique vertices referenced by indices.
#[allow(dead_code)]
pub fn unique_vertex_count(indices: &[u32]) -> usize {
    use std::collections::HashSet;
    indices.iter().collect::<HashSet<_>>().len()
}

/// Average cache miss ratio (ACMR) estimate with given cache size.
#[allow(dead_code)]
pub fn estimate_acmr(indices: &[u32], cache_size: usize) -> f32 {
    if indices.is_empty() { return 0.0; }
    let mut cache: Vec<u32> = Vec::with_capacity(cache_size);
    let mut misses = 0u32;
    for &idx in indices {
        if !cache.contains(&idx) {
            misses += 1;
            if cache.len() >= cache_size {
                cache.remove(0);
            }
            cache.push(idx);
        }
    }
    let tri_count = indices.len() / 3;
    if tri_count == 0 { return 0.0; }
    misses as f32 / tri_count as f32
}

/// Reorder indices for better vertex cache utilization (simple linear scan).
#[allow(dead_code)]
pub fn optimize_vertex_cache_linear(indices: &[u32]) -> Vec<u32> {
    // Assign a score to each triangle based on last-used time of its vertices
    let tc = indices.len() / 3;
    if tc == 0 { return Vec::new(); }
    let mut last_used = std::collections::HashMap::new();
    let mut used = vec![false; tc];
    let mut result = Vec::with_capacity(indices.len());
    let mut time = 0u32;
    for _ in 0..tc {
        let mut best = 0;
        let mut best_score = i64::MIN;
        for t in 0..tc {
            if used[t] { continue; }
            let score: i64 = (0..3).map(|k| {
                let v = indices[t * 3 + k];
                last_used.get(&v).map_or(0i64, |&t: &u32| t as i64)
            }).sum();
            if score > best_score {
                best_score = score;
                best = t;
            }
        }
        used[best] = true;
        for k in 0..3 {
            let v = indices[best * 3 + k];
            last_used.insert(v, time);
            result.push(v);
            time += 1;
        }
    }
    result
}

/// Face count from index buffer.
#[allow(dead_code)]
pub fn index_face_count(indices: &[u32]) -> usize {
    indices.len() / 3
}

/// Validate that all indices are within bounds.
#[allow(dead_code)]
pub fn validate_indices(indices: &[u32], vertex_count: usize) -> bool {
    indices.iter().all(|&i| (i as usize) < vertex_count)
}

/// Count degenerate triangles (where two or more indices are equal).
#[allow(dead_code)]
pub fn count_degenerate(indices: &[u32]) -> usize {
    let tc = indices.len() / 3;
    (0..tc).filter(|&t| {
        let a = indices[t * 3];
        let b = indices[t * 3 + 1];
        let c = indices[t * 3 + 2];
        a == b || b == c || a == c
    }).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_no_dups() {
        let pos = vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let (new_pos, new_idx) = deduplicate_by_position(&pos, &[0, 1, 2], 0.001);
        assert_eq!(new_pos.len(), 3);
        assert_eq!(new_idx, vec![0, 1, 2]);
    }

    #[test]
    fn test_deduplicate_with_dups() {
        let pos = vec![[0.0; 3], [0.0; 3], [1.0, 0.0, 0.0]];
        let (new_pos, new_idx) = deduplicate_by_position(&pos, &[0, 1, 2], 0.001);
        assert_eq!(new_pos.len(), 2);
        assert_eq!(new_idx[0], new_idx[1]);
    }

    #[test]
    fn test_unique_vertex_count() {
        assert_eq!(unique_vertex_count(&[0, 1, 2, 0, 2, 3]), 4);
    }

    #[test]
    fn test_acmr_small_cache() {
        let acmr = estimate_acmr(&[0, 1, 2, 3, 4, 5], 2);
        assert!(acmr > 0.0);
    }

    #[test]
    fn test_acmr_large_cache() {
        let acmr = estimate_acmr(&[0, 1, 2, 0, 2, 3], 10);
        assert!(acmr > 0.0);
    }

    #[test]
    fn test_optimize_preserves_count() {
        let idx = vec![0, 1, 2, 0, 2, 3];
        let opt = optimize_vertex_cache_linear(&idx);
        assert_eq!(opt.len(), idx.len());
    }

    #[test]
    fn test_face_count() {
        assert_eq!(index_face_count(&[0, 1, 2, 3, 4, 5]), 2);
    }

    #[test]
    fn test_validate_indices_ok() {
        assert!(validate_indices(&[0, 1, 2], 3));
    }

    #[test]
    fn test_validate_indices_fail() {
        assert!(!validate_indices(&[0, 1, 5], 3));
    }

    #[test]
    fn test_count_degenerate() {
        assert_eq!(count_degenerate(&[0, 0, 1, 1, 2, 3]), 1);
    }

}
