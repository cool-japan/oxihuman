// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Voxel grid debug view — renders occupancy or value voxels as colored cubes.

/// Voxel grid view configuration.
#[derive(Debug, Clone)]
pub struct VoxelGridView {
    pub enabled: bool,
    pub show_empty: bool,
    pub voxel_opacity: f32,
    pub value_min: f32,
    pub value_max: f32,
}

impl VoxelGridView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            show_empty: false,
            voxel_opacity: 0.6,
            value_min: 0.0,
            value_max: 1.0,
        }
    }
}

impl Default for VoxelGridView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new voxel grid view.
pub fn new_voxel_grid_view() -> VoxelGridView {
    VoxelGridView::new()
}

/// Enable or disable voxel grid display.
pub fn vgv_set_enabled(v: &mut VoxelGridView, enabled: bool) {
    v.enabled = enabled;
}

/// Toggle display of empty voxels.
pub fn vgv_set_show_empty(v: &mut VoxelGridView, show: bool) {
    v.show_empty = show;
}

/// Set voxel opacity.
pub fn vgv_set_voxel_opacity(v: &mut VoxelGridView, o: f32) {
    v.voxel_opacity = o.clamp(0.0, 1.0);
}

/// Set minimum value for color mapping.
pub fn vgv_set_value_min(v: &mut VoxelGridView, min: f32) {
    v.value_min = min;
}

/// Set maximum value for color mapping.
pub fn vgv_set_value_max(v: &mut VoxelGridView, max: f32) {
    v.value_max = max;
}

/// Normalize a value to 0-1 for color lookup.
pub fn vgv_normalize(v: &VoxelGridView, val: f32) -> f32 {
    let range = v.value_max - v.value_min;
    if range.abs() < 1e-9 {
        return 0.0;
    }
    ((val - v.value_min) / range).clamp(0.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn voxel_grid_view_to_json(v: &VoxelGridView) -> String {
    format!(
        r#"{{"enabled":{},"show_empty":{},"voxel_opacity":{:.4},"value_min":{:.4},"value_max":{:.4}}}"#,
        v.enabled, v.show_empty, v.voxel_opacity, v.value_min, v.value_max
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_voxel_grid_view();
        assert!(!v.enabled);
        assert!(!v.show_empty);
    }

    #[test]
    fn test_enable() {
        let mut v = new_voxel_grid_view();
        vgv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_show_empty() {
        let mut v = new_voxel_grid_view();
        vgv_set_show_empty(&mut v, true);
        assert!(v.show_empty);
    }

    #[test]
    fn test_opacity_clamp() {
        let mut v = new_voxel_grid_view();
        vgv_set_voxel_opacity(&mut v, 2.0);
        assert_eq!(v.voxel_opacity, 1.0);
    }

    #[test]
    fn test_value_min_set() {
        let mut v = new_voxel_grid_view();
        vgv_set_value_min(&mut v, -1.0);
        assert!((v.value_min - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_value_max_set() {
        let mut v = new_voxel_grid_view();
        vgv_set_value_max(&mut v, 5.0);
        assert!((v.value_max - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_mid() {
        let v = new_voxel_grid_view();
        assert!((vgv_normalize(&v, 0.5) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_json_keys() {
        let v = new_voxel_grid_view();
        let s = voxel_grid_view_to_json(&v);
        assert!(s.contains("voxel_opacity"));
    }

    #[test]
    fn test_clone() {
        let v = new_voxel_grid_view();
        let v2 = v.clone();
        assert!((v2.voxel_opacity - v.voxel_opacity).abs() < 1e-6);
    }
}
