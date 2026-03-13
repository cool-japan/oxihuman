// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! LOD chain manager — stores and manages multiple LOD levels for a mesh.

/// Configuration for a single LOD level.
#[derive(Debug, Clone)]
pub struct LodLevel {
    pub level: u32,
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub screen_size_threshold: f32,
}

/// Manages a chain of LOD levels for a mesh.
#[derive(Debug, Default, Clone)]
pub struct LodChain {
    levels: Vec<LodLevel>,
    active_level: usize,
}

impl LodChain {
    /// Creates a new empty LOD chain.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a LOD level to the chain.
    pub fn add_level(&mut self, level: LodLevel) {
        self.levels.push(level);
        self.levels.sort_by(|a, b| a.level.cmp(&b.level));
    }

    /// Returns the number of LOD levels in the chain.
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }

    /// Selects the best LOD level for a given screen-space size (0..=1).
    pub fn select_level(&mut self, screen_size: f32) -> usize {
        let screen_size = screen_size.clamp(0.0, 1.0);
        let mut chosen = self.levels.len().saturating_sub(1);
        for (i, lvl) in self.levels.iter().enumerate() {
            if screen_size >= lvl.screen_size_threshold {
                chosen = i;
                break;
            }
        }
        self.active_level = chosen;
        chosen
    }

    /// Returns the currently active LOD level index.
    pub fn active_level(&self) -> usize {
        self.active_level
    }

    /// Removes all LOD levels from the chain.
    pub fn clear(&mut self) {
        self.levels.clear();
        self.active_level = 0;
    }
}

/// Builds a default LOD chain with `n` levels.
pub fn build_default_lod_chain(base_vertex_count: usize, levels: u32) -> LodChain {
    let mut chain = LodChain::new();
    for i in 0..levels {
        let ratio = 1.0 / (1u32 << i) as f32;
        chain.add_level(LodLevel {
            level: i,
            vertex_count: ((base_vertex_count as f32) * ratio) as usize,
            triangle_count: ((base_vertex_count as f32) * ratio * 0.5) as usize,
            screen_size_threshold: ratio,
        });
    }
    chain
}

/// Returns the total triangle count across all LOD levels.
pub fn total_triangle_budget(chain: &LodChain) -> usize {
    chain.levels.iter().map(|l| l.triangle_count).sum()
}

/// Validates that LOD levels are ordered correctly.
pub fn validate_lod_chain(chain: &LodChain) -> bool {
    chain.levels.windows(2).all(|w| w[0].level < w[1].level)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_chain_is_empty() {
        /* A fresh chain should have zero levels */
        let chain = LodChain::new();
        assert_eq!(chain.level_count(), 0);
    }

    #[test]
    fn test_add_level() {
        /* Adding a level should increase count */
        let mut chain = LodChain::new();
        chain.add_level(LodLevel {
            level: 0,
            vertex_count: 1000,
            triangle_count: 500,
            screen_size_threshold: 1.0,
        });
        assert_eq!(chain.level_count(), 1);
    }

    #[test]
    fn test_select_level_high_screen_size() {
        /* High screen size should select level 0 */
        let mut chain = build_default_lod_chain(1000, 4);
        let idx = chain.select_level(1.0);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_select_level_zero_screen_size() {
        /* Zero screen size should select last level */
        let mut chain = build_default_lod_chain(1000, 4);
        let idx = chain.select_level(0.0);
        assert_eq!(idx, 3);
    }

    #[test]
    fn test_active_level_updated() {
        /* Active level should match last select_level call */
        let mut chain = build_default_lod_chain(1000, 3);
        chain.select_level(0.6);
        assert_eq!(chain.active_level(), chain.select_level(0.6));
    }

    #[test]
    fn test_clear() {
        /* Clear should remove all levels */
        let mut chain = build_default_lod_chain(1000, 4);
        chain.clear();
        assert_eq!(chain.level_count(), 0);
    }

    #[test]
    fn test_validate_ordered() {
        /* Default chain should be valid */
        let chain = build_default_lod_chain(1000, 4);
        assert!(validate_lod_chain(&chain));
    }

    #[test]
    fn test_total_triangle_budget() {
        /* Budget should be positive for non-empty chain */
        let chain = build_default_lod_chain(1000, 3);
        assert!(total_triangle_budget(&chain) > 0);
    }

    #[test]
    fn test_build_default_levels_count() {
        /* build_default_lod_chain should produce exactly n levels */
        let chain = build_default_lod_chain(2000, 5);
        assert_eq!(chain.level_count(), 5);
    }

    #[test]
    fn test_level_sort_on_add() {
        /* Levels added out of order should be sorted */
        let mut chain = LodChain::new();
        chain.add_level(LodLevel {
            level: 2,
            vertex_count: 100,
            triangle_count: 50,
            screen_size_threshold: 0.25,
        });
        chain.add_level(LodLevel {
            level: 0,
            vertex_count: 1000,
            triangle_count: 500,
            screen_size_threshold: 1.0,
        });
        assert!(validate_lod_chain(&chain));
    }
}
