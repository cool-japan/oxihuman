#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// A single LOD level for morph targets.
#[derive(Debug, Clone)]
pub struct MorphLodLevel {
    pub lod: u8,
    pub morph_count: usize,
    pub resolution: f32,
}

/// LOD-aware morph target switcher.
#[derive(Debug, Clone)]
pub struct MorphLodSwitcher {
    pub levels: Vec<MorphLodLevel>,
    pub current_lod: u8,
}

#[allow(dead_code)]
pub fn new_morph_lod_switcher() -> MorphLodSwitcher {
    MorphLodSwitcher {
        levels: Vec::new(),
        current_lod: 0,
    }
}

#[allow(dead_code)]
pub fn add_lod_level(ms: &mut MorphLodSwitcher, lod: u8, count: usize, res: f32) {
    ms.levels.push(MorphLodLevel {
        lod,
        morph_count: count,
        resolution: res,
    });
}

#[allow(dead_code)]
pub fn select_lod(ms: &mut MorphLodSwitcher, screen_size: f32) {
    // Pick the highest-resolution level whose index fits within screen_size thresholds.
    // Simple heuristic: if screen_size > 0.5 use lod 0, else lod 1, etc.
    let n = ms.levels.len() as f32;
    if n < 1.0 {
        return;
    }
    let threshold = (1.0 - screen_size.clamp(0.0, 1.0)) * (n - 1.0);
    let idx = threshold.round() as usize;
    let idx = idx.min(ms.levels.len().saturating_sub(1));
    ms.current_lod = ms.levels[idx].lod;
}

#[allow(dead_code)]
pub fn current_morph_count(ms: &MorphLodSwitcher) -> usize {
    ms.levels
        .iter()
        .find(|l| l.lod == ms.current_lod)
        .map(|l| l.morph_count)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_morph_lod_switcher_empty() {
        let ms = new_morph_lod_switcher();
        assert!(ms.levels.is_empty());
        assert_eq!(ms.current_lod, 0);
    }

    #[test]
    fn test_add_lod_level() {
        let mut ms = new_morph_lod_switcher();
        add_lod_level(&mut ms, 0, 64, 1.0);
        assert_eq!(ms.levels.len(), 1);
        assert_eq!(ms.levels[0].morph_count, 64);
    }

    #[test]
    fn test_add_multiple_levels() {
        let mut ms = new_morph_lod_switcher();
        add_lod_level(&mut ms, 0, 64, 1.0);
        add_lod_level(&mut ms, 1, 32, 0.5);
        add_lod_level(&mut ms, 2, 16, 0.25);
        assert_eq!(ms.levels.len(), 3);
    }

    #[test]
    fn test_current_morph_count_no_levels() {
        let ms = new_morph_lod_switcher();
        assert_eq!(current_morph_count(&ms), 0);
    }

    #[test]
    fn test_current_morph_count_found() {
        let mut ms = new_morph_lod_switcher();
        add_lod_level(&mut ms, 0, 64, 1.0);
        assert_eq!(current_morph_count(&ms), 64);
    }

    #[test]
    fn test_select_lod_full_screen() {
        let mut ms = new_morph_lod_switcher();
        add_lod_level(&mut ms, 0, 64, 1.0);
        add_lod_level(&mut ms, 1, 32, 0.5);
        select_lod(&mut ms, 1.0);
        assert_eq!(ms.current_lod, 0);
    }

    #[test]
    fn test_select_lod_small_screen() {
        let mut ms = new_morph_lod_switcher();
        add_lod_level(&mut ms, 0, 64, 1.0);
        add_lod_level(&mut ms, 1, 32, 0.5);
        select_lod(&mut ms, 0.0);
        assert_eq!(ms.current_lod, 1);
    }

    #[test]
    fn test_select_lod_empty() {
        let mut ms = new_morph_lod_switcher();
        select_lod(&mut ms, 0.5);
        assert_eq!(ms.current_lod, 0);
    }

    #[test]
    fn test_lod_resolution_stored() {
        let mut ms = new_morph_lod_switcher();
        add_lod_level(&mut ms, 0, 64, 1.0);
        assert!((ms.levels[0].resolution - 1.0).abs() < 1e-6);
    }
}
