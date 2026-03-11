// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! CullingStats — tracks object visibility culling statistics.

#![allow(dead_code)]

/// Statistics for frustum/occlusion culling.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CullingStats {
    culled: u32,
    visible: u32,
}

/// Create empty culling stats.
#[allow(dead_code)]
pub fn new_culling_stats() -> CullingStats {
    CullingStats::default()
}

/// Record that an object was culled.
#[allow(dead_code)]
pub fn record_culled(stats: &mut CullingStats) {
    stats.culled += 1;
}

/// Record that an object was visible.
#[allow(dead_code)]
pub fn record_visible(stats: &mut CullingStats) {
    stats.visible += 1;
}

/// Total number of objects tested.
#[allow(dead_code)]
pub fn total_objects(stats: &CullingStats) -> u32 {
    stats.culled + stats.visible
}

/// Number of culled objects.
#[allow(dead_code)]
pub fn culled_count(stats: &CullingStats) -> u32 {
    stats.culled
}

/// Number of visible objects.
#[allow(dead_code)]
pub fn visible_count(stats: &CullingStats) -> u32 {
    stats.visible
}

/// Ratio of culled to total objects (0.0 if none tested).
#[allow(dead_code)]
pub fn cull_ratio(stats: &CullingStats) -> f32 {
    let total = total_objects(stats);
    if total == 0 {
        return 0.0;
    }
    stats.culled as f32 / total as f32
}

/// Reset all counters.
#[allow(dead_code)]
pub fn culling_stats_reset(stats: &mut CullingStats) {
    stats.culled = 0;
    stats.visible = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_culling_stats() {
        let s = new_culling_stats();
        assert_eq!(total_objects(&s), 0);
    }

    #[test]
    fn test_record_culled() {
        let mut s = new_culling_stats();
        record_culled(&mut s);
        assert_eq!(culled_count(&s), 1);
    }

    #[test]
    fn test_record_visible() {
        let mut s = new_culling_stats();
        record_visible(&mut s);
        assert_eq!(visible_count(&s), 1);
    }

    #[test]
    fn test_total_objects() {
        let mut s = new_culling_stats();
        record_culled(&mut s);
        record_visible(&mut s);
        record_visible(&mut s);
        assert_eq!(total_objects(&s), 3);
    }

    #[test]
    fn test_culled_count() {
        let mut s = new_culling_stats();
        record_culled(&mut s);
        record_culled(&mut s);
        assert_eq!(culled_count(&s), 2);
    }

    #[test]
    fn test_visible_count_zero() {
        let s = new_culling_stats();
        assert_eq!(visible_count(&s), 0);
    }

    #[test]
    fn test_cull_ratio_empty() {
        let s = new_culling_stats();
        assert!((cull_ratio(&s) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_cull_ratio_half() {
        let mut s = new_culling_stats();
        record_culled(&mut s);
        record_visible(&mut s);
        assert!((cull_ratio(&s) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_culling_stats_reset() {
        let mut s = new_culling_stats();
        record_culled(&mut s);
        record_visible(&mut s);
        culling_stats_reset(&mut s);
        assert_eq!(total_objects(&s), 0);
    }

    #[test]
    fn test_cull_ratio_all_culled() {
        let mut s = new_culling_stats();
        record_culled(&mut s);
        record_culled(&mut s);
        assert!((cull_ratio(&s) - 1.0).abs() < 1e-6);
    }
}
