// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Array modifier: duplicate mesh with cumulative offset.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArrayModConfig {
    pub count: usize,
    pub offset: [f32; 3],
    pub relative_offset: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArrayModResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub instance_count: usize,
}

#[allow(dead_code)]
pub fn default_array_mod_config() -> ArrayModConfig {
    ArrayModConfig {
        count: 3,
        offset: [1.0, 0.0, 0.0],
        relative_offset: false,
    }
}

#[allow(dead_code)]
pub fn array_mod_offset_at(config: &ArrayModConfig, i: usize) -> [f32; 3] {
    [
        config.offset[0] * i as f32,
        config.offset[1] * i as f32,
        config.offset[2] * i as f32,
    ]
}

#[allow(dead_code)]
pub fn array_mod_instance_count(config: &ArrayModConfig) -> usize {
    config.count
}

#[allow(dead_code)]
pub fn array_mod_vertex_count(source_vertex_count: usize, config: &ArrayModConfig) -> usize {
    source_vertex_count * config.count
}

#[allow(dead_code)]
pub fn array_mod_validate(config: &ArrayModConfig) -> bool {
    config.count > 0
}

#[allow(dead_code)]
pub fn apply_array_mod(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &ArrayModConfig,
) -> ArrayModResult {
    let nv = positions.len();
    let mut out_positions = Vec::with_capacity(nv * config.count);
    let mut out_indices = Vec::with_capacity(indices.len() * config.count);
    for i in 0..config.count {
        let off = array_mod_offset_at(config, i);
        for p in positions {
            out_positions.push([p[0] + off[0], p[1] + off[1], p[2] + off[2]]);
        }
        let base = (i * nv) as u32;
        for &idx in indices {
            out_indices.push(idx + base);
        }
    }
    ArrayModResult {
        instance_count: config.count,
        positions: out_positions,
        indices: out_indices,
    }
}

#[allow(dead_code)]
pub fn array_mod_to_json(result: &ArrayModResult) -> String {
    format!(
        "{{\"instance_count\":{},\"vertex_count\":{},\"index_count\":{}}}",
        result.instance_count,
        result.positions.len(),
        result.indices.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> (Vec<[f32; 3]>, Vec<u32>) {
        (
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
        )
    }

    #[test]
    fn test_default_config() {
        let cfg = default_array_mod_config();
        assert_eq!(cfg.count, 3);
    }

    #[test]
    fn test_validate() {
        let cfg = default_array_mod_config();
        assert!(array_mod_validate(&cfg));
        let bad = ArrayModConfig { count: 0, ..cfg };
        assert!(!array_mod_validate(&bad));
    }

    #[test]
    fn test_offset_at_zero() {
        let cfg = default_array_mod_config();
        let off = array_mod_offset_at(&cfg, 0);
        assert!((off[0]).abs() < 1e-6);
    }

    #[test]
    fn test_offset_at_two() {
        let cfg = default_array_mod_config();
        let off = array_mod_offset_at(&cfg, 2);
        assert!((off[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_vertex_count() {
        let (pos, idx) = triangle();
        let cfg = ArrayModConfig { count: 4, offset: [2.0, 0.0, 0.0], relative_offset: false };
        let result = apply_array_mod(&pos, &idx, &cfg);
        assert_eq!(result.positions.len(), 12);
    }

    #[test]
    fn test_apply_index_count() {
        let (pos, idx) = triangle();
        let cfg = ArrayModConfig { count: 3, offset: [2.0, 0.0, 0.0], relative_offset: false };
        let result = apply_array_mod(&pos, &idx, &cfg);
        assert_eq!(result.indices.len(), 9);
    }

    #[test]
    fn test_instance_count() {
        let cfg = ArrayModConfig { count: 5, offset: [1.0, 0.0, 0.0], relative_offset: false };
        assert_eq!(array_mod_instance_count(&cfg), 5);
    }

    #[test]
    fn test_to_json() {
        let (pos, idx) = triangle();
        let cfg = default_array_mod_config();
        let result = apply_array_mod(&pos, &idx, &cfg);
        let j = array_mod_to_json(&result);
        assert!(j.contains("instance_count"));
    }
}
