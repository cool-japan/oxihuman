// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// LOD level definition for morph targets.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct LodLevel {
    threshold: f32,
    morph_count: usize,
}

/// Selects the appropriate LOD level for morph evaluation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphLodSelector {
    levels: Vec<LodLevel>,
    current: usize,
}

/// Create a new LOD selector with default levels.
#[allow(dead_code)]
pub fn new_morph_lod_selector(level_count: usize) -> MorphLodSelector {
    let mut levels = Vec::new();
    for i in 0..level_count.max(1) {
        levels.push(LodLevel {
            threshold: (i as f32 + 1.0) * 10.0,
            morph_count: (level_count - i) * 10,
        });
    }
    MorphLodSelector { levels, current: 0 }
}

/// Select the LOD level based on distance. Returns the level index.
#[allow(dead_code)]
pub fn select_lod(selector: &mut MorphLodSelector, distance: f32) -> usize {
    let mut chosen = 0;
    for (i, level) in selector.levels.iter().enumerate() {
        if distance >= level.threshold {
            chosen = i;
        }
    }
    selector.current = chosen;
    chosen
}

/// Return the number of LOD levels.
#[allow(dead_code)]
pub fn lod_level_count(selector: &MorphLodSelector) -> usize {
    selector.levels.len()
}

/// Return the threshold for a given level.
#[allow(dead_code)]
pub fn lod_threshold(selector: &MorphLodSelector, level: usize) -> f32 {
    selector.levels.get(level).map_or(0.0, |l| l.threshold)
}

/// Return the morph count at a given level.
#[allow(dead_code)]
pub fn lod_morph_count_at(selector: &MorphLodSelector, level: usize) -> usize {
    selector.levels.get(level).map_or(0, |l| l.morph_count)
}

/// Set the threshold for a given level.
#[allow(dead_code)]
pub fn set_lod_threshold(selector: &mut MorphLodSelector, level: usize, threshold: f32) {
    if let Some(l) = selector.levels.get_mut(level) {
        l.threshold = threshold.max(0.0);
    }
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn lod_to_json(selector: &MorphLodSelector) -> String {
    let levels: Vec<String> = selector
        .levels
        .iter()
        .map(|l| {
            format!(
                "{{\"threshold\":{:.4},\"morph_count\":{}}}",
                l.threshold, l.morph_count
            )
        })
        .collect();
    format!(
        "{{\"current\":{},\"levels\":[{}]}}",
        selector.current,
        levels.join(",")
    )
}

/// Return the currently selected LOD level.
#[allow(dead_code)]
pub fn lod_current_level(selector: &MorphLodSelector) -> usize {
    selector.current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_selector() {
        let s = new_morph_lod_selector(3);
        assert_eq!(lod_level_count(&s), 3);
    }

    #[test]
    fn select_close() {
        let mut s = new_morph_lod_selector(3);
        let level = select_lod(&mut s, 0.0);
        assert_eq!(level, 0);
    }

    #[test]
    fn select_far() {
        let mut s = new_morph_lod_selector(3);
        let level = select_lod(&mut s, 100.0);
        assert!(level > 0);
    }

    #[test]
    fn current_level() {
        let mut s = new_morph_lod_selector(3);
        select_lod(&mut s, 50.0);
        assert!(lod_current_level(&s) > 0);
    }

    #[test]
    fn threshold_accessor() {
        let s = new_morph_lod_selector(3);
        assert!(lod_threshold(&s, 0) > 0.0);
    }

    #[test]
    fn set_threshold() {
        let mut s = new_morph_lod_selector(3);
        set_lod_threshold(&mut s, 0, 5.0);
        assert!((lod_threshold(&s, 0) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn morph_count_at() {
        let s = new_morph_lod_selector(3);
        assert!(lod_morph_count_at(&s, 0) > 0);
    }

    #[test]
    fn to_json() {
        let s = new_morph_lod_selector(2);
        let j = lod_to_json(&s);
        assert!(j.contains("\"current\""));
    }

    #[test]
    fn single_level() {
        let s = new_morph_lod_selector(1);
        assert_eq!(lod_level_count(&s), 1);
    }

    #[test]
    fn zero_level_gets_one() {
        let s = new_morph_lod_selector(0);
        assert_eq!(lod_level_count(&s), 1);
    }
}
