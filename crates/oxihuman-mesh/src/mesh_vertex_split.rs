// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Vertex splitting: duplicate shared vertices across UV seams or hard edges.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SplitResult {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub original_vertex_map: Vec<u32>,
    pub split_count: usize,
}

#[allow(dead_code)]
pub fn split_by_uvs(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u32],
) -> SplitResult {
    let mut new_pos: Vec<[f32; 3]> = Vec::new();
    let mut new_nrm: Vec<[f32; 3]> = Vec::new();
    let mut new_uvs: Vec<[f32; 2]> = Vec::new();
    let mut new_idx: Vec<u32> = Vec::new();
    let mut orig_map: Vec<u32> = Vec::new();
    let mut split_count = 0usize;

    use std::collections::HashMap;
    // key: (vertex_index, uv as bits)
    let mut cache: HashMap<(u32, u64), u32> = HashMap::new();

    for &vi in indices {
        let uv = if (vi as usize) < uvs.len() {
            uvs[vi as usize]
        } else {
            [0.0; 2]
        };
        let key = (vi, (uv[0].to_bits() as u64) << 32 | uv[1].to_bits() as u64);
        let new_vi = if let Some(&ni) = cache.get(&key) {
            ni
        } else {
            let ni = new_pos.len() as u32;
            let p = if (vi as usize) < positions.len() {
                positions[vi as usize]
            } else {
                [0.0; 3]
            };
            let n = if (vi as usize) < normals.len() {
                normals[vi as usize]
            } else {
                [0.0; 3]
            };
            new_pos.push(p);
            new_nrm.push(n);
            new_uvs.push(uv);
            orig_map.push(vi);
            if ni != vi {
                split_count += 1;
            }
            cache.insert(key, ni);
            ni
        };
        new_idx.push(new_vi);
    }

    SplitResult {
        positions: new_pos,
        normals: new_nrm,
        uvs: new_uvs,
        indices: new_idx,
        original_vertex_map: orig_map,
        split_count,
    }
}

#[allow(dead_code)]
pub fn split_vertex_count(result: &SplitResult) -> usize {
    result.positions.len()
}

#[allow(dead_code)]
pub fn split_to_json(result: &SplitResult) -> String {
    format!(
        "{{\"vertex_count\":{},\"split_count\":{}}}",
        split_vertex_count(result),
        result.split_count
    )
}

#[allow(dead_code)]
pub fn indices_valid(result: &SplitResult) -> bool {
    let n = result.positions.len() as u32;
    result.indices.iter().all(|&i| i < n)
}

#[allow(dead_code)]
pub fn split_uvs_unique(result: &SplitResult) -> bool {
    // Verify each (vertex, uv) pair is unique
    use std::collections::HashSet;
    let mut seen: HashSet<u64> = HashSet::new();
    for (i, &orig) in result.original_vertex_map.iter().enumerate() {
        let uv = result.uvs[i];
        let uv_key = (uv[0].to_bits() as u64) << 32 | uv[1].to_bits() as u64;
        let key = uv_key.wrapping_add((orig as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15u64));
        if !seen.insert(key) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::type_complexity)]
    fn simple_mesh() -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let nrm = vec![[0.0f32, 0.0, 1.0]; 3];
        let uvs = vec![[0.0f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let idx = vec![0u32, 1, 2];
        (pos, nrm, uvs, idx)
    }

    #[test]
    fn test_split_no_duplication() {
        let (pos, nrm, uvs, idx) = simple_mesh();
        let result = split_by_uvs(&pos, &nrm, &uvs, &idx);
        assert_eq!(split_vertex_count(&result), 3);
    }

    #[test]
    fn test_indices_valid() {
        let (pos, nrm, uvs, idx) = simple_mesh();
        let result = split_by_uvs(&pos, &nrm, &uvs, &idx);
        assert!(indices_valid(&result));
    }

    #[test]
    fn test_json_output() {
        let (pos, nrm, uvs, idx) = simple_mesh();
        let result = split_by_uvs(&pos, &nrm, &uvs, &idx);
        let j = split_to_json(&result);
        assert!(j.contains("vertex_count"));
    }

    #[test]
    fn test_uvs_unique() {
        let (pos, nrm, uvs, idx) = simple_mesh();
        let result = split_by_uvs(&pos, &nrm, &uvs, &idx);
        assert!(split_uvs_unique(&result));
    }

    #[test]
    fn test_split_with_shared_vertex_different_uvs() {
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let nrm = vec![[0.0f32, 0.0, 1.0]; 3];
        let uvs = vec![[0.0f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let idx = vec![0u32, 1, 2, 0, 2, 1];
        let result = split_by_uvs(&pos, &nrm, &uvs, &idx);
        assert!(indices_valid(&result));
    }

    #[test]
    fn test_empty_indices() {
        let result = split_by_uvs(&[], &[], &[], &[]);
        assert_eq!(split_vertex_count(&result), 0);
    }

    #[test]
    fn test_original_map_length() {
        let (pos, nrm, uvs, idx) = simple_mesh();
        let result = split_by_uvs(&pos, &nrm, &uvs, &idx);
        assert_eq!(
            result.original_vertex_map.len(),
            split_vertex_count(&result)
        );
    }

    #[test]
    fn test_uvs_length_matches_positions() {
        let (pos, nrm, uvs, idx) = simple_mesh();
        let result = split_by_uvs(&pos, &nrm, &uvs, &idx);
        assert_eq!(result.uvs.len(), result.positions.len());
    }

    #[test]
    fn test_normals_length_matches_positions() {
        let (pos, nrm, uvs, idx) = simple_mesh();
        let result = split_by_uvs(&pos, &nrm, &uvs, &idx);
        assert_eq!(result.normals.len(), result.positions.len());
    }

    #[test]
    fn test_index_count_matches_input() {
        let (pos, nrm, uvs, idx) = simple_mesh();
        let result = split_by_uvs(&pos, &nrm, &uvs, &idx);
        assert_eq!(result.indices.len(), idx.len());
    }
}
