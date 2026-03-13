// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Incremental morph update system.
//!
//! Tracks which morph targets have changed since the last mesh build and caches
//! per-target position contributions so that only dirty targets need recomputation.

use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// DirtyTracker
// ---------------------------------------------------------------------------

/// Tracks which morph targets have been modified and need recomputation.
#[derive(Debug, Clone)]
pub struct DirtyTracker {
    dirty: HashSet<String>,
}

impl Default for DirtyTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl DirtyTracker {
    /// Create a new tracker with no dirty targets.
    pub fn new() -> Self {
        Self {
            dirty: HashSet::new(),
        }
    }

    /// Mark a single target as dirty (needs recomputation).
    pub fn mark_dirty(&mut self, name: &str) {
        self.dirty.insert(name.to_string());
    }

    /// Mark every known target as dirty. This accepts the set of all target
    /// names so the tracker does not need to maintain its own registry.
    pub fn mark_all_dirty(&mut self, all_names: &[&str]) {
        for name in all_names {
            self.dirty.insert((*name).to_string());
        }
    }

    /// Clear all dirty flags (call after a successful rebuild).
    pub fn clear(&mut self) {
        self.dirty.clear();
    }

    /// Returns `true` if the given target is dirty.
    pub fn is_dirty(&self, name: &str) -> bool {
        self.dirty.contains(name)
    }

    /// Number of targets currently marked dirty.
    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
    }

    /// Snapshot of all dirty target names (unordered).
    pub fn dirty_targets(&self) -> Vec<&str> {
        self.dirty.iter().map(|s| s.as_str()).collect()
    }

    /// Returns `true` when there are no dirty targets.
    pub fn is_clean(&self) -> bool {
        self.dirty.is_empty()
    }
}

// ---------------------------------------------------------------------------
// IncrementalMorphCache
// ---------------------------------------------------------------------------

/// Per-target weighted delta buffer stored as a flat `Vec<f32>` of length
/// `vertex_count * 3`. Entry `[i*3], [i*3+1], [i*3+2]` stores the xyz
/// displacement contributed by this target at vertex `i`.
#[derive(Debug, Clone)]
struct TargetContribution {
    /// Flat xyz contribution buffer (length = vertex_count * 3).
    buf: Vec<f32>,
}

/// Caches the weighted contribution of each morph target so that a full
/// summation is only needed once; subsequent updates recompute only the
/// targets that changed.
#[derive(Debug, Clone)]
pub struct IncrementalMorphCache {
    contributions: HashMap<String, TargetContribution>,
    vertex_count: usize,
}

impl IncrementalMorphCache {
    /// Create a new empty cache for a mesh with `vertex_count` vertices.
    pub fn new(vertex_count: usize) -> Self {
        Self {
            contributions: HashMap::new(),
            vertex_count,
        }
    }

    /// Return the vertex count this cache was created for.
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    /// Number of cached target contributions.
    pub fn target_count(&self) -> usize {
        self.contributions.len()
    }

    /// Update (or insert) the cached contribution for target `name`.
    ///
    /// `deltas` is a slice of `(vertex_id, dx, dy, dz)` sparse deltas and
    /// `weight` is the scalar weight to multiply them by.  The contribution
    /// buffer is zeroed first, then the weighted deltas are scattered in.
    pub fn update_target(
        &mut self,
        name: &str,
        deltas: &[(u32, f32, f32, f32)],
        weight: f32,
        vertex_count: usize,
    ) {
        let len = vertex_count * 3;
        let entry = self
            .contributions
            .entry(name.to_string())
            .or_insert_with(|| TargetContribution {
                buf: vec![0.0; len],
            });

        // Resize if needed (defensive).
        if entry.buf.len() != len {
            entry.buf.resize(len, 0.0);
        }

        // Zero out old contribution.
        for v in entry.buf.iter_mut() {
            *v = 0.0;
        }

        // Scatter weighted deltas.
        for &(vid, dx, dy, dz) in deltas {
            let idx = vid as usize * 3;
            if idx + 2 < len {
                entry.buf[idx] += weight * dx;
                entry.buf[idx + 1] += weight * dy;
                entry.buf[idx + 2] += weight * dz;
            }
        }
    }

    /// Remove a target's contribution from the cache.
    pub fn remove_target(&mut self, name: &str) {
        self.contributions.remove(name);
    }

