// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Distance-based Level-of-Detail (LOD) management.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodLevel {
    pub id: usize,
    pub max_distance: f32,
    pub vertex_ratio: f32,
    pub description: String,
}

impl LodLevel {
    pub fn new(id: usize, max_distance: f32, vertex_ratio: f32, description: &str) -> Self {
        Self {
            id,
            max_distance,
            vertex_ratio,
            description: description.to_string(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodState {
    pub current_level: usize,
    pub distance: f32,
    pub transition_blend: f32,
}

impl Default for LodState {
    fn default() -> Self {
        Self {
            current_level: 0,
            distance: 0.0,
            transition_blend: 0.0,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodManager {
    pub levels: Vec<LodLevel>,
    pub objects: Vec<(String, LodState)>,
    pub hysteresis: f32,
}

impl LodManager {
    pub fn new() -> Self {
        Self {
            levels: Vec::new(),
            objects: Vec::new(),
            hysteresis: 0.05,
        }
    }

    /// Add a LOD level in order. Levels should be added from nearest to farthest.
    pub fn add_level(&mut self, max_dist: f32, ratio: f32, desc: &str) {
        let id = self.levels.len();
        self.levels.push(LodLevel::new(id, max_dist, ratio, desc));
    }

    /// Register a new object and return its id.
    pub fn register_object(&mut self, name: &str) -> usize {
        let id = self.objects.len();
        self.objects.push((name.to_string(), LodState::default()));
        id
    }

    /// Update distance for an object and recompute its LOD level.
    pub fn update_distance(&mut self, obj_id: usize, distance: f32) {
        if let Some((_, state)) = self.objects.get_mut(obj_id) {
            state.distance = distance;
            state.current_level = select_lod_level(&self.levels, distance);
            state.transition_blend = lod_blend_factor(&self.levels, state.current_level, distance);
        }
    }

    /// Get current LOD level for an object.
    pub fn get_lod_level(&self, obj_id: usize) -> usize {
        self.objects
            .get(obj_id)
            .map(|(_, s)| s.current_level)
            .unwrap_or(0)
    }

    /// Get vertex ratio for an object's current LOD level.
    pub fn get_vertex_ratio(&self, obj_id: usize) -> f32 {
        let level_idx = self.get_lod_level(obj_id);
        self.levels
            .get(level_idx)
            .map(|l| l.vertex_ratio)
            .unwrap_or(1.0)
    }

    /// Create a manager pre-loaded with standard LOD levels.
    pub fn with_standard_levels() -> Self {
        let mut m = Self::new();
        for level in standard_lod_levels() {
            m.levels.push(level);
        }
        m
    }
}

impl Default for LodManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Five standard LOD levels: 0%, 25%, 50%, 75%, 100% of max distance (100 units).
pub fn standard_lod_levels() -> Vec<LodLevel> {
    vec![
        LodLevel::new(0, 10.0, 1.0, "LOD0 full detail"),
        LodLevel::new(1, 25.0, 0.75, "LOD1 75% detail"),
        LodLevel::new(2, 50.0, 0.50, "LOD2 50% detail"),
        LodLevel::new(3, 75.0, 0.25, "LOD3 25% detail"),
        LodLevel::new(4, 100.0, 0.10, "LOD4 10% detail"),
    ]
}

/// Select the appropriate LOD level index for a given distance.
pub fn select_lod_level(levels: &[LodLevel], distance: f32) -> usize {
    if levels.is_empty() {
        return 0;
    }
    for (i, level) in levels.iter().enumerate() {
        if distance <= level.max_distance {
            return i;
        }
    }
    levels.len() - 1
}

/// Compute a blend factor `[0,1]` for smooth transitions between LOD levels.
pub fn lod_blend_factor(levels: &[LodLevel], cur: usize, distance: f32) -> f32 {
    let Some(level) = levels.get(cur) else {
        return 0.0;
    };
    let lower_bound = if cur == 0 {
        0.0
    } else {
        levels[cur - 1].max_distance
    };
    let span = level.max_distance - lower_bound;
    if span <= 0.0 {
        return 0.0;
    }
    ((distance - lower_bound) / span).clamp(0.0, 1.0)
}

/// JSON stats for all registered objects.
pub fn lod_stats_json(manager: &LodManager) -> String {
    let entries: Vec<String> = manager
        .objects
        .iter()
        .enumerate()
        .map(|(i, (name, state))| {
            format!(
                r#"{{"id":{i},"name":"{name}","level":{},"distance":{:.4},"blend":{:.4}}}"#,
                state.current_level, state.distance, state.transition_blend
            )
        })
        .collect();
    format!(
        r#"{{"objects":[{}],"level_count":{}}}"#,
        entries.join(","),
        manager.levels.len()
    )
}

// ─── Tests ───────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn make_manager() -> LodManager {
        let mut m = LodManager::new();
        m.add_level(10.0, 1.0, "LOD0");
        m.add_level(30.0, 0.5, "LOD1");
        m.add_level(60.0, 0.25, "LOD2");
        m
    }

    #[test]
    fn add_level_increases_count() {
        let mut m = LodManager::new();
        assert_eq!(m.levels.len(), 0);
        m.add_level(10.0, 1.0, "a");
        m.add_level(20.0, 0.5, "b");
        assert_eq!(m.levels.len(), 2);
    }

    #[test]
    fn register_object_returns_sequential_ids() {
        let mut m = make_manager();
        let id0 = m.register_object("body");
        let id1 = m.register_object("head");
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
    }

    #[test]
    fn update_distance_zero_gives_lod_zero() {
        let mut m = make_manager();
        let obj = m.register_object("body");
        m.update_distance(obj, 0.0);
        assert_eq!(m.get_lod_level(obj), 0);
    }

    #[test]
    fn update_distance_mid_range_gives_lod_one() {
        let mut m = make_manager();
        let obj = m.register_object("body");
        m.update_distance(obj, 20.0);
        assert_eq!(m.get_lod_level(obj), 1);
    }

    #[test]
    fn update_distance_far_gives_last_lod() {
        let mut m = make_manager();
        let obj = m.register_object("body");
        m.update_distance(obj, 200.0);
        assert_eq!(m.get_lod_level(obj), 2);
    }

    #[test]
    fn vertex_ratio_correct_for_lod_zero() {
        let mut m = make_manager();
        let obj = m.register_object("body");
        m.update_distance(obj, 5.0);
        assert!((m.get_vertex_ratio(obj) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn vertex_ratio_correct_for_lod_one() {
        let mut m = make_manager();
        let obj = m.register_object("body");
        m.update_distance(obj, 20.0);
        assert!((m.get_vertex_ratio(obj) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn standard_levels_count() {
        let levels = standard_lod_levels();
        assert_eq!(levels.len(), 5);
    }

    #[test]
    fn standard_levels_ids_sequential() {
        let levels = standard_lod_levels();
        for (i, l) in levels.iter().enumerate() {
            assert_eq!(l.id, i);
        }
    }

    #[test]
    fn with_standard_levels_has_five_levels() {
        let m = LodManager::with_standard_levels();
        assert_eq!(m.levels.len(), 5);
    }

    #[test]
    fn select_lod_level_returns_zero_at_origin() {
        let levels = standard_lod_levels();
        assert_eq!(select_lod_level(&levels, 0.0), 0);
    }

    #[test]
    fn select_lod_level_returns_last_beyond_max() {
        let levels = standard_lod_levels();
        assert_eq!(select_lod_level(&levels, 9999.0), 4);
    }

    #[test]
    fn lod_blend_factor_within_bounds() {
        let levels = standard_lod_levels();
        let blend = lod_blend_factor(&levels, 1, 20.0);
        assert!((0.0..=1.0).contains(&blend));
    }

    #[test]
    fn lod_stats_json_non_empty() {
        let mut m = make_manager();
        let obj = m.register_object("body");
        m.update_distance(obj, 5.0);
        let j = lod_stats_json(&m);
        assert!(j.contains("body"));
    }
}
