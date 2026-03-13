// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Named camera viewpoint bookmarks (save/restore camera state).

#![allow(dead_code)]

/// Configuration for the bookmark store.
#[derive(Debug, Clone)]
pub struct ViewportBookmarkConfig {
    /// Maximum number of bookmarks to store (0 = unlimited).
    pub max_bookmarks: usize,
}

/// A saved camera state used by bookmarks.
/// Named `BookmarkCameraState` to avoid conflict with the top-level `CameraState` in lib.rs.
#[derive(Debug, Clone, PartialEq)]
pub struct BookmarkCameraState {
    /// Eye position in world space [x, y, z].
    pub position: [f32; 3],
    /// Look-target [x, y, z].
    pub target: [f32; 3],
    /// Up vector [x, y, z].
    pub up: [f32; 3],
    /// Vertical field of view in degrees.
    pub fov_deg: f32,
}

/// Stores named camera bookmarks.
#[derive(Debug, Clone)]
pub struct ViewportBookmarkStore {
    pub config: ViewportBookmarkConfig,
    /// (name, camera state) pairs.
    bookmarks: Vec<(String, BookmarkCameraState)>,
}

/// Returns the default [`ViewportBookmarkConfig`].
#[allow(dead_code)]
pub fn default_viewport_bookmark_config() -> ViewportBookmarkConfig {
    ViewportBookmarkConfig {
        max_bookmarks: 32,
    }
}

/// Creates a new, empty [`ViewportBookmarkStore`].
#[allow(dead_code)]
pub fn new_bookmark_store(cfg: ViewportBookmarkConfig) -> ViewportBookmarkStore {
    ViewportBookmarkStore {
        config: cfg,
        bookmarks: Vec::new(),
    }
}

/// Saves the given camera state under `name`.  Overwrites any existing entry with the same name.
/// Returns `false` if the store is full and the name is new.
#[allow(dead_code)]
pub fn bookmark_save(
    store: &mut ViewportBookmarkStore,
    name: &str,
    state: BookmarkCameraState,
) -> bool {
    // Overwrite existing
    if let Some(entry) = store.bookmarks.iter_mut().find(|(n, _)| n == name) {
        entry.1 = state;
        return true;
    }
    // Check capacity
    let max = store.config.max_bookmarks;
    if max > 0 && store.bookmarks.len() >= max {
        return false;
    }
    store.bookmarks.push((name.to_string(), state));
    true
}

/// Restores the camera state for `name`. Returns `None` if not found.
#[allow(dead_code)]
pub fn bookmark_restore(
    store: &ViewportBookmarkStore,
    name: &str,
) -> Option<BookmarkCameraState> {
    store
        .bookmarks
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, s)| s.clone())
}

/// Deletes the bookmark with the given name.  Returns `true` if found and removed.
#[allow(dead_code)]
pub fn bookmark_delete(store: &mut ViewportBookmarkStore, name: &str) -> bool {
    let before = store.bookmarks.len();
    store.bookmarks.retain(|(n, _)| n != name);
    store.bookmarks.len() < before
}

/// Returns the number of bookmarks stored.
#[allow(dead_code)]
pub fn bookmark_count(store: &ViewportBookmarkStore) -> usize {
    store.bookmarks.len()
}

/// Returns the names of all bookmarks in insertion order.
#[allow(dead_code)]
pub fn bookmark_names(store: &ViewportBookmarkStore) -> Vec<&str> {
    store.bookmarks.iter().map(|(n, _)| n.as_str()).collect()
}

/// Returns `true` if a bookmark with `name` exists.
#[allow(dead_code)]
pub fn bookmark_exists(store: &ViewportBookmarkStore, name: &str) -> bool {
    store.bookmarks.iter().any(|(n, _)| n == name)
}

/// Serialises the bookmark store to a JSON string.
#[allow(dead_code)]
pub fn bookmark_store_to_json(store: &ViewportBookmarkStore) -> String {
    let entries: Vec<String> = store
        .bookmarks
        .iter()
        .map(|(name, s)| {
            format!(
                "{{\"name\":\"{}\",\"position\":[{:.4},{:.4},{:.4}],\"fov\":{:.4}}}",
                name, s.position[0], s.position[1], s.position[2], s.fov_deg
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}

/// Removes all bookmarks.
#[allow(dead_code)]
pub fn bookmark_clear(store: &mut ViewportBookmarkStore) {
    store.bookmarks.clear();
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn default_cam() -> BookmarkCameraState {
    BookmarkCameraState {
        position: [0.0, 2.0, -5.0],
        target: [0.0, 0.9, 0.0],
        up: [0.0, 1.0, 0.0],
        fov_deg: 60.0,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_max_is_32() {
        let cfg = default_viewport_bookmark_config();
        assert_eq!(cfg.max_bookmarks, 32);
    }

    #[test]
    fn new_store_is_empty() {
        let store = new_bookmark_store(default_viewport_bookmark_config());
        assert_eq!(bookmark_count(&store), 0);
    }

    #[test]
    fn save_and_restore_roundtrip() {
        let mut store = new_bookmark_store(default_viewport_bookmark_config());
        let cam = default_cam();
        bookmark_save(&mut store, "front", cam.clone());
        let restored = bookmark_restore(&store, "front").expect("should succeed");
        assert_eq!(restored, cam);
    }

    #[test]
    fn save_overwrites_existing() {
        let mut store = new_bookmark_store(default_viewport_bookmark_config());
        bookmark_save(&mut store, "top", default_cam());
        let mut cam2 = default_cam();
        cam2.fov_deg = 90.0;
        bookmark_save(&mut store, "top", cam2.clone());
        assert_eq!(bookmark_count(&store), 1);
        let r = bookmark_restore(&store, "top").expect("should succeed");
        assert!((r.fov_deg - 90.0).abs() < 1e-5);
    }

    #[test]
    fn restore_missing_returns_none() {
        let store = new_bookmark_store(default_viewport_bookmark_config());
        assert!(bookmark_restore(&store, "nope").is_none());
    }

    #[test]
    fn delete_removes_bookmark() {
        let mut store = new_bookmark_store(default_viewport_bookmark_config());
        bookmark_save(&mut store, "cam1", default_cam());
        let ok = bookmark_delete(&mut store, "cam1");
        assert!(ok);
        assert_eq!(bookmark_count(&store), 0);
    }

    #[test]
    fn bookmark_names_in_order() {
        let mut store = new_bookmark_store(default_viewport_bookmark_config());
        bookmark_save(&mut store, "a", default_cam());
        bookmark_save(&mut store, "b", default_cam());
        let names = bookmark_names(&store);
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn bookmark_exists_true_and_false() {
        let mut store = new_bookmark_store(default_viewport_bookmark_config());
        bookmark_save(&mut store, "present", default_cam());
        assert!(bookmark_exists(&store, "present"));
        assert!(!bookmark_exists(&store, "absent"));
    }

    #[test]
    fn to_json_contains_name() {
        let mut store = new_bookmark_store(default_viewport_bookmark_config());
        bookmark_save(&mut store, "side", default_cam());
        let json = bookmark_store_to_json(&store);
        assert!(json.contains("\"side\""));
    }

    #[test]
    fn clear_removes_all() {
        let mut store = new_bookmark_store(default_viewport_bookmark_config());
        bookmark_save(&mut store, "x", default_cam());
        bookmark_save(&mut store, "y", default_cam());
        bookmark_clear(&mut store);
        assert_eq!(bookmark_count(&store), 0);
    }
}
