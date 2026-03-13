// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Real-time slider-driven morph target updates.
//!
//! [`MorphUpdater`] manages a set of [`MorphSlider`]s and applies dirty
//! sliders to a flat position buffer using pre-computed delta targets.
//!
//! Throttling: updates are skipped when the last GPU upload was less than
//! 16 ms ago (60 fps cap), preventing wasted uploads on stale data.

use std::collections::HashMap;
use std::time::{Duration, Instant};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Minimum interval between GPU uploads (≈ 60 fps).
const MIN_UPDATE_INTERVAL: Duration = Duration::from_millis(16);

// ── MorphSlider ───────────────────────────────────────────────────────────────

/// A named morph target slider with a normalised value in `[min, max]`.
#[derive(Debug, Clone)]
pub struct MorphSlider {
    /// Morph target name (must match keys in the target map).
    pub name: String,
    /// Current slider value.
    pub value: f32,
    /// Minimum allowed value (typically `0.0`).
    pub min: f32,
    /// Maximum allowed value (typically `1.0`).
    pub max: f32,
    /// `true` when the value has changed since the last flush.
    pub dirty: bool,
}

impl MorphSlider {
    /// Create a new slider with value clamped to `[min, max]`.
    pub fn new(name: &str, value: f32, min: f32, max: f32) -> Self {
        MorphSlider {
            name: name.to_string(),
            value: value.clamp(min, max),
            min,
            max,
            dirty: false,
        }
    }

    /// Set the slider value, clamping to `[self.min, self.max]`.
    ///
    /// Marks the slider dirty only when the value actually changes.
    pub fn set_value(&mut self, new_value: f32) {
        let clamped = new_value.clamp(self.min, self.max);
        if (clamped - self.value).abs() > f32::EPSILON {
            self.value = clamped;
            self.dirty = true;
        }
    }

    /// Clear the dirty flag.
    pub fn flush(&mut self) {
        self.dirty = false;
    }

    /// Normalised value in `[0, 1]` relative to `[min, max]`.
    pub fn normalized(&self) -> f32 {
        let span = self.max - self.min;
        if span.abs() < f32::EPSILON {
            return 0.0;
        }
        (self.value - self.min) / span
    }
}

impl Default for MorphSlider {
    fn default() -> Self {
        MorphSlider::new("default", 0.0, 0.0, 1.0)
    }
}

// ── MorphTarget type alias ────────────────────────────────────────────────────

/// A morph target is a list of `(vertex_index, delta_x, delta_y, delta_z)` tuples.
pub type MorphTargetDeltas = Vec<(u32, f32, f32, f32)>;

// ── MorphUpdater ──────────────────────────────────────────────────────────────

/// Manages morph sliders and applies their deltas to a CPU-side position buffer.
///
/// # Usage
///
/// ```ignore
/// let mut updater = MorphUpdater::new();
/// updater.add_slider(MorphSlider::new("brow_raise", 0.0, 0.0, 1.0));
/// updater.set_slider("brow_raise", 0.7);
///
/// if updater.should_update() {
///     updater.apply_dirty_to_mesh(&mut positions, &targets);
///     updater.flush_dirty();
/// }
/// ```
#[derive(Debug)]
pub struct MorphUpdater {
    sliders: Vec<MorphSlider>,
    /// Instant of the last successful GPU upload.
    last_update: Option<Instant>,
}

impl MorphUpdater {
    /// Create an empty [`MorphUpdater`].
    pub fn new() -> Self {
        MorphUpdater {
            sliders: Vec::new(),
            last_update: None,
        }
    }

    /// Add a slider to the updater.
    ///
    /// If a slider with the same name already exists its value is updated
    /// rather than adding a duplicate.
    pub fn add_slider(&mut self, slider: MorphSlider) {
        if let Some(existing) = self.sliders.iter_mut().find(|s| s.name == slider.name) {
            existing.set_value(slider.value);
        } else {
            self.sliders.push(slider);
        }
    }

    /// Set the value of a named slider.
    ///
    /// If no slider with that name exists, a new one with range `[0, 1]` is
    /// added automatically.
    pub fn set_slider(&mut self, name: &str, value: f32) {
        if let Some(s) = self.sliders.iter_mut().find(|s| s.name == name) {
            s.set_value(value);
        } else {
            let mut new_slider = MorphSlider::new(name, value, 0.0, 1.0);
            new_slider.dirty = true;
            self.sliders.push(new_slider);
        }
    }

