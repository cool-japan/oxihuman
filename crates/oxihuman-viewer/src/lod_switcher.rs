//! LOD (Level of Detail) switcher — selects a mesh LOD level based on
//! camera-to-object distance for the OxiHuman viewer.

// ──────────────────────────────────────────────────────────────────────────────
// Structs
// ──────────────────────────────────────────────────────────────────────────────

/// Configuration for the LOD switcher.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodConfig {
    /// Whether LOD switching is active at creation time.
    pub enabled: bool,
    /// Distance beyond which the coarsest LOD is always selected.
    pub max_distance: f32,
}

/// A single LOD level entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodLevel {
    /// Camera distance at which this level becomes active.
    pub distance_threshold: f32,
    /// Approximate vertex count for this LOD.
    pub vertex_count: u32,
}

/// LOD switcher managing an ordered list of LOD levels.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodSwitcher {
    /// Active configuration.
    pub config: LodConfig,
    /// Ordered list of LOD levels (sorted ascending by distance_threshold).
    pub levels: Vec<LodLevel>,
    /// Whether LOD switching is enabled.
    pub enabled: bool,
}

// ──────────────────────────────────────────────────────────────────────────────
// Functions
// ──────────────────────────────────────────────────────────────────────────────

/// Return a default [`LodConfig`].
#[allow(dead_code)]
pub fn default_lod_config() -> LodConfig {
    LodConfig {
        enabled: true,
        max_distance: 1000.0,
    }
}

/// Create a new, empty [`LodSwitcher`] from a config.
#[allow(dead_code)]
pub fn new_lod_switcher(cfg: &LodConfig) -> LodSwitcher {
    LodSwitcher {
        enabled: cfg.enabled,
        config: cfg.clone(),
        levels: Vec::new(),
    }
}

/// Append a new LOD level. The list is kept sorted in ascending order of
/// `distance_threshold` after insertion.
#[allow(dead_code)]
pub fn lod_add_level(switcher: &mut LodSwitcher, distance_threshold: f32, vertex_count: u32) {
    switcher.levels.push(LodLevel {
        distance_threshold,
        vertex_count,
    });
    switcher
        .levels
        .sort_by(|a, b| a.distance_threshold.partial_cmp(&b.distance_threshold).unwrap_or(std::cmp::Ordering::Equal));
}

/// Select the appropriate LOD level index for a given camera distance.
///
/// Returns the index of the first level whose `distance_threshold` is greater
/// than `camera_distance`.  If `camera_distance` exceeds all thresholds the
/// last (coarsest) level is returned.  Returns `0` when the list is empty.
#[allow(dead_code)]
pub fn lod_select_level(switcher: &LodSwitcher, camera_distance: f32) -> usize {
    if switcher.levels.is_empty() {
        return 0;
    }
    for (i, level) in switcher.levels.iter().enumerate() {
        if camera_distance <= level.distance_threshold {
            return i;
        }
    }
    switcher.levels.len() - 1
}

/// Return the total number of LOD levels.
#[allow(dead_code)]
pub fn lod_level_count(switcher: &LodSwitcher) -> usize {
    switcher.levels.len()
}

/// Return the vertex count for the LOD level at `level` index.
///
/// Returns `0` if the index is out of range.
#[allow(dead_code)]
pub fn lod_vertex_count_at(switcher: &LodSwitcher, level: usize) -> u32 {
    switcher.levels.get(level).map_or(0, |l| l.vertex_count)
}

/// Return the distance threshold for the LOD level at `level` index.
///
/// Returns `f32::MAX` if the index is out of range.
#[allow(dead_code)]
pub fn lod_distance_threshold(switcher: &LodSwitcher, level: usize) -> f32 {
    switcher
        .levels
        .get(level)
        .map_or(f32::MAX, |l| l.distance_threshold)
}

/// Return whether LOD switching is currently enabled.
#[allow(dead_code)]
pub fn lod_is_enabled(switcher: &LodSwitcher) -> bool {
    switcher.enabled
}

/// Enable or disable LOD switching.
#[allow(dead_code)]
pub fn set_lod_enabled(switcher: &mut LodSwitcher, enabled: bool) {
    switcher.enabled = enabled;
}

