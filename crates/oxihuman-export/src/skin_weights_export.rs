// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export per-vertex skin weight data for skeletal animation.

/// Bone influence.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct BoneInfluence {
    pub bone_index: u32,
    pub weight: f32,
}

/// Per-vertex skin weights.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkinWeightsExport {
    pub vertex_weights: Vec<Vec<BoneInfluence>>,
    pub max_influences: u32,
}

#[allow(dead_code)]
pub fn new_skin_weights(vertex_count: usize, max_inf: u32) -> SkinWeightsExport {
    SkinWeightsExport { vertex_weights: vec![Vec::new(); vertex_count], max_influences: max_inf }
}

#[allow(dead_code)]
pub fn sw_add_weight(export: &mut SkinWeightsExport, vertex: usize, bone: u32, weight: f32) {
    if vertex < export.vertex_weights.len() {
        export.vertex_weights[vertex].push(BoneInfluence { bone_index: bone, weight });
    }
}

#[allow(dead_code)]
pub fn sw_normalize(export: &mut SkinWeightsExport) {
    for weights in export.vertex_weights.iter_mut() {
        let sum: f32 = weights.iter().map(|w| w.weight).sum();
        if sum > 1e-12 {
            for w in weights.iter_mut() { w.weight /= sum; }
        }
    }
}

#[allow(dead_code)]
pub fn sw_prune(export: &mut SkinWeightsExport, threshold: f32) {
    for weights in export.vertex_weights.iter_mut() {
        weights.retain(|w| w.weight > threshold);
    }
}

#[allow(dead_code)]
pub fn sw_vertex_count(export: &SkinWeightsExport) -> usize { export.vertex_weights.len() }

#[allow(dead_code)]
pub fn sw_max_influences_used(export: &SkinWeightsExport) -> usize {
    export.vertex_weights.iter().map(|w| w.len()).max().unwrap_or(0)
}

#[allow(dead_code)]
pub fn sw_bone_count(export: &SkinWeightsExport) -> usize {
    let mut bones = std::collections::HashSet::new();
    for weights in &export.vertex_weights {
        for w in weights { bones.insert(w.bone_index); }
    }
    bones.len()
}

#[allow(dead_code)]
pub fn sw_to_json(export: &SkinWeightsExport) -> String {
    format!(r#"{{"vertices":{},"max_influences":{},"bones_used":{}}}"#,
        sw_vertex_count(export), export.max_influences, sw_bone_count(export))
}

#[allow(dead_code)]
pub fn sw_to_bytes(export: &SkinWeightsExport) -> Vec<u8> {
    let mut bytes = Vec::new();
    for weights in &export.vertex_weights {
        bytes.extend_from_slice(&(weights.len() as u32).to_le_bytes());
        for w in weights {
            bytes.extend_from_slice(&w.bone_index.to_le_bytes());
            bytes.extend_from_slice(&w.weight.to_le_bytes());
        }
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_weights() {
        let sw = new_skin_weights(10, 4);
        assert_eq!(sw_vertex_count(&sw), 10);
    }

    #[test]
    fn test_add_weight() {
        let mut sw = new_skin_weights(3, 4);
        sw_add_weight(&mut sw, 0, 1, 0.5);
        sw_add_weight(&mut sw, 0, 2, 0.3);
        assert_eq!(sw.vertex_weights[0].len(), 2);
    }

    #[test]
    fn test_normalize() {
        let mut sw = new_skin_weights(1, 4);
        sw_add_weight(&mut sw, 0, 0, 2.0);
        sw_add_weight(&mut sw, 0, 1, 2.0);
        sw_normalize(&mut sw);
        let sum: f32 = sw.vertex_weights[0].iter().map(|w| w.weight).sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_prune() {
        let mut sw = new_skin_weights(1, 4);
        sw_add_weight(&mut sw, 0, 0, 0.9);
        sw_add_weight(&mut sw, 0, 1, 0.001);
        sw_prune(&mut sw, 0.01);
        assert_eq!(sw.vertex_weights[0].len(), 1);
    }

    #[test]
    fn test_max_influences() {
        let mut sw = new_skin_weights(2, 4);
        sw_add_weight(&mut sw, 0, 0, 0.5);
        sw_add_weight(&mut sw, 0, 1, 0.5);
        sw_add_weight(&mut sw, 1, 0, 1.0);
        assert_eq!(sw_max_influences_used(&sw), 2);
    }

    #[test]
    fn test_bone_count() {
        let mut sw = new_skin_weights(2, 4);
        sw_add_weight(&mut sw, 0, 0, 0.5);
        sw_add_weight(&mut sw, 0, 1, 0.5);
        sw_add_weight(&mut sw, 1, 1, 1.0);
        assert_eq!(sw_bone_count(&sw), 2);
    }

    #[test]
    fn test_to_json() {
        let sw = new_skin_weights(5, 4);
        let json = sw_to_json(&sw);
        assert!(json.contains("vertices"));
    }

    #[test]
    fn test_to_bytes() {
        let mut sw = new_skin_weights(1, 4);
        sw_add_weight(&mut sw, 0, 0, 1.0);
        let bytes = sw_to_bytes(&sw);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_empty_weights() {
        let sw = new_skin_weights(0, 4);
        assert_eq!(sw_vertex_count(&sw), 0);
    }

    #[test]
    fn test_max_influences_empty() {
        let sw = new_skin_weights(3, 4);
        assert_eq!(sw_max_influences_used(&sw), 0);
    }

}
