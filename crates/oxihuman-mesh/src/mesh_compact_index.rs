// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Compact index buffer by removing unused vertices and remapping indices.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CompactResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<[u32; 3]>,
    pub remap: Vec<u32>,
}

#[allow(dead_code)]
pub fn compact_indices(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> CompactResult {
    let mut used = vec![false; positions.len()];
    for tri in indices {
        for &v in tri { if (v as usize) < positions.len() { used[v as usize] = true; } }
    }
    let mut remap = vec![u32::MAX; positions.len()];
    let mut new_pos = Vec::new();
    for (i, &u) in used.iter().enumerate() {
        if u {
            remap[i] = new_pos.len() as u32;
            new_pos.push(positions[i]);
        }
    }
    let new_indices: Vec<[u32; 3]> = indices.iter().map(|t| [remap[t[0] as usize], remap[t[1] as usize], remap[t[2] as usize]]).collect();
    CompactResult { positions: new_pos, indices: new_indices, remap }
}

#[allow(dead_code)]
pub fn compact_removed_count(original: usize, result: &CompactResult) -> usize {
    original - result.positions.len()
}

#[allow(dead_code)]
pub fn compact_vertex_count(r: &CompactResult) -> usize { r.positions.len() }

#[allow(dead_code)]
pub fn compact_tri_count(r: &CompactResult) -> usize { r.indices.len() }

#[allow(dead_code)]
pub fn compact_validate(r: &CompactResult) -> bool {
    r.indices.iter().all(|t| t.iter().all(|&v| (v as usize) < r.positions.len()))
}

#[allow(dead_code)]
pub fn compact_to_json(r: &CompactResult) -> String {
    format!("{{\"vertices\":{},\"triangles\":{}}}", r.positions.len(), r.indices.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_compact_basic() {
        let p = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],[9.0,9.0,9.0]];
        let i = vec![[0,1,2]];
        let r = compact_indices(&p, &i);
        assert_eq!(r.positions.len(), 3);
    }
    #[test] fn test_compact_no_change() {
        let p = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]];
        let i = vec![[0,1,2]];
        let r = compact_indices(&p, &i);
        assert_eq!(r.positions.len(), 3);
    }
    #[test] fn test_removed_count() {
        let p = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],[9.0,9.0,9.0]];
        let i = vec![[0,1,2]];
        let r = compact_indices(&p, &i);
        assert_eq!(compact_removed_count(4, &r), 1);
    }
    #[test] fn test_validate() {
        let p = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]];
        let i = vec![[0,1,2]];
        let r = compact_indices(&p, &i);
        assert!(compact_validate(&r));
    }
    #[test] fn test_vertex_count() {
        let p = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]];
        let r = compact_indices(&p, &[[0,1,2]]);
        assert_eq!(compact_vertex_count(&r), 3);
    }
    #[test] fn test_tri_count() {
        let r = compact_indices(&[[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]], &[[0,1,2]]);
        assert_eq!(compact_tri_count(&r), 1);
    }
    #[test] fn test_to_json() {
        let r = compact_indices(&[[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]], &[[0,1,2]]);
        assert!(compact_to_json(&r).contains("vertices"));
    }
    #[test] fn test_empty() { let r = compact_indices(&[], &[]); assert!(r.positions.is_empty()); }
    #[test] fn test_remap_len() {
        let p = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],[9.0,9.0,9.0]];
        let r = compact_indices(&p, &[[0,1,2]]);
        assert_eq!(r.remap.len(), 4);
    }
    #[test] fn test_remap_unused() {
        let p = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],[9.0,9.0,9.0]];
        let r = compact_indices(&p, &[[0,1,2]]);
        assert_eq!(r.remap[3], u32::MAX);
    }
}
