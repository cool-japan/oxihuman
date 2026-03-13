// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// LOD screen-coverage visualization.
#[derive(Debug, Clone)]
pub struct LodCoverageView {
    pub enabled: bool,
    /// Number of LOD levels to display.
    pub lod_count: u32,
    /// Screen-space coverage thresholds for each LOD switch (0.0 … 1.0).
    pub thresholds: Vec<f32>,
}

pub fn new_lod_coverage_view(lod_count: u32) -> LodCoverageView {
    let thresholds = (0..lod_count)
        .map(|i| 1.0 - i as f32 / lod_count.max(1) as f32)
        .collect();
    LodCoverageView {
        enabled: false,
        lod_count,
        thresholds,
    }
}

pub fn lcv_enable(v: &mut LodCoverageView) {
    v.enabled = true;
}

pub fn lcv_set_threshold(v: &mut LodCoverageView, level: usize, t: f32) {
    if let Some(slot) = v.thresholds.get_mut(level) {
        *slot = t.clamp(0.0, 1.0);
    }
}

/// Returns the active LOD level for the given screen coverage (0.0 … 1.0).
pub fn lcv_active_lod(v: &LodCoverageView, coverage: f32) -> u32 {
    for (i, &thresh) in v.thresholds.iter().enumerate() {
        if coverage >= thresh {
            return i as u32;
        }
    }
    v.lod_count.saturating_sub(1)
}

/// Returns the colour for a LOD level.
pub fn lcv_lod_color(level: u32, lod_count: u32) -> [f32; 3] {
    let t = if lod_count == 0 {
        0.0
    } else {
        level as f32 / lod_count as f32
    };
    [t, 1.0 - t, 0.0]
}

pub fn lcv_to_json(v: &LodCoverageView) -> String {
    format!(r#"{{"enabled":{},"lod_count":{}}}"#, v.enabled, v.lod_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* lod_count=4, enabled=false */
        let v = new_lod_coverage_view(4);
        assert_eq!(v.lod_count, 4);
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable() {
        /* enable */
        let mut v = new_lod_coverage_view(3);
        lcv_enable(&mut v);
        assert!(v.enabled);
    }

    #[test]
    fn test_threshold_count() {
        /* thresholds count matches lod_count */
        let v = new_lod_coverage_view(4);
        assert_eq!(v.thresholds.len(), 4);
    }

    #[test]
    fn test_set_threshold() {
        /* set threshold for level 0 */
        let mut v = new_lod_coverage_view(3);
        lcv_set_threshold(&mut v, 0, 0.9);
        assert!((v.thresholds[0] - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_active_lod_high_coverage() {
        /* high coverage → lod 0 */
        let v = new_lod_coverage_view(4);
        assert_eq!(lcv_active_lod(&v, 1.0), 0);
    }

    #[test]
    fn test_lod_color_range() {
        /* colours in [0,1] */
        let c = lcv_lod_color(2, 4);
        for ch in c {
            assert!((0.0..=1.0).contains(&ch));
        }
    }

    #[test]
    fn test_lod_color_lod0_green() {
        /* LOD 0 -> 0% from max -> green */
        let c = lcv_lod_color(0, 4);
        assert_eq!(c[0], 0.0);
    }

    #[test]
    fn test_threshold_clamp() {
        /* threshold clamped */
        let mut v = new_lod_coverage_view(2);
        lcv_set_threshold(&mut v, 0, 2.0);
        assert_eq!(v.thresholds[0], 1.0);
    }

    #[test]
    fn test_to_json() {
        /* JSON has lod_count */
        assert!(lcv_to_json(&new_lod_coverage_view(3)).contains("lod_count"));
    }
}
