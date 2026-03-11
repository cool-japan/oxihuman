//! File watching / hot-reload detection for assets.
//! Provides a simulated file watcher for testing and integration with real watchers.

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum ChangeKind {
    Modified,
    Created,
    Deleted,
    Renamed,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct FileChange {
    pub path: String,
    pub kind: ChangeKind,
    pub timestamp_ms: u64,
}

#[allow(dead_code)]
pub struct HotReloadConfig {
    pub poll_interval_ms: u64,
    pub debounce_ms: u64,
    pub watch_extensions: Vec<String>,
}

#[allow(dead_code)]
pub struct HotReloadWatcher {
    pub watched_paths: Vec<String>,
    pub changes: Vec<FileChange>,
    pub config: HotReloadConfig,
    pub enabled: bool,
    pub simulated_time_ms: u64,
}

#[allow(dead_code)]
pub fn default_hot_reload_config() -> HotReloadConfig {
    HotReloadConfig {
        poll_interval_ms: 500,
        debounce_ms: 100,
        watch_extensions: vec![
            "obj".to_string(),
            "mtl".to_string(),
            "glb".to_string(),
            "gltf".to_string(),
            "png".to_string(),
            "json".to_string(),
        ],
    }
}

#[allow(dead_code)]
pub fn new_watcher(cfg: HotReloadConfig) -> HotReloadWatcher {
    HotReloadWatcher {
        watched_paths: Vec::new(),
        changes: Vec::new(),
        config: cfg,
        enabled: true,
        simulated_time_ms: 0,
    }
}

#[allow(dead_code)]
pub fn watch_path(watcher: &mut HotReloadWatcher, path: &str) {
    if !watcher.watched_paths.contains(&path.to_string()) {
        watcher.watched_paths.push(path.to_string());
    }
}

/// Returns true if the path was found and removed.
#[allow(dead_code)]
pub fn unwatch_path(watcher: &mut HotReloadWatcher, path: &str) -> bool {
    let before = watcher.watched_paths.len();
    watcher.watched_paths.retain(|p| p != path);
    watcher.watched_paths.len() < before
}

/// Simulate a file change event (for testing).
#[allow(dead_code)]
pub fn simulate_file_change(watcher: &mut HotReloadWatcher, path: &str, kind: ChangeKind) {
    if !watcher.enabled {
        return;
    }
    watcher.changes.push(FileChange {
        path: path.to_string(),
        kind,
        timestamp_ms: watcher.simulated_time_ms,
    });
}

#[allow(dead_code)]
pub fn pending_changes(watcher: &HotReloadWatcher) -> &[FileChange] {
    &watcher.changes
}

#[allow(dead_code)]
pub fn clear_changes(watcher: &mut HotReloadWatcher) {
    watcher.changes.clear();
}

#[allow(dead_code)]
pub fn is_watched(watcher: &HotReloadWatcher, path: &str) -> bool {
    watcher.watched_paths.contains(&path.to_string())
}

#[allow(dead_code)]
pub fn changes_for_path<'a>(watcher: &'a HotReloadWatcher, path: &str) -> Vec<&'a FileChange> {
    watcher.changes.iter().filter(|c| c.path == path).collect()
}

/// Check whether a file's extension is in the watch list.
#[allow(dead_code)]
pub fn extension_matches(watcher: &HotReloadWatcher, path: &str) -> bool {
    let ext = path.rsplit('.').next().unwrap_or("");
    watcher.config.watch_extensions.iter().any(|e| e == ext)
}

#[allow(dead_code)]
pub fn watched_path_count(watcher: &HotReloadWatcher) -> usize {
    watcher.watched_paths.len()
}

#[allow(dead_code)]
pub fn change_count(watcher: &HotReloadWatcher) -> usize {
    watcher.changes.len()
}

#[allow(dead_code)]
pub fn enable_watcher(watcher: &mut HotReloadWatcher) {
    watcher.enabled = true;
}