/// Return the vertex-count reduction ratio for level `level` relative to level
/// `0` (the highest-detail level).
///
/// Returns `1.0` for level `0` or if either vertex count is zero.
#[allow(dead_code)]
pub fn lod_reduction_ratio(switcher: &LodSwitcher, level: usize) -> f32 {
    if switcher.levels.is_empty() {
        return 1.0;
    }
    let base = switcher.levels[0].vertex_count;
    if base == 0 {
        return 1.0;
    }
    let target = lod_vertex_count_at(switcher, level);
    if target == 0 {
        return 1.0;
    }
    target as f32 / base as f32
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_switcher() -> LodSwitcher {
        let cfg = default_lod_config();
        let mut sw = new_lod_switcher(&cfg);
        lod_add_level(&mut sw, 10.0, 10_000);
        lod_add_level(&mut sw, 50.0, 5_000);
        lod_add_level(&mut sw, 200.0, 1_000);
        sw
    }

    #[test]
    fn test_default_config() {
        let cfg = default_lod_config();
        assert!(cfg.enabled);
        assert!((cfg.max_distance - 1000.0).abs() < 1e-5);
    }

    #[test]
    fn test_new_switcher_empty() {
        let cfg = default_lod_config();
        let sw = new_lod_switcher(&cfg);
        assert_eq!(lod_level_count(&sw), 0);
        assert!(lod_is_enabled(&sw));
    }

    #[test]
    fn test_add_level_sorted() {
        let cfg = default_lod_config();
        let mut sw = new_lod_switcher(&cfg);
        lod_add_level(&mut sw, 50.0, 5_000);
        lod_add_level(&mut sw, 10.0, 10_000);
        lod_add_level(&mut sw, 200.0, 1_000);
        assert_eq!(lod_level_count(&sw), 3);
        // Sorted by distance threshold
        assert!((lod_distance_threshold(&sw, 0) - 10.0).abs() < 1e-5);
        assert!((lod_distance_threshold(&sw, 1) - 50.0).abs() < 1e-5);
        assert!((lod_distance_threshold(&sw, 2) - 200.0).abs() < 1e-5);
    }

    #[test]
    fn test_select_level_close() {
        let sw = make_switcher();
        // Distance 5 < 10 → level 0 (highest detail)
        assert_eq!(lod_select_level(&sw, 5.0), 0);
    }

    #[test]
    fn test_select_level_mid() {
        let sw = make_switcher();
        // Distance 25 > 10 but ≤ 50 → level 1
        assert_eq!(lod_select_level(&sw, 25.0), 1);
    }

    #[test]
    fn test_select_level_far() {
        let sw = make_switcher();
        // Distance 500 > all thresholds → last level
        assert_eq!(lod_select_level(&sw, 500.0), 2);
    }

    #[test]
    fn test_vertex_count() {
        let sw = make_switcher();
        assert_eq!(lod_vertex_count_at(&sw, 0), 10_000);
        assert_eq!(lod_vertex_count_at(&sw, 2), 1_000);
        assert_eq!(lod_vertex_count_at(&sw, 99), 0);
    }

    #[test]
    fn test_enable_disable() {
        let cfg = default_lod_config();
        let mut sw = new_lod_switcher(&cfg);
        set_lod_enabled(&mut sw, false);
        assert!(!lod_is_enabled(&sw));
        set_lod_enabled(&mut sw, true);
        assert!(lod_is_enabled(&sw));
    }

    #[test]
    fn test_reduction_ratio() {
        let sw = make_switcher();
        // Level 0 ratio should be 1.0
        let r0 = lod_reduction_ratio(&sw, 0);
        assert!((r0 - 1.0).abs() < 1e-5);
        // Level 2: 1000 / 10000 = 0.1
        let r2 = lod_reduction_ratio(&sw, 2);
        assert!((r2 - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_select_empty_switcher() {
        let cfg = default_lod_config();
        let sw = new_lod_switcher(&cfg);
        assert_eq!(lod_select_level(&sw, 100.0), 0);
    }
}
