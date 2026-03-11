// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Vertex data transfer modifier.

/// Type of data to transfer between meshes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataLayer {
    VertexNormals,
    UvCoords,
    VertexColors,
    Weights,
}

/// Configuration for data transfer.
#[derive(Debug, Clone)]
pub struct DataTransferConfig {
    pub layer: DataLayer,
    pub max_distance: f32,
    pub mix_factor: f32,
}

impl Default for DataTransferConfig {
    fn default() -> Self {
        Self {
            layer: DataLayer::VertexNormals,
            max_distance: 0.01,
            mix_factor: 1.0,
        }
    }
}

impl DataTransferConfig {
    pub fn new(layer: DataLayer, max_distance: f32) -> Self {
        Self {
            layer,
            max_distance,
            mix_factor: 1.0,
        }
    }
}

/// Find the nearest source vertex index for a given target position.
pub fn nearest_vertex(target: [f32; 3], source_positions: &[[f32; 3]]) -> Option<usize> {
    let mut best = None;
    let mut best_dist = f32::MAX;
    for (i, &sp) in source_positions.iter().enumerate() {
        let d =
            (target[0] - sp[0]).powi(2) + (target[1] - sp[1]).powi(2) + (target[2] - sp[2]).powi(2);
        if d < best_dist {
            best_dist = d;
            best = Some(i);
        }
    }
    best
}

/// Transfer normals from source mesh to target positions.
pub fn transfer_normals(
    target_positions: &[[f32; 3]],
    source_positions: &[[f32; 3]],
    source_normals: &[[f32; 3]],
    cfg: &DataTransferConfig,
) -> Vec<[f32; 3]> {
    target_positions
        .iter()
        .map(|&tp| {
            if let Some(idx) = nearest_vertex(tp, source_positions) {
                let dx = tp[0] - source_positions[idx][0];
                let dy = tp[1] - source_positions[idx][1];
                let dz = tp[2] - source_positions[idx][2];
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                if dist <= cfg.max_distance {
                    let sn = source_normals[idx];
                    let f = cfg.mix_factor;
                    let inv_f = 1.0 - f;
                    return [sn[0] * f + inv_f, sn[1] * f, sn[2] * f];
                }
            }
            [0.0, 1.0, 0.0]
        })
        .collect()
}

/// Transfer UV coordinates.
pub fn transfer_uvs(
    target_positions: &[[f32; 3]],
    source_positions: &[[f32; 3]],
    source_uvs: &[[f32; 2]],
    cfg: &DataTransferConfig,
) -> Vec<[f32; 2]> {
    target_positions
        .iter()
        .map(|&tp| {
            if let Some(idx) = nearest_vertex(tp, source_positions) {
                let dx = tp[0] - source_positions[idx][0];
                let dy = tp[1] - source_positions[idx][1];
                let dz = tp[2] - source_positions[idx][2];
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                if dist <= cfg.max_distance {
                    return source_uvs[idx];
                }
            }
            [0.0, 0.0]
        })
        .collect()
}

/// Validate data transfer config.
pub fn validate_data_transfer_config(cfg: &DataTransferConfig) -> bool {
    cfg.max_distance >= 0.0 && (0.0..=1.0).contains(&cfg.mix_factor)
}

// ── New required API ──────────────────────────────────────────────────────────

pub struct VertexData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
}

pub fn new_vertex_data(n: usize) -> VertexData {
    VertexData {
        positions: vec![[0.0; 3]; n],
        normals: vec![[0.0, 1.0, 0.0]; n],
        uvs: vec![[0.0; 2]; n],
    }
}

pub fn vertex_data_count(d: &VertexData) -> usize {
    d.positions.len()
}

pub fn vertex_nearest_index(source: &VertexData, target_pos: [f32; 3]) -> usize {
    let mut best_dist = f32::MAX;
    let mut best_idx = 0usize;
    for (i, &sp) in source.positions.iter().enumerate() {
        let d = (target_pos[0] - sp[0]).powi(2)
            + (target_pos[1] - sp[1]).powi(2)
            + (target_pos[2] - sp[2]).powi(2);
        if d < best_dist {
            best_dist = d;
            best_idx = i;
        }
    }
    best_idx
}

pub fn transfer_nearest_normal(source: &VertexData, target_pos: [f32; 3]) -> [f32; 3] {
    if source.positions.is_empty() {
        return [0.0, 1.0, 0.0];
    }
    let idx = vertex_nearest_index(source, target_pos);
    if idx < source.normals.len() {
        source.normals[idx]
    } else {
        [0.0, 1.0, 0.0]
    }
}

pub fn transfer_nearest_uv(source: &VertexData, target_pos: [f32; 3]) -> [f32; 2] {
    if source.positions.is_empty() {
        return [0.0; 2];
    }
    let idx = vertex_nearest_index(source, target_pos);
    if idx < source.uvs.len() {
        source.uvs[idx]
    } else {
        [0.0; 2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_transfer_config_default() {
        let cfg = DataTransferConfig::default();
        assert_eq!(cfg.layer, DataLayer::VertexNormals);
    }

    #[test]
    fn test_nearest_vertex_single() {
        let target = [0.0_f32, 0.0, 0.0];
        let sources = vec![[1.0_f32, 0.0, 0.0]];
        assert_eq!(nearest_vertex(target, &sources), Some(0));
    }

    #[test]
    fn test_nearest_vertex_closest() {
        let target = [1.0_f32, 0.0, 0.0];
        let sources = vec![[0.0_f32, 0.0, 0.0], [1.1, 0.0, 0.0]];
        assert_eq!(nearest_vertex(target, &sources), Some(1));
    }

    #[test]
    fn test_nearest_vertex_empty() {
        assert_eq!(nearest_vertex([0.0; 3], &[]), None);
    }

    #[test]
    fn test_transfer_normals_count() {
        let tp = vec![[0.0_f32; 3]];
        let sp = vec![[0.0_f32; 3]];
        let sn = vec![[0.0_f32, 1.0, 0.0]];
        let cfg = DataTransferConfig::new(DataLayer::VertexNormals, 1.0);
        let out = transfer_normals(&tp, &sp, &sn, &cfg);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn test_transfer_uvs_count() {
        let tp = vec![[0.0_f32; 3]];
        let sp = vec![[0.0_f32; 3]];
        let su = vec![[0.5_f32, 0.5]];
        let cfg = DataTransferConfig::new(DataLayer::UvCoords, 1.0);
        let out = transfer_uvs(&tp, &sp, &su, &cfg);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn test_validate_config_valid() {
        let cfg = DataTransferConfig::default();
        assert!(validate_data_transfer_config(&cfg));
    }

    #[test]
    fn test_validate_config_invalid_factor() {
        let cfg = DataTransferConfig {
            mix_factor: 2.0,
            ..DataTransferConfig::default()
        };
        assert!(!validate_data_transfer_config(&cfg));
    }

    #[test]
    fn test_data_layer_debug() {
        let s = format!("{:?}", DataLayer::Weights);
        assert!(s.contains("Weights"));
    }

    #[test]
    fn test_transfer_normals_beyond_max_distance() {
        let tp = vec![[100.0_f32, 0.0, 0.0]];
        let sp = vec![[0.0_f32; 3]];
        let sn = vec![[1.0_f32, 0.0, 0.0]];
        let cfg = DataTransferConfig::new(DataLayer::VertexNormals, 0.001);
        let out = transfer_normals(&tp, &sp, &sn, &cfg);
        assert!((out[0][1] - 1.0).abs() < 1e-5); /* fallback normal */
    }
}
