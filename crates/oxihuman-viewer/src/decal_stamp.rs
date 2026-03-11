// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Decal stamp — projective decal placement onto a mesh surface.

use std::f32::consts::FRAC_PI_4;

/// Blend mode for the stamped decal.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StampBlend {
    Multiply,
    Overlay,
    Normal,
    Add,
}

/// A single decal stamp instance.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecalStamp {
    /// World-space position.
    pub position: [f32; 3],
    /// Normal direction of the stamp projector.
    pub normal: [f32; 3],
    /// Half-size in local X and Y.
    pub half_size: [f32; 2],
    /// Opacity in `[0.0, 1.0]`.
    pub opacity: f32,
    /// Rotation around the normal in radians.
    pub rotation_rad: f32,
    pub blend: StampBlend,
    pub enabled: bool,
}

/// Collection of stamps.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecalStampSet {
    stamps: Vec<DecalStamp>,
    max_stamps: usize,
}

/// Create a new stamp set.
pub fn new_decal_stamp_set(max_stamps: usize) -> DecalStampSet {
    DecalStampSet {
        stamps: Vec::new(),
        max_stamps,
    }
}

/// Create a default stamp.
pub fn default_decal_stamp() -> DecalStamp {
    DecalStamp {
        position: [0.0; 3],
        normal: [0.0, 1.0, 0.0],
        half_size: [0.5, 0.5],
        opacity: 1.0,
        rotation_rad: 0.0,
        blend: StampBlend::Normal,
        enabled: true,
    }
}

/// Add a stamp; returns `Some(index)` on success, `None` if full.
pub fn ds_add_stamp(set: &mut DecalStampSet, stamp: DecalStamp) -> Option<usize> {
    if set.stamps.len() >= set.max_stamps {
        return None;
    }
    set.stamps.push(stamp);
    Some(set.stamps.len() - 1)
}

/// Remove a stamp by index.
pub fn ds_remove_stamp(set: &mut DecalStampSet, idx: usize) {
    if idx < set.stamps.len() {
        set.stamps.remove(idx);
    }
}

/// Number of stamps.
pub fn ds_count(set: &DecalStampSet) -> usize {
    set.stamps.len()
}

/// Count enabled stamps.
pub fn ds_enabled_count(set: &DecalStampSet) -> usize {
    set.stamps.iter().filter(|s| s.enabled).count()
}

/// Clear all stamps.
pub fn ds_clear(set: &mut DecalStampSet) {
    set.stamps.clear();
}

/// Set opacity for a stamp.
pub fn ds_set_opacity(set: &mut DecalStampSet, idx: usize, opacity: f32) {
    if let Some(s) = set.stamps.get_mut(idx) {
        s.opacity = opacity.clamp(0.0, 1.0);
    }
}

/// Set enabled for a stamp.
pub fn ds_set_enabled(set: &mut DecalStampSet, idx: usize, enabled: bool) {
    if let Some(s) = set.stamps.get_mut(idx) {
        s.enabled = enabled;
    }
}

/// Project a world-space point into the stamp's local UV space.
///
/// Returns `Some([u, v])` if the point falls within the stamp's half-size.
pub fn ds_project_point(stamp: &DecalStamp, point: [f32; 3]) -> Option<[f32; 2]> {
    let ref_angle = FRAC_PI_4; // used for rotation
    let _ = ref_angle;
    let dx = point[0] - stamp.position[0];
    let dz = point[2] - stamp.position[2];
    let cos_r = stamp.rotation_rad.cos();
    let sin_r = stamp.rotation_rad.sin();
    let lx = cos_r * dx + sin_r * dz;
    let lz = -sin_r * dx + cos_r * dz;
    let u = lx / stamp.half_size[0];
    let v = lz / stamp.half_size[1];
    if (0.0..=1.0).contains(&u.abs()) && (0.0..=1.0).contains(&v.abs()) {
        Some([u * 0.5 + 0.5, v * 0.5 + 0.5])
    } else {
        None
    }
}

/// Serialise.
pub fn ds_to_json(set: &DecalStampSet) -> String {
    format!(
        r#"{{"count":{},"enabled":{}}}"#,
        ds_count(set),
        ds_enabled_count(set)
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make_set() -> DecalStampSet {
        new_decal_stamp_set(10)
    }

    #[test]
    fn empty_set_zero_count() {
        assert_eq!(ds_count(&make_set()), 0);
    }

    #[test]
    fn add_stamp_increments_count() {
        let mut s = make_set();
        ds_add_stamp(&mut s, default_decal_stamp());
        assert_eq!(ds_count(&s), 1);
    }

    #[test]
    fn max_stamps_respected() {
        let mut s = new_decal_stamp_set(1);
        ds_add_stamp(&mut s, default_decal_stamp());
        let r = ds_add_stamp(&mut s, default_decal_stamp());
        assert!(r.is_none());
    }

    #[test]
    fn remove_stamp_decrements() {
        let mut s = make_set();
        ds_add_stamp(&mut s, default_decal_stamp());
        ds_remove_stamp(&mut s, 0);
        assert_eq!(ds_count(&s), 0);
    }

    #[test]
    fn clear_empties_set() {
        let mut s = make_set();
        ds_add_stamp(&mut s, default_decal_stamp());
        ds_clear(&mut s);
        assert_eq!(ds_count(&s), 0);
    }

    #[test]
    fn set_opacity_clamps() {
        let mut s = make_set();
        ds_add_stamp(&mut s, default_decal_stamp());
        ds_set_opacity(&mut s, 0, 5.0);
        assert!((s.stamps[0].opacity - 1.0).abs() < 1e-5);
    }

    #[test]
    fn project_point_inside_returns_some() {
        let stamp = default_decal_stamp();
        let result = ds_project_point(&stamp, [0.0, 0.0, 0.0]);
        assert!(result.is_some());
    }

    #[test]
    fn project_point_outside_returns_none() {
        let stamp = default_decal_stamp();
        let result = ds_project_point(&stamp, [10.0, 0.0, 10.0]);
        assert!(result.is_none());
    }

    #[test]
    fn json_has_count() {
        assert!(ds_to_json(&make_set()).contains("count"));
    }
}
