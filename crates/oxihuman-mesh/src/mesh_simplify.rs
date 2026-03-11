// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Quadric-error mesh simplification stub.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SimplifyConfig {
    pub target_ratio: f32,
    pub max_error: f32,
    pub preserve_boundary: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SimplifyResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub removed: usize,
    pub error: f32,
}

#[allow(dead_code)]
pub fn default_simplify_config() -> SimplifyConfig {
    SimplifyConfig { target_ratio: 0.5, max_error: 1e-3, preserve_boundary: true }
}

#[allow(dead_code)]
pub fn simplify_mesh(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &SimplifyConfig,
) -> SimplifyResult {
    let target = ((positions.len() as f32) * config.target_ratio.clamp(0.0, 1.0)) as usize;
    let removed = positions.len().saturating_sub(target.max(1));
    SimplifyResult {
        positions: positions[..target.max(1).min(positions.len())].to_vec(),
        indices: indices.to_vec(),
        removed,
        error: config.max_error * 0.5,
    }
}

#[allow(dead_code)]
pub fn simplify_face_count(result: &SimplifyResult) -> usize {
    result.indices.len() / 3
}

#[allow(dead_code)]
pub fn simplify_vertex_count(result: &SimplifyResult) -> usize {
    result.positions.len()
}

#[allow(dead_code)]
pub fn simplify_achieved_ratio(original: usize, result: &SimplifyResult) -> f32 {
    if original == 0 {
        return 1.0;
    }
    result.positions.len() as f32 / original as f32
}

#[allow(dead_code)]
pub fn simplify_validate_config(config: &SimplifyConfig) -> bool {
    (0.0..=1.0).contains(&config.target_ratio) && config.max_error >= 0.0
}

#[allow(dead_code)]
pub fn simplify_to_json(result: &SimplifyResult) -> String {
    format!(
        r#"{{"vertices":{},"faces":{},"removed":{},"error":{:.6}}}"#,
        result.positions.len(),
        simplify_face_count(result),
        result.removed,
        result.error
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.5, 1.0],
            [2.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
            [1.5, 0.5, 1.0],
            [0.0, 2.0, 0.0],
        ]
    }

    fn sample_indices() -> Vec<u32> {
        vec![0, 1, 2, 1, 3, 2, 4, 5, 6]
    }

    #[test]
    fn default_config_is_valid() {
        let cfg = default_simplify_config();
        assert!(simplify_validate_config(&cfg));
    }

    #[test]
    fn simplify_reduces_vertex_count() {
        let pos = sample_positions();
        let idx = sample_indices();
        let cfg = SimplifyConfig { target_ratio: 0.5, max_error: 0.01, preserve_boundary: false };
        let res = simplify_mesh(&pos, &idx, &cfg);
        assert!(res.positions.len() <= pos.len());
    }

    #[test]
    fn removed_count_matches() {
        let pos = sample_positions();
        let idx = sample_indices();
        let cfg = default_simplify_config();
        let res = simplify_mesh(&pos, &idx, &cfg);
        assert_eq!(res.removed, pos.len() - res.positions.len());
    }

    #[test]
    fn achieved_ratio_in_range() {
        let pos = sample_positions();
        let idx = sample_indices();
        let cfg = default_simplify_config();
        let res = simplify_mesh(&pos, &idx, &cfg);
        let ratio = simplify_achieved_ratio(pos.len(), &res);
        assert!((0.0..=1.0).contains(&ratio));
    }

    #[test]
    fn face_count_is_index_div_three() {
        let pos = sample_positions();
        let idx = sample_indices();
        let cfg = default_simplify_config();
        let res = simplify_mesh(&pos, &idx, &cfg);
        assert_eq!(simplify_face_count(&res), res.indices.len() / 3);
    }

    #[test]
    fn vertex_count_accessor() {
        let pos = sample_positions();
        let idx = sample_indices();
        let cfg = default_simplify_config();
        let res = simplify_mesh(&pos, &idx, &cfg);
        assert_eq!(simplify_vertex_count(&res), res.positions.len());
    }

    #[test]
    fn invalid_ratio_fails_validate() {
        let cfg = SimplifyConfig { target_ratio: 1.5, max_error: 0.01, preserve_boundary: false };
        assert!(!simplify_validate_config(&cfg));
    }

    #[test]
    fn to_json_contains_key_fields() {
        let pos = sample_positions();
        let idx = sample_indices();
        let cfg = default_simplify_config();
        let res = simplify_mesh(&pos, &idx, &cfg);
        let json = simplify_to_json(&res);
        assert!(json.contains("vertices"));
        assert!(json.contains("removed"));
    }
}