    /// Return references to all sliders that are marked dirty.
    pub fn dirty_sliders(&self) -> Vec<&MorphSlider> {
        self.sliders.iter().filter(|s| s.dirty).collect()
    }

    /// Returns `true` if any slider is dirty **and** the throttle window has
    /// elapsed (i.e., at least 16 ms since the last update).
    pub fn should_update(&self) -> bool {
        if self.dirty_sliders().is_empty() {
            return false;
        }
        match self.last_update {
            None => true,
            Some(last) => last.elapsed() >= MIN_UPDATE_INTERVAL,
        }
    }

    /// Apply all dirty sliders' deltas to the flat `positions` buffer.
    ///
    /// `positions` is a flat `[x0, y0, z0, x1, y1, z1, ...]` array of
    /// `f32` values. Each entry in `targets` maps a morph name to a list of
    /// `(vertex_index, delta_x, delta_y, delta_z)` tuples.
    ///
    /// Only sliders that are both dirty **and** have a matching entry in
    /// `targets` contribute to the update.  Vertex indices that exceed the
    /// buffer bounds are silently skipped.
    pub fn apply_dirty_to_mesh(
        &self,
        positions: &mut [f32],
        targets: &HashMap<String, MorphTargetDeltas>,
    ) {
        for slider in self.dirty_sliders() {
            let Some(deltas) = targets.get(&slider.name) else {
                continue;
            };
            let weight = slider.value;
            for &(vertex_idx, dx, dy, dz) in deltas {
                let base = (vertex_idx as usize).saturating_mul(3);
                // Bounds-check before writing; skip silently if OOB.
                if base + 2 >= positions.len() {
                    continue;
                }
                positions[base] += dx * weight;
                positions[base + 1] += dy * weight;
                positions[base + 2] += dz * weight;
            }
        }
    }

    /// Clear dirty flags on all sliders and record the update timestamp.
    ///
    /// Call this after successfully uploading the updated mesh to the GPU.
    pub fn flush_dirty(&mut self) {
        for s in &mut self.sliders {
            s.flush();
        }
        self.last_update = Some(Instant::now());
    }

    /// Return the total number of managed sliders.
    pub fn slider_count(&self) -> usize {
        self.sliders.len()
    }

    /// Return a reference to a slider by name, if it exists.
    pub fn slider(&self, name: &str) -> Option<&MorphSlider> {
        self.sliders.iter().find(|s| s.name == name)
    }

    /// Return a mutable reference to a slider by name, if it exists.
    pub fn slider_mut(&mut self, name: &str) -> Option<&mut MorphSlider> {
        self.sliders.iter_mut().find(|s| s.name == name)
    }

    /// Remove a slider by name.  Returns `true` if a slider was found and
    /// removed.
    pub fn remove_slider(&mut self, name: &str) -> bool {
        let before = self.sliders.len();
        self.sliders.retain(|s| s.name != name);
        self.sliders.len() < before
    }

    /// Reset all slider values to their minimum and clear dirty flags.
    pub fn reset_all(&mut self) {
        for s in &mut self.sliders {
            s.value = s.min;
            s.dirty = false;
        }
    }

    /// Elapsed milliseconds since the last flush, or `None` if never flushed.
    pub fn ms_since_last_update(&self) -> Option<f64> {
        self.last_update.map(|t| t.elapsed().as_secs_f64() * 1000.0)
    }
}

impl Default for MorphUpdater {
    fn default() -> Self {
        MorphUpdater::new()
    }
}

// ── Helper ────────────────────────────────────────────────────────────────────

