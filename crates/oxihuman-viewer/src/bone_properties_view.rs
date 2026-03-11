#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bone properties panel state.

/// Properties of a single bone.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BonePropertiesView {
    pub bone_name: String,
    pub head: [f32; 3],
    pub tail: [f32; 3],
    pub roll: f32,
    pub length: f32,
    pub connected: bool,
}

/// Create a default `BonePropertiesView` for an unnamed unit bone.
#[allow(dead_code)]
pub fn default_bone_properties_view() -> BonePropertiesView {
    BonePropertiesView {
        bone_name: "Bone".to_string(),
        head: [0.0, 0.0, 0.0],
        tail: [0.0, 1.0, 0.0],
        roll: 0.0,
        length: 1.0,
        connected: false,
    }
}

/// Set the bone name and head/tail positions; recompute length.
#[allow(dead_code)]
pub fn set_bone(view: &mut BonePropertiesView, name: &str, head: [f32; 3], tail: [f32; 3]) {
    view.bone_name = name.to_string();
    view.head = head;
    view.tail = tail;
    view.length = bone_length(view);
}

/// Compute the unit direction vector from head to tail.
#[allow(dead_code)]
pub fn bone_direction(view: &BonePropertiesView) -> [f32; 3] {
    let dx = view.tail[0] - view.head[0];
    let dy = view.tail[1] - view.head[1];
    let dz = view.tail[2] - view.head[2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt();
    if len < 1e-9 {
        [0.0, 1.0, 0.0]
    } else {
        [dx / len, dy / len, dz / len]
    }
}

/// Compute the Euclidean length of the bone.
#[allow(dead_code)]
pub fn bone_length(view: &BonePropertiesView) -> f32 {
    let dx = view.tail[0] - view.head[0];
    let dy = view.tail[1] - view.head[1];
    let dz = view.tail[2] - view.head[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bone_length_one() {
        let v = default_bone_properties_view();
        assert!((v.length - 1.0).abs() < 1e-6);
    }

    #[test]
    fn default_bone_name() {
        let v = default_bone_properties_view();
        assert_eq!(v.bone_name, "Bone");
    }

    #[test]
    fn set_bone_updates_name() {
        let mut v = default_bone_properties_view();
        set_bone(&mut v, "Spine", [0.0; 3], [0.0, 0.5, 0.0]);
        assert_eq!(v.bone_name, "Spine");
    }

    #[test]
    fn set_bone_recomputes_length() {
        let mut v = default_bone_properties_view();
        set_bone(&mut v, "Arm", [0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
        assert!((v.length - 5.0).abs() < 1e-5);
    }

    #[test]
    fn bone_direction_unit_length() {
        let mut v = default_bone_properties_view();
        set_bone(&mut v, "B", [0.0; 3], [1.0, 1.0, 1.0]);
        let d = bone_direction(&v);
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn bone_direction_degenerate() {
        let mut v = default_bone_properties_view();
        set_bone(&mut v, "B", [0.0; 3], [0.0; 3]);
        let d = bone_direction(&v);
        assert!((d[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn bone_length_zero() {
        let mut v = default_bone_properties_view();
        set_bone(&mut v, "B", [1.0, 2.0, 3.0], [1.0, 2.0, 3.0]);
        assert!((bone_length(&v)).abs() < 1e-6);
    }

    #[test]
    fn bone_length_positive() {
        let v = default_bone_properties_view();
        assert!(bone_length(&v) > 0.0);
    }

    #[test]
    fn default_not_connected() {
        let v = default_bone_properties_view();
        assert!(!v.connected);
    }

    #[test]
    fn bone_length_computed_on_set() {
        let mut v = default_bone_properties_view();
        set_bone(&mut v, "B", [0.0; 3], [0.0, 2.0, 0.0]);
        assert!((v.length - 2.0).abs() < 1e-5);
    }
}