#[allow(dead_code)]
pub fn disable_watcher(watcher: &mut HotReloadWatcher) {
    watcher.enabled = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_watcher() -> HotReloadWatcher {
        new_watcher(default_hot_reload_config())
    }

    #[test]
    fn test_default_config() {
        let cfg = default_hot_reload_config();
        assert!(cfg.poll_interval_ms > 0);
        assert!(!cfg.watch_extensions.is_empty());
    }

    #[test]
    fn test_new_watcher() {
        let w = make_watcher();
        assert!(w.enabled);
        assert!(w.watched_paths.is_empty());
        assert!(w.changes.is_empty());
    }

    #[test]
    fn test_watch_path() {
        let mut w = make_watcher();
        watch_path(&mut w, "assets/model.glb");
        assert!(is_watched(&w, "assets/model.glb"));
        assert_eq!(watched_path_count(&w), 1);
    }

    #[test]
    fn test_watch_path_no_duplicates() {
        let mut w = make_watcher();
        watch_path(&mut w, "assets/model.glb");
        watch_path(&mut w, "assets/model.glb");
        assert_eq!(watched_path_count(&w), 1);
    }

    #[test]
    fn test_is_watched_false() {
        let w = make_watcher();
        assert!(!is_watched(&w, "missing.obj"));
    }

    #[test]
    fn test_unwatch_path() {
        let mut w = make_watcher();
        watch_path(&mut w, "assets/tex.png");
        let removed = unwatch_path(&mut w, "assets/tex.png");
        assert!(removed);
        assert!(!is_watched(&w, "assets/tex.png"));
    }

    #[test]
    fn test_unwatch_path_missing() {
        let mut w = make_watcher();
        let removed = unwatch_path(&mut w, "nonexistent.obj");
        assert!(!removed);
    }

    #[test]
    fn test_simulate_file_change() {
        let mut w = make_watcher();
        simulate_file_change(&mut w, "scene.json", ChangeKind::Modified);
        assert_eq!(change_count(&w), 1);
        assert_eq!(pending_changes(&w)[0].path, "scene.json");
        assert_eq!(pending_changes(&w)[0].kind, ChangeKind::Modified);
    }

    #[test]
    fn test_simulate_disabled_watcher() {
        let mut w = make_watcher();
        disable_watcher(&mut w);
        simulate_file_change(&mut w, "scene.json", ChangeKind::Modified);
        assert_eq!(change_count(&w), 0);
    }

    #[test]
    fn test_clear_changes() {
        let mut w = make_watcher();
        simulate_file_change(&mut w, "a.obj", ChangeKind::Created);
        simulate_file_change(&mut w, "b.obj", ChangeKind::Deleted);
        clear_changes(&mut w);
        assert_eq!(change_count(&w), 0);
    }

    #[test]
    fn test_changes_for_path() {
        let mut w = make_watcher();
        simulate_file_change(&mut w, "a.obj", ChangeKind::Modified);
        simulate_file_change(&mut w, "b.obj", ChangeKind::Created);
        simulate_file_change(&mut w, "a.obj", ChangeKind::Deleted);
        let changes = changes_for_path(&w, "a.obj");
        assert_eq!(changes.len(), 2);
    }

    #[test]
    fn test_extension_matches() {
        let w = make_watcher();
        assert!(extension_matches(&w, "model.glb"));
        assert!(extension_matches(&w, "texture.png"));
        assert!(!extension_matches(&w, "script.lua"));
        assert!(!extension_matches(&w, "data.bin"));
    }

    #[test]
    fn test_enable_disable_watcher() {
        let mut w = make_watcher();
        disable_watcher(&mut w);
        assert!(!w.enabled);
        enable_watcher(&mut w);
        assert!(w.enabled);
    }

    #[test]
    fn test_watched_path_count() {
        let mut w = make_watcher();
        assert_eq!(watched_path_count(&w), 0);
        watch_path(&mut w, "a.obj");
        watch_path(&mut w, "b.glb");
        assert_eq!(watched_path_count(&w), 2);
    }

    #[test]
    fn test_change_kind_eq() {
        assert_eq!(ChangeKind::Modified, ChangeKind::Modified);
        assert_ne!(ChangeKind::Created, ChangeKind::Deleted);
    }
}
