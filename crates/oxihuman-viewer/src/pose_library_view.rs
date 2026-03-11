// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Pose library thumbnail view stub.

/// Thumbnail sort order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PoseSortOrder {
    ByName,
    ByDate,
    ByCategory,
    Manual,
}

/// A pose library entry.
#[derive(Debug, Clone)]
pub struct PoseEntry {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub thumbnail_size: [u32; 2],
}

/// Pose library view configuration.
#[derive(Debug, Clone)]
pub struct PoseLibraryView {
    pub poses: Vec<PoseEntry>,
    pub sort_order: PoseSortOrder,
    pub thumbnail_size: u32,
    pub columns: u32,
    pub enabled: bool,
}

impl PoseLibraryView {
    pub fn new() -> Self {
        PoseLibraryView {
            poses: Vec::new(),
            sort_order: PoseSortOrder::ByName,
            thumbnail_size: 64,
            columns: 4,
            enabled: true,
        }
    }
}

impl Default for PoseLibraryView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new pose library view.
pub fn new_pose_library_view() -> PoseLibraryView {
    PoseLibraryView::new()
}

/// Add a pose entry.
pub fn plv_add_pose(view: &mut PoseLibraryView, pose: PoseEntry) {
    view.poses.push(pose);
}

/// Clear all poses.
pub fn plv_clear(view: &mut PoseLibraryView) {
    view.poses.clear();
}

/// Set sort order.
pub fn plv_set_sort_order(view: &mut PoseLibraryView, order: PoseSortOrder) {
    view.sort_order = order;
}

/// Set thumbnail size in pixels.
pub fn plv_set_thumbnail_size(view: &mut PoseLibraryView, size: u32) {
    view.thumbnail_size = size.max(16);
}

/// Set column count.
pub fn plv_set_columns(view: &mut PoseLibraryView, columns: u32) {
    view.columns = columns.max(1);
}

/// Enable or disable.
pub fn plv_set_enabled(view: &mut PoseLibraryView, enabled: bool) {
    view.enabled = enabled;
}

/// Return pose count.
pub fn plv_pose_count(view: &PoseLibraryView) -> usize {
    view.poses.len()
}

/// Serialize to JSON-like string.
pub fn plv_to_json(view: &PoseLibraryView) -> String {
    let sort = match view.sort_order {
        PoseSortOrder::ByName => "by_name",
        PoseSortOrder::ByDate => "by_date",
        PoseSortOrder::ByCategory => "by_category",
        PoseSortOrder::Manual => "manual",
    };
    format!(
        r#"{{"pose_count":{},"sort_order":"{}","thumbnail_size":{},"columns":{},"enabled":{}}}"#,
        view.poses.len(),
        sort,
        view.thumbnail_size,
        view.columns,
        view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pose(id: u32) -> PoseEntry {
        PoseEntry {
            id,
            name: format!("pose_{id}"),
            category: "body".to_string(),
            thumbnail_size: [64, 64],
        }
    }

    #[test]
    fn test_initial_empty() {
        let v = new_pose_library_view();
        assert_eq!(plv_pose_count(&v), 0 /* no poses initially */);
    }

    #[test]
    fn test_add_pose() {
        let mut v = new_pose_library_view();
        plv_add_pose(&mut v, make_pose(0));
        assert_eq!(plv_pose_count(&v), 1 /* one pose after add */);
    }

    #[test]
    fn test_clear() {
        let mut v = new_pose_library_view();
        plv_add_pose(&mut v, make_pose(0));
        plv_clear(&mut v);
        assert_eq!(plv_pose_count(&v), 0 /* cleared */);
    }

    #[test]
    fn test_set_sort_order() {
        let mut v = new_pose_library_view();
        plv_set_sort_order(&mut v, PoseSortOrder::ByCategory);
        assert_eq!(
            v.sort_order,
            PoseSortOrder::ByCategory /* sort order must be set */
        );
    }

    #[test]
    fn test_thumbnail_size_min() {
        let mut v = new_pose_library_view();
        plv_set_thumbnail_size(&mut v, 0);
        assert_eq!(
            v.thumbnail_size,
            16 /* minimum thumbnail size must be 16 */
        );
    }

    #[test]
    fn test_columns_min() {
        let mut v = new_pose_library_view();
        plv_set_columns(&mut v, 0);
        assert_eq!(v.columns, 1 /* minimum columns must be 1 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_pose_library_view();
        plv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_pose_count() {
        let v = new_pose_library_view();
        let j = plv_to_json(&v);
        assert!(j.contains("\"pose_count\"") /* JSON must have pose_count */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_pose_library_view();
        assert!(v.enabled /* must be enabled by default */);
    }
}
