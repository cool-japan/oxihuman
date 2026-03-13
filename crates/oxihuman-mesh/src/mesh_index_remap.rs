// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Index remapping utilities for mesh index buffers.

/// Result of index remapping.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RemapResult {
    pub new_indices: Vec<u32>,
    pub old_to_new: Vec<u32>,
    pub new_to_old: Vec<u32>,
}

/// Build a compact remap: remove unused vertex indices and remap.
#[allow(dead_code)]
pub fn compact_remap(indices: &[u32], vertex_count: usize) -> RemapResult {
    let mut used = vec![false; vertex_count];
    for &i in indices {
        if (i as usize) < vertex_count {
            used[i as usize] = true;
        }
    }

    let mut old_to_new = vec![u32::MAX; vertex_count];
    let mut new_to_old = Vec::new();
    let mut next = 0u32;
    for (i, &u) in used.iter().enumerate() {
        if u {
            old_to_new[i] = next;
            new_to_old.push(i as u32);
            next += 1;
        }
    }
    let new_indices: Vec<u32> = indices.iter().map(|&i| old_to_new[i as usize]).collect();
    RemapResult {
        new_indices,
        old_to_new,
        new_to_old,
    }
}

/// Apply a remap table to an index buffer.
#[allow(dead_code)]
pub fn apply_remap(indices: &[u32], remap: &[u32]) -> Vec<u32> {
    indices.iter().map(|&i| remap[i as usize]).collect()
}

/// Count how many unique vertex indices are used.
#[allow(dead_code)]
pub fn used_index_count(indices: &[u32]) -> usize {
    let mut set = std::collections::HashSet::new();
    for &i in indices {
        set.insert(i);
    }
    set.len()
}

/// Count unused vertices (vertex_count - used).
#[allow(dead_code)]
pub fn unused_index_count(indices: &[u32], vertex_count: usize) -> usize {
    vertex_count.saturating_sub(used_index_count(indices))
}

/// Compute the maximum index value.
#[allow(dead_code)]
pub fn max_index(indices: &[u32]) -> Option<u32> {
    indices.iter().copied().max()
}

/// Validate that all indices are within bounds.
#[allow(dead_code)]
pub fn indices_in_bounds(indices: &[u32], vertex_count: usize) -> bool {
    indices.iter().all(|&i| (i as usize) < vertex_count)
}

/// Reverse the winding order of all triangles.
#[allow(dead_code)]
pub fn reverse_winding(indices: &[u32]) -> Vec<u32> {
    let tri_count = indices.len() / 3;
    let mut result = Vec::with_capacity(indices.len());
    for t in 0..tri_count {
        result.push(indices[t * 3]);
        result.push(indices[t * 3 + 2]);
        result.push(indices[t * 3 + 1]);
    }
    result
}

/// Convert result to JSON.
#[allow(dead_code)]
pub fn remap_result_to_json(result: &RemapResult) -> String {
    format!(
        "{{\"new_vertex_count\":{},\"index_count\":{}}}",
        result.new_to_old.len(),
        result.new_indices.len(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_remap_all_used() {
        let r = compact_remap(&[0, 1, 2], 3);
        assert_eq!(r.new_to_old.len(), 3);
        assert_eq!(r.new_indices, vec![0, 1, 2]);
    }

    #[test]
    fn test_compact_remap_gap() {
        // vertex 1 is unused
        let r = compact_remap(&[0, 2, 3], 4);
        assert_eq!(r.new_to_old.len(), 3);
        assert!(r.new_indices.iter().all(|&i| i < 3));
    }

    #[test]
    fn test_apply_remap() {
        let remap = vec![2, 0, 1];
        let result = apply_remap(&[0, 1, 2], &remap);
        assert_eq!(result, vec![2, 0, 1]);
    }

    #[test]
    fn test_used_index_count() {
        assert_eq!(used_index_count(&[0, 1, 2, 0, 1, 2]), 3);
    }

    #[test]
    fn test_unused_index_count() {
        assert_eq!(unused_index_count(&[0, 2], 4), 2);
    }

    #[test]
    fn test_max_index() {
        assert_eq!(max_index(&[3, 1, 4, 1, 5]), Some(5));
        assert_eq!(max_index(&[]), None);
    }

    #[test]
    fn test_indices_in_bounds() {
        assert!(indices_in_bounds(&[0, 1, 2], 3));
        assert!(!indices_in_bounds(&[0, 1, 3], 3));
    }

    #[test]
    fn test_reverse_winding() {
        let r = reverse_winding(&[0, 1, 2, 3, 4, 5]);
        assert_eq!(r, vec![0, 2, 1, 3, 5, 4]);
    }

    #[test]
    fn test_remap_result_to_json() {
        let r = compact_remap(&[0, 1, 2], 3);
        let json = remap_result_to_json(&r);
        assert!(json.contains("\"new_vertex_count\":3"));
    }

    #[test]
    fn test_empty_indices() {
        let r = compact_remap(&[], 5);
        assert_eq!(r.new_to_old.len(), 0);
    }
}