    /// Full rebuild: `base_positions + sum(all contributions)`.
    ///
    /// `base_positions` is a flat `[x0, y0, z0, x1, y1, z1, ...]` array of
    /// length `vertex_count * 3`.  Returns a new flat position buffer of the
    /// same layout.
    pub fn rebuild_mesh(&self, base_positions: &[f32]) -> Vec<f32> {
        let len = base_positions.len();
        let mut out = base_positions.to_vec();
        for contrib in self.contributions.values() {
            let n = contrib.buf.len().min(len);
            for (out_val, &src_val) in out[..n].iter_mut().zip(contrib.buf[..n].iter()) {
                *out_val += src_val;
            }
        }
        out
    }

    /// Incremental rebuild: only recompute dirty targets.
    ///
    /// `current` is the mesh position buffer from the previous frame (mutated
    /// in-place).  For each dirty target the old contribution is subtracted
    /// and the new one (already stored via `update_target`) is added.
    ///
    /// `old_contributions` maps target name -> the *previous* contribution
    /// buffer that was already baked into `current`.  After this call,
    /// `current` reflects the latest cached contributions.
    ///
    /// Targets present in the dirty set but absent from `old_contributions`
    /// are treated as newly added (old contribution is zero).
    pub fn rebuild_incremental(
        &self,
        current: &mut [f32],
        dirty: &DirtyTracker,
        old_contributions: &HashMap<String, Vec<f32>>,
    ) {
        let len = current.len();

        for dirty_name in dirty.dirty_targets() {
            // Subtract old contribution if it existed.
            if let Some(old_buf) = old_contributions.get(dirty_name) {
                let n = old_buf.len().min(len);
                for i in 0..n {
                    current[i] -= old_buf[i];
                }
            }

            // Add new contribution.
            if let Some(new_contrib) = self.contributions.get(dirty_name) {
                let n = new_contrib.buf.len().min(len);
                for (cur_val, &src_val) in current[..n].iter_mut().zip(new_contrib.buf[..n].iter())
                {
                    *cur_val += src_val;
                }
            }
        }
    }

    /// Snapshot the current contribution buffer for a target (for use as
    /// `old_contributions` in the next incremental rebuild).
    pub fn snapshot_contribution(&self, name: &str) -> Option<Vec<f32>> {
        self.contributions.get(name).map(|c| c.buf.clone())
    }

    /// Snapshot all contributions (cheap clone of the inner buffers).
    pub fn snapshot_all(&self) -> HashMap<String, Vec<f32>> {
        self.contributions
            .iter()
            .map(|(k, v)| (k.clone(), v.buf.clone()))
            .collect()
    }

    /// Clear all cached contributions.
    pub fn clear(&mut self) {
        self.contributions.clear();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- DirtyTracker tests --

    #[test]
    fn tracker_starts_clean() {
        let tracker = DirtyTracker::new();
        assert!(tracker.is_clean());
        assert_eq!(tracker.dirty_count(), 0);
        assert!(!tracker.is_dirty("foo"));
    }

    #[test]
    fn mark_and_query() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty("height");
        tracker.mark_dirty("weight");

        assert!(tracker.is_dirty("height"));
        assert!(tracker.is_dirty("weight"));
        assert!(!tracker.is_dirty("age"));
        assert_eq!(tracker.dirty_count(), 2);
    }

    #[test]
    fn mark_all_dirty() {
        let mut tracker = DirtyTracker::new();
        let names = vec!["a", "b", "c"];
        tracker.mark_all_dirty(&names);
        assert_eq!(tracker.dirty_count(), 3);
        for name in &names {
            assert!(tracker.is_dirty(name));
        }
    }

