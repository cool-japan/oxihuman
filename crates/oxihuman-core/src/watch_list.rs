//! Ordered list of watched file/asset paths with change-state tracking.
//!
//! Paths are added to a [`WatchList`]. External code calls
//! [`watch_mark_changed`] to signal that a path has changed. The list keeps
//! the ordered insertion sequence and can report which entries have pending
//! changes.

/// Configuration for a watch list.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WatchListConfig {
    /// Maximum number of entries (0 = unlimited).
    pub max_entries: usize,
    /// Whether duplicate paths are silently ignored on insertion.
    pub deduplicate: bool,
}

/// A single watched entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WatchEntry {
    /// The path being watched.
    pub path: String,
    /// Whether this entry has a pending change notification.
    pub changed: bool,
}

/// An ordered list of watched paths.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WatchList {
    config: WatchListConfig,
    entries: Vec<WatchEntry>,
}

/// Return a default [`WatchListConfig`].
#[allow(dead_code)]
pub fn default_watch_list_config() -> WatchListConfig {
    WatchListConfig {
        max_entries: 0,
        deduplicate: true,
    }
}

/// Create a new, empty [`WatchList`].
#[allow(dead_code)]
pub fn new_watch_list(config: WatchListConfig) -> WatchList {
    WatchList {
        config,
        entries: Vec::new(),
    }
}

/// Add a path to the watch list.  Returns `true` if added, `false` if rejected
/// (duplicate when `deduplicate` is set, or capacity exceeded).
#[allow(dead_code)]
pub fn watch_add(wl: &mut WatchList, path: &str) -> bool {
    if wl.config.deduplicate && wl.entries.iter().any(|e| e.path == path) {
        return false;
    }
    if wl.config.max_entries > 0 && wl.entries.len() >= wl.config.max_entries {
        return false;
    }
    wl.entries.push(WatchEntry {
        path: path.to_owned(),
        changed: false,
    });
    true
}

/// Remove the first entry matching `path`.  Returns `true` if removed.
#[allow(dead_code)]
pub fn watch_remove(wl: &mut WatchList, path: &str) -> bool {
    if let Some(pos) = wl.entries.iter().position(|e| e.path == path) {
        wl.entries.remove(pos);
        return true;
    }
    false
}

/// Mark the entry with the given `path` as changed.  Returns `true` if found.
#[allow(dead_code)]
pub fn watch_mark_changed(wl: &mut WatchList, path: &str) -> bool {
    if let Some(e) = wl.entries.iter_mut().find(|e| e.path == path) {
        e.changed = true;
        return true;
    }
    false
}

/// Return `true` if the entry for `path` has a pending change.
#[allow(dead_code)]
pub fn watch_is_changed(wl: &WatchList, path: &str) -> bool {
    wl.entries
        .iter()
        .any(|e| e.path == path && e.changed)
}

/// Return a list of paths that are currently marked as changed.
#[allow(dead_code)]
pub fn watch_changed_entries(wl: &WatchList) -> Vec<&str> {
    wl.entries
        .iter()
        .filter(|e| e.changed)
        .map(|e| e.path.as_str())
        .collect()
}

/// Clear the changed flag on all entries.
#[allow(dead_code)]
pub fn watch_clear_changed(wl: &mut WatchList) {
    for e in &mut wl.entries {
        e.changed = false;
    }
}

/// Return the total number of watched entries.
#[allow(dead_code)]
pub fn watch_count(wl: &WatchList) -> usize {
    wl.entries.len()
}

/// Serialize the watch list to a compact JSON string.
#[allow(dead_code)]
pub fn watch_list_to_json(wl: &WatchList) -> String {
    let entries_json: Vec<String> = wl
        .entries
        .iter()
        .map(|e| format!(r#"{{"path":"{}","changed":{}}}"#, e.path, e.changed))
        .collect();
    format!(
        r#"{{"count":{},"entries":[{}]}}"#,
        wl.entries.len(),
        entries_json.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_wl() -> WatchList {
        new_watch_list(default_watch_list_config())
    }

    #[test]
    fn test_add_and_count() {
        let mut wl = make_wl();
        watch_add(&mut wl, "assets/tex.png");
        assert_eq!(watch_count(&wl), 1);
    }

    #[test]
    fn test_deduplicate_prevents_repeat() {
        let mut wl = make_wl();
        assert!(watch_add(&mut wl, "a.obj"));
        assert!(!watch_add(&mut wl, "a.obj"));
        assert_eq!(watch_count(&wl), 1);
    }

    #[test]
    fn test_remove() {
        let mut wl = make_wl();
        watch_add(&mut wl, "a.obj");
        assert!(watch_remove(&mut wl, "a.obj"));
        assert_eq!(watch_count(&wl), 0);
    }

    #[test]
    fn test_remove_unknown_returns_false() {
        let mut wl = make_wl();
        assert!(!watch_remove(&mut wl, "nonexistent"));
    }

    #[test]
    fn test_mark_changed_and_query() {
        let mut wl = make_wl();
        watch_add(&mut wl, "scene.json");
        assert!(!watch_is_changed(&wl, "scene.json"));
        watch_mark_changed(&mut wl, "scene.json");
        assert!(watch_is_changed(&wl, "scene.json"));
    }

    #[test]
    fn test_changed_entries_list() {
        let mut wl = make_wl();
        watch_add(&mut wl, "a");
        watch_add(&mut wl, "b");
        watch_mark_changed(&mut wl, "b");
        let changed = watch_changed_entries(&wl);
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0], "b");
    }

    #[test]
    fn test_clear_changed() {
        let mut wl = make_wl();
        watch_add(&mut wl, "x");
        watch_mark_changed(&mut wl, "x");
        watch_clear_changed(&mut wl);
        assert!(!watch_is_changed(&wl, "x"));
    }

    #[test]
    fn test_max_entries_limit() {
        let cfg = WatchListConfig {
            max_entries: 2,
            deduplicate: true,
        };
        let mut wl = new_watch_list(cfg);
        watch_add(&mut wl, "a");
        watch_add(&mut wl, "b");
        let added = watch_add(&mut wl, "c");
        assert!(!added);
        assert_eq!(watch_count(&wl), 2);
    }

    #[test]
    fn test_to_json_format() {
        let mut wl = make_wl();
        watch_add(&mut wl, "model.gltf");
        let json = watch_list_to_json(&wl);
        assert!(json.contains("count"));
        assert!(json.contains("model.gltf"));
    }

    #[test]
    fn test_mark_unknown_path_returns_false() {
        let mut wl = make_wl();
        assert!(!watch_mark_changed(&mut wl, "not_watched"));
    }
}