/// Build a zeroed flat position buffer with `vertex_count` vertices.
///
/// Useful in tests and tools when constructing synthetic meshes.
pub fn zero_positions(vertex_count: usize) -> Vec<f32> {
    vec![0.0f32; vertex_count * 3]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_simple_target() -> HashMap<String, MorphTargetDeltas> {
        let mut map = HashMap::new();
        // vertex 0: delta (+1, 0, 0); vertex 1: delta (0, +2, 0)
        map.insert(
            "smile".to_string(),
            vec![(0u32, 1.0, 0.0, 0.0), (1u32, 0.0, 2.0, 0.0)],
        );
        map
    }

    #[test]
    fn morph_slider_clamps_value() {
        let s = MorphSlider::new("test", 5.0, 0.0, 1.0);
        assert!((s.value - 1.0).abs() < f32::EPSILON, "should clamp to max");
    }

    #[test]
    fn morph_slider_set_value_marks_dirty() {
        let mut s = MorphSlider::new("test", 0.0, 0.0, 1.0);
        assert!(!s.dirty);
        s.set_value(0.5);
        assert!(s.dirty);
    }

    #[test]
    fn morph_slider_same_value_not_dirty() {
        let mut s = MorphSlider::new("test", 0.5, 0.0, 1.0);
        s.dirty = false;
        s.set_value(0.5);
        assert!(!s.dirty, "no change should not set dirty");
    }

    #[test]
    fn morph_slider_normalized_midpoint() {
        let s = MorphSlider::new("test", 0.5, 0.0, 1.0);
        assert!((s.normalized() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn morph_slider_flush_clears_dirty() {
        let mut s = MorphSlider::new("test", 0.0, 0.0, 1.0);
        s.dirty = true;
        s.flush();
        assert!(!s.dirty);
    }

    #[test]
    fn updater_set_slider_creates_new() {
        let mut u = MorphUpdater::new();
        u.set_slider("brow", 0.3);
        assert_eq!(u.slider_count(), 1);
    }

    #[test]
    fn updater_dirty_sliders_empty_initially() {
        let u = MorphUpdater::new();
        assert!(u.dirty_sliders().is_empty());
    }

    #[test]
    fn updater_dirty_sliders_after_set() {
        let mut u = MorphUpdater::new();
        u.set_slider("brow", 0.5);
        assert_eq!(u.dirty_sliders().len(), 1);
    }

    #[test]
    fn updater_apply_adds_delta_to_positions() {
        let mut u = MorphUpdater::new();
        u.set_slider("smile", 1.0); // weight = 1
        let targets = make_simple_target();
        let mut pos = zero_positions(2);
        u.apply_dirty_to_mesh(&mut pos, &targets);
        // vertex 0 x should be +1.0
        assert!((pos[0] - 1.0).abs() < 1e-6, "pos[0] = {}", pos[0]);
        // vertex 1 y should be +2.0
        assert!((pos[4] - 2.0).abs() < 1e-6, "pos[4] = {}", pos[4]);
    }

    #[test]
    fn updater_apply_scales_by_weight() {
        let mut u = MorphUpdater::new();
        u.set_slider("smile", 0.5); // weight = 0.5
        let targets = make_simple_target();
        let mut pos = zero_positions(2);
        u.apply_dirty_to_mesh(&mut pos, &targets);
        assert!(
            (pos[0] - 0.5).abs() < 1e-6,
            "expected 0.5 delta at weight 0.5"
        );
    }

    #[test]
    fn updater_apply_oob_vertex_skipped() {
        let mut u = MorphUpdater::new();
        u.set_slider("smile", 1.0);
        // Only 1 vertex (3 floats), but target references vertex 1 (index 3+)
        let targets = make_simple_target();
        let mut pos = zero_positions(1);
        // Should not panic
        u.apply_dirty_to_mesh(&mut pos, &targets);
    }

    #[test]
    fn updater_flush_dirty_clears_flags() {
        let mut u = MorphUpdater::new();
        u.set_slider("brow", 0.3);
        u.flush_dirty();
        assert!(u.dirty_sliders().is_empty());
    }

    #[test]
    fn updater_should_update_false_when_no_dirty() {
        let u = MorphUpdater::new();
        assert!(!u.should_update());
    }

    #[test]
    fn updater_should_update_true_when_dirty_and_never_flushed() {
        let mut u = MorphUpdater::new();
        u.set_slider("brow", 0.5);
        assert!(u.should_update());
    }

    #[test]
    fn updater_remove_slider() {
        let mut u = MorphUpdater::new();
        u.set_slider("brow", 0.3);
        let removed = u.remove_slider("brow");
        assert!(removed);
        assert_eq!(u.slider_count(), 0);
    }

    #[test]
    fn updater_reset_all_zeroes_values() {
        let mut u = MorphUpdater::new();
        u.set_slider("brow", 0.8);
        u.reset_all();
        let s = u.slider("brow").expect("slider should still exist");
        assert!((s.value - 0.0).abs() < f32::EPSILON);
        assert!(!s.dirty);
    }

    #[test]
    fn updater_add_slider_dedup_by_name() {
        let mut u = MorphUpdater::new();
        u.add_slider(MorphSlider::new("brow", 0.3, 0.0, 1.0));
        u.add_slider(MorphSlider::new("brow", 0.7, 0.0, 1.0));
        assert_eq!(u.slider_count(), 1, "no duplicate should be created");
    }
}
