// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! ShaderHotReload — watch shader files and signal when they change.

#![allow(dead_code)]

/// Signals that a watched shader has changed.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ReloadEvent {
    pub path: String,
    pub timestamp: u64,
}

/// Watches a set of shader file paths for changes.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ShaderReloadWatcher {
    pub watched: Vec<(String, u64)>,
    pub events: Vec<ReloadEvent>,
}

/// Create a new `ShaderReloadWatcher`.
#[allow(dead_code)]
pub fn new_shader_reload_watcher() -> ShaderReloadWatcher {
    ShaderReloadWatcher::default()
}

/// Add a shader file path to the watch list (with initial timestamp 0).
#[allow(dead_code)]
pub fn watch_shader_file(watcher: &mut ShaderReloadWatcher, path: &str) {
    watcher.watched.push((path.to_owned(), 0));
}

/// Check for changes (stub: compares current timestamp against stored).
/// If `new_timestamp > stored`, records a reload event.
#[allow(dead_code)]
pub fn check_for_changes(watcher: &mut ShaderReloadWatcher, new_timestamp: u64) {
    for (path, ts) in &mut watcher.watched {
        if new_timestamp > *ts {
            watcher.events.push(ReloadEvent { path: path.clone(), timestamp: new_timestamp });
            *ts = new_timestamp;
        }
    }
}

/// Return the number of pending reload events.
#[allow(dead_code)]
pub fn reload_event_count(watcher: &ShaderReloadWatcher) -> usize {
    watcher.events.len()
}

/// Return the file path of watched shader at `index`.
#[allow(dead_code)]
pub fn shader_file_path(watcher: &ShaderReloadWatcher, index: usize) -> Option<&str> {
    watcher.watched.get(index).map(|(p, _)| p.as_str())
}

/// Return the total number of watched files.
#[allow(dead_code)]
pub fn watcher_file_count(watcher: &ShaderReloadWatcher) -> usize {
    watcher.watched.len()
}

/// Mark the most recent event as handled (pop it).
#[allow(dead_code)]
pub fn mark_reloaded(watcher: &mut ShaderReloadWatcher) {
    watcher.events.pop();
}

/// Return whether any shader needs reloading (i.e., events pending).
#[allow(dead_code)]
pub fn shader_needs_reload(watcher: &ShaderReloadWatcher) -> bool {
    !watcher.events.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_watcher_empty() {
        let w = new_shader_reload_watcher();
        assert_eq!(watcher_file_count(&w), 0);
        assert_eq!(reload_event_count(&w), 0);
    }

    #[test]
    fn test_watch_shader_file() {
        let mut w = new_shader_reload_watcher();
        watch_shader_file(&mut w, "shaders/pbr.wgsl");
        assert_eq!(watcher_file_count(&w), 1);
    }

    #[test]
    fn test_shader_file_path() {
        let mut w = new_shader_reload_watcher();
        watch_shader_file(&mut w, "test.glsl");
        assert_eq!(shader_file_path(&w, 0), Some("test.glsl"));
        assert!(shader_file_path(&w, 1).is_none());
    }

    #[test]
    fn test_check_for_changes_triggers_event() {
        let mut w = new_shader_reload_watcher();
        watch_shader_file(&mut w, "a.wgsl");
        check_for_changes(&mut w, 100);
        assert_eq!(reload_event_count(&w), 1);
        assert!(shader_needs_reload(&w));
    }

    #[test]
    fn test_no_change_on_same_timestamp() {
        let mut w = new_shader_reload_watcher();
        watch_shader_file(&mut w, "b.wgsl");
        check_for_changes(&mut w, 50);
        check_for_changes(&mut w, 50);
        // Only one event because ts is updated on first call
        assert_eq!(reload_event_count(&w), 1);
    }

    #[test]
    fn test_mark_reloaded_pops_event() {
        let mut w = new_shader_reload_watcher();
        watch_shader_file(&mut w, "c.wgsl");
        check_for_changes(&mut w, 1);
        mark_reloaded(&mut w);
        assert!(!shader_needs_reload(&w));
    }

    #[test]
    fn test_shader_needs_reload_false() {
        let w = new_shader_reload_watcher();
        assert!(!shader_needs_reload(&w));
    }

    #[test]
    fn test_multiple_files() {
        let mut w = new_shader_reload_watcher();
        watch_shader_file(&mut w, "x.wgsl");
        watch_shader_file(&mut w, "y.wgsl");
        check_for_changes(&mut w, 99);
        assert_eq!(reload_event_count(&w), 2);
    }
}
