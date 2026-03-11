// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Multi-resolution mesh representation.
#[allow(dead_code)]
pub struct MeshLevel {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub level: usize,
}

#[allow(dead_code)]
pub struct MultiResMesh {
    pub levels: Vec<MeshLevel>,
}

#[allow(dead_code)]
pub struct MultiResConfig {
    pub levels: usize,
    pub reduction_ratio: f32,
}

#[allow(dead_code)]
pub fn default_multi_res_config() -> MultiResConfig {
    MultiResConfig {
        levels: 4,
        reduction_ratio: 0.5,
    }
}

fn midpoint(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

/// Decimate by removing every other triangle (simplified).
fn decimate_half(level: &MeshLevel) -> MeshLevel {
    let keep = level.indices.len().div_ceil(3 * 2);
    let new_indices = level.indices[..(keep * 3).min(level.indices.len())].to_vec();
    MeshLevel {
        positions: level.positions.clone(),
        indices: new_indices,
        level: level.level + 1,
    }
}

/// Build a multi-resolution mesh hierarchy.
#[allow(dead_code)]
pub fn build_multi_res(
    positions: &[[f32; 3]],
    indices: &[u32],
    cfg: &MultiResConfig,
) -> MultiResMesh {
    let base = MeshLevel {
        positions: positions.to_vec(),
        indices: indices.to_vec(),
        level: 0,
    };
    let mut levels = vec![base];
    for _ in 1..cfg.levels {
        let last = &levels[levels.len() - 1];
        if last.indices.len() < 6 {
            break;
        }
        levels.push(decimate_half(last));
    }
    MultiResMesh { levels }
}

#[allow(dead_code)]
pub fn level_count(mr: &MultiResMesh) -> usize {
    mr.levels.len()
}

#[allow(dead_code)]
pub fn get_level(mr: &MultiResMesh, idx: usize) -> Option<&MeshLevel> {
    mr.levels.get(idx)
}

#[allow(dead_code)]
pub fn coarsest_level(mr: &MultiResMesh) -> Option<&MeshLevel> {
    mr.levels.last()
}

#[allow(dead_code)]
pub fn finest_level(mr: &MultiResMesh) -> Option<&MeshLevel> {
    mr.levels.first()
}

#[allow(dead_code)]
pub fn level_face_counts(mr: &MultiResMesh) -> Vec<usize> {
    mr.levels.iter().map(|l| l.indices.len() / 3).collect()
}

#[allow(dead_code)]
pub fn multi_res_to_json(mr: &MultiResMesh) -> String {
    let counts: Vec<String> = mr
        .levels
        .iter()
        .map(|l| format!("{}", l.indices.len() / 3))
        .collect();
    format!(
        "{{\"level_count\":{},\"face_counts\":[{}]}}",
        mr.levels.len(),
        counts.join(",")
    )
}

#[allow(dead_code)]
pub fn midpoint_upsample(level: &MeshLevel) -> MeshLevel {
    // Insert midpoints for all triangle edges (1->4 subdivision)
    let n = level.positions.len();
    let mut new_pos = level.positions.clone();
    let mut new_idx: Vec<u32> = Vec::new();
    let mut edge_map: std::collections::HashMap<(u32, u32), u32> = std::collections::HashMap::new();

    for chunk in level.indices.chunks(3) {
        if chunk.len() < 3 {
            continue;
        }
        let (a, b, c) = (chunk[0], chunk[1], chunk[2]);
        let get_mid = |p: u32,
                       q: u32,
                       positions: &mut Vec<[f32; 3]>,
                       map: &mut std::collections::HashMap<(u32, u32), u32>|
         -> u32 {
            let key = if p < q { (p, q) } else { (q, p) };
            if let Some(&idx) = map.get(&key) {
                return idx;
            }
            let mp = midpoint(positions[p as usize], positions[q as usize]);
            let idx = positions.len() as u32;
            positions.push(mp);
            map.insert(key, idx);
            idx
        };
        let ab = get_mid(a, b, &mut new_pos, &mut edge_map);
        let bc = get_mid(b, c, &mut new_pos, &mut edge_map);
        let ca = get_mid(c, a, &mut new_pos, &mut edge_map);
        new_idx.extend_from_slice(&[a, ab, ca]);
        new_idx.extend_from_slice(&[ab, b, bc]);
        new_idx.extend_from_slice(&[ca, bc, c]);
        new_idx.extend_from_slice(&[ab, bc, ca]);
    }
    let _ = n;
    MeshLevel {
        positions: new_pos,
        indices: new_idx,
        level: level.level.saturating_sub(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        (pos, idx)
    }

    fn quad_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    #[test]
    fn test_build_multi_res_levels() {
        let (pos, idx) = quad_mesh();
        let cfg = MultiResConfig {
            levels: 3,
            reduction_ratio: 0.5,
        };
        let mr = build_multi_res(&pos, &idx, &cfg);
        assert!(level_count(&mr) >= 1);
    }

    #[test]
    fn test_finest_level_face_count() {
        let (pos, idx) = quad_mesh();
        let cfg = default_multi_res_config();
        let mr = build_multi_res(&pos, &idx, &cfg);
        let finest = finest_level(&mr).unwrap();
        assert_eq!(finest.indices.len() / 3, 2);
    }

    #[test]
    fn test_coarsest_level_fewer_faces() {
        let (pos, idx) = quad_mesh();
        let cfg = MultiResConfig {
            levels: 3,
            reduction_ratio: 0.5,
        };
        let mr = build_multi_res(&pos, &idx, &cfg);
        let finest_fc = finest_level(&mr).unwrap().indices.len() / 3;
        let coarsest_fc = coarsest_level(&mr).unwrap().indices.len() / 3;
        assert!(coarsest_fc <= finest_fc);
    }

    #[test]
    fn test_level_face_counts() {
        let (pos, idx) = quad_mesh();
        let cfg = MultiResConfig {
            levels: 3,
            reduction_ratio: 0.5,
        };
        let mr = build_multi_res(&pos, &idx, &cfg);
        let counts = level_face_counts(&mr);
        assert_eq!(counts.len(), mr.levels.len());
    }

    #[test]
    fn test_get_level_some() {
        let (pos, idx) = quad_mesh();
        let cfg = default_multi_res_config();
        let mr = build_multi_res(&pos, &idx, &cfg);
        assert!(get_level(&mr, 0).is_some());
    }

    #[test]
    fn test_get_level_out_of_bounds() {
        let (pos, idx) = quad_mesh();
        let cfg = default_multi_res_config();
        let mr = build_multi_res(&pos, &idx, &cfg);
        assert!(get_level(&mr, 999).is_none());
    }

    #[test]
    fn test_to_json() {
        let (pos, idx) = quad_mesh();
        let cfg = default_multi_res_config();
        let mr = build_multi_res(&pos, &idx, &cfg);
        let j = multi_res_to_json(&mr);
        assert!(j.contains("level_count"));
    }

    #[test]
    fn test_midpoint_upsample_increases_faces() {
        let (pos, idx) = simple_mesh();
        let level = MeshLevel {
            positions: pos,
            indices: idx,
            level: 1,
        };
        let up = midpoint_upsample(&level);
        assert_eq!(up.indices.len() / 3, 4);
    }

    #[test]
    fn test_empty_mesh_one_level() {
        let cfg = default_multi_res_config();
        let mr = build_multi_res(&[], &[], &cfg);
        assert_eq!(level_count(&mr), 1);
    }

    #[test]
    fn test_default_config() {
        let cfg = default_multi_res_config();
        assert_eq!(cfg.levels, 4);
        assert!((cfg.reduction_ratio - 0.5).abs() < 1e-6);
    }
}