    #[test]
    fn clear_resets() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty("x");
        tracker.clear();
        assert!(tracker.is_clean());
        assert_eq!(tracker.dirty_count(), 0);
    }

    #[test]
    fn dirty_targets_returns_all_marked() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty("alpha");
        tracker.mark_dirty("beta");
        let mut targets = tracker.dirty_targets();
        targets.sort();
        assert_eq!(targets, vec!["alpha", "beta"]);
    }

    #[test]
    fn duplicate_mark_is_idempotent() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty("x");
        tracker.mark_dirty("x");
        assert_eq!(tracker.dirty_count(), 1);
    }

    // -- IncrementalMorphCache tests --

    fn base_3v() -> Vec<f32> {
        // 3 vertices: (0,0,0), (1,0,0), (0,1,0)
        vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]
    }

    #[test]
    fn empty_cache_rebuild_equals_base() {
        let cache = IncrementalMorphCache::new(3);
        let base = base_3v();
        let result = cache.rebuild_mesh(&base);
        assert_eq!(result, base);
    }

    #[test]
    fn update_target_and_rebuild() {
        let mut cache = IncrementalMorphCache::new(3);
        // Target moves vertex 1 by (0.5, 0, 0) with weight 1.0
        let deltas = vec![(1u32, 0.5f32, 0.0f32, 0.0f32)];
        cache.update_target("height", &deltas, 1.0, 3);

        let base = base_3v();
        let result = cache.rebuild_mesh(&base);

        // vertex 1 x should be 1.0 + 0.5 = 1.5
        assert!((result[3] - 1.5).abs() < 1e-6);
        // other vertices unchanged
        assert!((result[0] - 0.0).abs() < 1e-6);
        assert!((result[6] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn update_target_with_weight() {
        let mut cache = IncrementalMorphCache::new(3);
        let deltas = vec![(0u32, 2.0f32, 0.0f32, 0.0f32)];
        cache.update_target("stretch", &deltas, 0.5, 3);

        let base = base_3v();
        let result = cache.rebuild_mesh(&base);

        // vertex 0 x: 0.0 + 2.0 * 0.5 = 1.0
        assert!((result[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn remove_target_excludes_contribution() {
        let mut cache = IncrementalMorphCache::new(3);
        let deltas = vec![(0u32, 10.0f32, 0.0f32, 0.0f32)];
        cache.update_target("big", &deltas, 1.0, 3);
        cache.remove_target("big");

        let base = base_3v();
        let result = cache.rebuild_mesh(&base);
        assert!((result[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn multiple_targets_sum() {
        let mut cache = IncrementalMorphCache::new(3);
        cache.update_target("a", &[(0u32, 1.0f32, 0.0f32, 0.0f32)], 1.0, 3);
        cache.update_target("b", &[(0u32, 0.0f32, 2.0f32, 0.0f32)], 1.0, 3);

        let base = base_3v();
        let result = cache.rebuild_mesh(&base);
        // vertex 0: (0+1, 0+2, 0) = (1, 2, 0)
        assert!((result[0] - 1.0).abs() < 1e-6);
        assert!((result[1] - 2.0).abs() < 1e-6);
        assert!((result[2] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn incremental_rebuild_matches_full() {
        let base = base_3v();
        let mut cache = IncrementalMorphCache::new(3);

        // Initial state: target "h" with weight 0.5
        let deltas = vec![(1u32, 2.0f32, 0.0f32, 0.0f32)];
        cache.update_target("h", &deltas, 0.5, 3);
        let old_snap = cache.snapshot_all();
        let mut current = cache.rebuild_mesh(&base);

        // Change weight to 0.8
        cache.update_target("h", &deltas, 0.8, 3);
        let mut dirty = DirtyTracker::new();
        dirty.mark_dirty("h");

        cache.rebuild_incremental(&mut current, &dirty, &old_snap);

        // Full rebuild for comparison
        let full = cache.rebuild_mesh(&base);

        for (i, (a, b)) in current.iter().zip(full.iter()).enumerate() {
            assert!(
                (a - b).abs() < 1e-5,
                "mismatch at index {}: incremental={}, full={}",
                i,
                a,
                b
            );
        }
    }

    #[test]
    fn incremental_new_target_matches_full() {
        let base = base_3v();
        let mut cache = IncrementalMorphCache::new(3);

        // Build with one target
        cache.update_target("a", &[(0u32, 1.0f32, 0.0f32, 0.0f32)], 1.0, 3);
        let old_snap = cache.snapshot_all();
        let mut current = cache.rebuild_mesh(&base);

        // Add a second target
        cache.update_target("b", &[(2u32, 0.0f32, 0.0f32, 3.0f32)], 1.0, 3);
        let mut dirty = DirtyTracker::new();
        dirty.mark_dirty("b");

        cache.rebuild_incremental(&mut current, &dirty, &old_snap);

        let full = cache.rebuild_mesh(&base);
        for (i, (a, b)) in current.iter().zip(full.iter()).enumerate() {
            assert!(
                (a - b).abs() < 1e-5,
                "mismatch at index {}: incremental={}, full={}",
                i,
                a,
                b
            );
        }
    }

    #[test]
    fn incremental_remove_target_matches_full() {
        let base = base_3v();
        let mut cache = IncrementalMorphCache::new(3);

        cache.update_target("a", &[(0u32, 5.0f32, 0.0f32, 0.0f32)], 1.0, 3);
        cache.update_target("b", &[(1u32, 0.0f32, 3.0f32, 0.0f32)], 1.0, 3);
        let old_snap = cache.snapshot_all();
        let mut current = cache.rebuild_mesh(&base);

        // Remove target "a"
        cache.remove_target("a");
        let mut dirty = DirtyTracker::new();
        dirty.mark_dirty("a");

        cache.rebuild_incremental(&mut current, &dirty, &old_snap);

        let full = cache.rebuild_mesh(&base);
        for (i, (a, b)) in current.iter().zip(full.iter()).enumerate() {
            assert!(
                (a - b).abs() < 1e-5,
                "mismatch at index {}: incremental={}, full={}",
                i,
                a,
                b
            );
        }
    }

    #[test]
    fn incremental_no_dirty_is_noop() {
        let base = base_3v();
        let mut cache = IncrementalMorphCache::new(3);
        cache.update_target("x", &[(0u32, 1.0f32, 2.0f32, 3.0f32)], 1.0, 3);
        let old_snap = cache.snapshot_all();
        let mut current = cache.rebuild_mesh(&base);
        let before = current.clone();

        let dirty = DirtyTracker::new(); // nothing dirty
        cache.rebuild_incremental(&mut current, &dirty, &old_snap);

        assert_eq!(current, before);
    }

    #[test]
    fn snapshot_contribution_round_trip() {
        let mut cache = IncrementalMorphCache::new(3);
        let deltas = vec![(0u32, 1.0f32, 2.0f32, 3.0f32)];
        cache.update_target("t", &deltas, 0.5, 3);

        let snap = cache.snapshot_contribution("t");
        assert!(snap.is_some());
        let snap = snap.expect("snapshot should exist");
        // vertex 0: (0.5, 1.0, 1.5)
        assert!((snap[0] - 0.5).abs() < 1e-6);
        assert!((snap[1] - 1.0).abs() < 1e-6);
        assert!((snap[2] - 1.5).abs() < 1e-6);
    }

    #[test]
    fn out_of_bounds_vertex_id_is_ignored() {
        let mut cache = IncrementalMorphCache::new(2);
        // vertex_count=2 so valid indices are 0..5. vid=10 is out of bounds.
        let deltas = vec![(10u32, 1.0f32, 1.0f32, 1.0f32)];
        cache.update_target("oob", &deltas, 1.0, 2);

        let base = vec![0.0f32; 6];
        let result = cache.rebuild_mesh(&base);
        // Nothing should have changed.
        assert_eq!(result, base);
    }

    #[test]
    fn clear_empties_cache() {
        let mut cache = IncrementalMorphCache::new(3);
        cache.update_target("a", &[(0u32, 1.0, 0.0, 0.0)], 1.0, 3);
        assert_eq!(cache.target_count(), 1);
        cache.clear();
        assert_eq!(cache.target_count(), 0);
    }

    /// End-to-end: simulate a multi-frame scenario where weights change each
    /// frame and verify incremental always matches full rebuild.
    #[test]
    fn multi_frame_incremental_consistency() {
        let base = base_3v();
        let deltas_h = vec![
            (0u32, 1.0f32, 0.0f32, 0.0f32),
            (1u32, 0.0f32, 0.5f32, 0.0f32),
        ];
        let deltas_w = vec![
            (1u32, 0.0f32, 0.0f32, 1.0f32),
            (2u32, 0.3f32, 0.0f32, 0.0f32),
        ];

        let weight_sequences: &[(f32, f32)] = &[
            (0.0, 0.0),
            (0.5, 0.2),
            (0.8, 0.6),
            (1.0, 1.0),
            (0.3, 0.9),
            (0.0, 0.0),
        ];

        let mut cache = IncrementalMorphCache::new(3);
        let mut current = base.clone();
        let mut old_snap: HashMap<String, Vec<f32>> = HashMap::new();

        for &(wh, ww) in weight_sequences {
            // Update targets with new weights
            cache.update_target("h", &deltas_h, wh, 3);
            cache.update_target("w", &deltas_w, ww, 3);

            let mut dirty = DirtyTracker::new();
            dirty.mark_dirty("h");
            dirty.mark_dirty("w");

            cache.rebuild_incremental(&mut current, &dirty, &old_snap);

            let full = cache.rebuild_mesh(&base);

            for (i, (a, b)) in current.iter().zip(full.iter()).enumerate() {
                assert!(
                    (a - b).abs() < 1e-4,
                    "frame wh={}, ww={}: mismatch at {}: inc={}, full={}",
                    wh,
                    ww,
                    i,
                    a,
                    b
                );
            }

            old_snap = cache.snapshot_all();
        }
    }
}
