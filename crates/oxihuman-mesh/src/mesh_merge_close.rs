// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Merge vertices closer than a threshold (weld).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MergeCloseConfig {
    pub threshold: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MergeCloseResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub merged_count: usize,
}

#[allow(dead_code)]
pub fn default_merge_close_config() -> MergeCloseConfig {
    MergeCloseConfig { threshold: 1e-4 }
}

#[allow(dead_code)]
pub fn merge_close_vertices(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &MergeCloseConfig,
) -> MergeCloseResult {
    let thresh_sq = config.threshold * config.threshold;
    let mut new_positions: Vec<[f32; 3]> = Vec::new();
    let mut remap = vec![0u32; positions.len()];
    for (i, &p) in positions.iter().enumerate() {
        let mut found = None;
        for (j, &q) in new_positions.iter().enumerate() {
            let dx = p[0] - q[0];
            let dy = p[1] - q[1];
            let dz = p[2] - q[2];
            if dx * dx + dy * dy + dz * dz <= thresh_sq {
                found = Some(j as u32);
                break;
            }
        }
        if let Some(idx) = found {
            remap[i] = idx;
        } else {
            remap[i] = new_positions.len() as u32;
            new_positions.push(p);
        }
    }
    let merged_count = positions.len() - new_positions.len();
    let new_indices: Vec<u32> = indices.iter().map(|&i| remap[i as usize]).collect();
    MergeCloseResult { positions: new_positions, indices: new_indices, merged_count }
}

#[allow(dead_code)]
pub fn merge_close_vertex_count(result: &MergeCloseResult) -> usize {
    result.positions.len()
}

#[allow(dead_code)]
pub fn merge_close_ratio(original: usize, result: &MergeCloseResult) -> f32 {
    if original == 0 { return 1.0; }
    result.positions.len() as f32 / original as f32
}

#[allow(dead_code)]
pub fn merge_close_validate(config: &MergeCloseConfig) -> bool {
    config.threshold >= 0.0
}

#[allow(dead_code)]
pub fn merge_close_to_json(result: &MergeCloseResult) -> String {
    format!(
        r#"{{"vertices":{},"indices":{},"merged":{}}}"#,
        result.positions.len(),
        result.indices.len(),
        result.merged_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_identical_vertices() {
        let positions = vec![[0.0f32; 3], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let indices = vec![0u32, 1, 2];
        let cfg = default_merge_close_config();
        let res = merge_close_vertices(&positions, &indices, &cfg);
        assert_eq!(res.merged_count, 1);
        assert_eq!(res.positions.len(), 2);
    }

    #[test]
    fn no_merge_when_far_apart() {
        let positions = vec![[0.0f32; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let indices = vec![0u32, 1, 2];
        let cfg = default_merge_close_config();
        let res = merge_close_vertices(&positions, &indices, &cfg);
        assert_eq!(res.merged_count, 0);
    }

    #[test]
    fn ratio_is_correct() {
        let positions = vec![[0.0f32; 3], [0.0, 0.0, 0.0]];
        let indices = vec![0u32, 1];
        let cfg = default_merge_close_config();
        let res = merge_close_vertices(&positions, &indices, &cfg);
        let ratio = merge_close_ratio(2, &res);
        assert!((ratio - 0.5).abs() < 1e-6);
    }

    #[test]
    fn validate_negative_threshold_fails() {
        let cfg = MergeCloseConfig { threshold: -1.0 };
        assert!(!merge_close_validate(&cfg));
    }

    #[test]
    fn validate_zero_threshold_ok() {
        let cfg = MergeCloseConfig { threshold: 0.0 };
        assert!(merge_close_validate(&cfg));
    }

    #[test]
    fn vertex_count_accessor() {
        let positions = vec![[0.0f32; 3], [1.0, 0.0, 0.0]];
        let indices = vec![0u32, 1];
        let cfg = default_merge_close_config();
        let res = merge_close_vertices(&positions, &indices, &cfg);
        assert_eq!(merge_close_vertex_count(&res), res.positions.len());
    }

    #[test]
    fn to_json_has_merged() {
        let positions = vec![[0.0f32; 3]];
        let indices: Vec<u32> = vec![];
        let cfg = default_merge_close_config();
        let res = merge_close_vertices(&positions, &indices, &cfg);
        let json = merge_close_to_json(&res);
        assert!(json.contains("merged"));
        assert!(json.contains("vertices"));
    }

    #[test]
    fn empty_positions_ok() {
        let cfg = default_merge_close_config();
        let res = merge_close_vertices(&[], &[], &cfg);
        assert_eq!(res.merged_count, 0);
    }
}
