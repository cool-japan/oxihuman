// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! LOD chain: different morph weights per LOD level.

#[allow(dead_code)]
pub struct LodLevel {
    pub distance: f32,
    pub weight_scale: f32,
}

#[allow(dead_code)]
pub struct MorphLodChain {
    pub levels: Vec<LodLevel>,
    pub morph_weights: Vec<f32>,
}

#[allow(dead_code)]
pub fn new_morph_lod_chain(weights: Vec<f32>) -> MorphLodChain {
    MorphLodChain { levels: Vec::new(), morph_weights: weights }
}

#[allow(dead_code)]
pub fn mlc_add_level(chain: &mut MorphLodChain, distance: f32, weight_scale: f32) {
    chain.levels.push(LodLevel { distance, weight_scale });
    chain.levels.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal));
}

#[allow(dead_code)]
pub fn mlc_weights_at_distance(chain: &MorphLodChain, dist: f32) -> Vec<f32> {
    let scale = if chain.levels.is_empty() {
        1.0
    } else {
        let idx = chain.levels.partition_point(|l| l.distance <= dist);
        if idx == 0 {
            chain.levels[0].weight_scale
        } else if idx >= chain.levels.len() {
            chain.levels[chain.levels.len() - 1].weight_scale
        } else {
            chain.levels[idx - 1].weight_scale
        }
    };
    chain.morph_weights.iter().map(|w| w * scale).collect()
}

#[allow(dead_code)]
pub fn mlc_level_count(chain: &MorphLodChain) -> usize {
    chain.levels.len()
}

#[allow(dead_code)]
pub fn mlc_morph_count(chain: &MorphLodChain) -> usize {
    chain.morph_weights.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_level() {
        let mut chain = new_morph_lod_chain(vec![1.0]);
        mlc_add_level(&mut chain, 10.0, 0.5);
        assert_eq!(mlc_level_count(&chain), 1);
    }

    #[test]
    fn test_level_count() {
        let mut chain = new_morph_lod_chain(vec![1.0]);
        mlc_add_level(&mut chain, 10.0, 1.0);
        mlc_add_level(&mut chain, 50.0, 0.5);
        assert_eq!(mlc_level_count(&chain), 2);
    }

    #[test]
    fn test_morph_count() {
        let chain = new_morph_lod_chain(vec![0.5, 0.5, 0.5]);
        assert_eq!(mlc_morph_count(&chain), 3);
    }

    #[test]
    fn test_weights_at_distance_near() {
        let mut chain = new_morph_lod_chain(vec![1.0]);
        mlc_add_level(&mut chain, 5.0, 1.0);
        mlc_add_level(&mut chain, 50.0, 0.5);
        let w = mlc_weights_at_distance(&chain, 1.0);
        assert!((w[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_weights_at_distance_far() {
        let mut chain = new_morph_lod_chain(vec![1.0]);
        mlc_add_level(&mut chain, 5.0, 1.0);
        mlc_add_level(&mut chain, 50.0, 0.5);
        let w = mlc_weights_at_distance(&chain, 100.0);
        assert!((w[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_no_levels_full_weight() {
        let chain = new_morph_lod_chain(vec![0.8]);
        let w = mlc_weights_at_distance(&chain, 100.0);
        assert!((w[0] - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_multiple_weights() {
        let chain = new_morph_lod_chain(vec![1.0, 0.5, 0.0]);
        assert_eq!(mlc_morph_count(&chain), 3);
    }

    #[test]
    fn test_weights_scaled() {
        let mut chain = new_morph_lod_chain(vec![2.0]);
        mlc_add_level(&mut chain, 10.0, 0.5);
        let w = mlc_weights_at_distance(&chain, 5.0);
        assert!((w[0] - 1.0).abs() < 1e-5);
    }
}
