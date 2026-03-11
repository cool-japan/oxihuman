// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Config file change watcher stub (no real IO).

#![allow(dead_code)]

use std::collections::HashMap;

/// Configuration for the watcher.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WatchConfig {
    pub poll_interval_ms: u64,
    pub max_keys: usize,
}

/// A watched key entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WatchEntry {
    pub key: String,
    pub last_value: String,
    pub dirty: bool,
}

/// Config watcher that tracks dirty keys.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConfigWatcher {
    config: WatchConfig,
    entries: HashMap<String, WatchEntry>,
}

/// Create default watch config.
#[allow(dead_code)]
pub fn default_watch_config() -> WatchConfig {
    WatchConfig {
        poll_interval_ms: 500,
        max_keys: 256,
    }
}

/// Create a new config watcher.
#[allow(dead_code)]
pub fn new_config_watcher(config: WatchConfig) -> ConfigWatcher {
    ConfigWatcher {
        config,
        entries: HashMap::new(),
    }
}

/// Add a key to watch with an initial value.
#[allow(dead_code)]
pub fn watcher_add_key(watcher: &mut ConfigWatcher, key: &str, initial_value: &str) -> bool {
    if watcher.entries.len() >= watcher.config.max_keys {
        return false;
    }
    watcher.entries.insert(key.to_string(), WatchEntry {
        key: key.to_string(),
        last_value: initial_value.to_string(),
        dirty: false,
    });
    true
}

/// Mark a key as dirty (value changed).
#[allow(dead_code)]
pub fn watcher_mark_dirty(watcher: &mut ConfigWatcher, key: &str, new_value: &str) -> bool {
    if let Some(entry) = watcher.entries.get_mut(key) {
        entry.last_value = new_value.to_string();
        entry.dirty = true;
        true
    } else {
        false
    }
}

/// Return a list of dirty keys.
#[allow(dead_code)]
pub fn watcher_dirty_keys(watcher: &ConfigWatcher) -> Vec<String> {
    watcher.entries.values()
        .filter(|e| e.dirty)
        .map(|e| e.key.clone())
        .collect()
}

/// Clear all dirty flags.
#[allow(dead_code)]
pub fn watcher_clear_dirty(watcher: &mut ConfigWatcher) {
    for entry in watcher.entries.values_mut() {
        entry.dirty = false;
    }
}

/// Return the number of watched keys.
#[allow(dead_code)]
pub fn watcher_key_count(watcher: &ConfigWatcher) -> usize {
    watcher.entries.len()
}

/// Remove a watched key.
#[allow(dead_code)]
pub fn watcher_remove_key(watcher: &mut ConfigWatcher, key: &str) -> bool {
    watcher.entries.remove(key).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_watch_config();
        assert!(cfg.poll_interval_ms > 0);
        assert!(cfg.max_keys > 0);
    }

    #[test]
    fn test_new_watcher_empty() {
        let cfg = default_watch_config();
        let watcher = new_config_watcher(cfg);
        assert_eq!(watcher_key_count(&watcher), 0);
    }

    #[test]
    fn test_add_key() {
        let mut watcher = new_config_watcher(default_watch_config());
        let added = watcher_add_key(&mut watcher, "db.url", "localhost");
        assert!(added);
        assert_eq!(watcher_key_count(&watcher), 1);
    }

    #[test]
    fn test_mark_and_check_dirty() {
        let mut watcher = new_config_watcher(default_watch_config());
        watcher_add_key(&mut watcher, "key1", "v1");
        watcher_mark_dirty(&mut watcher, "key1", "v2");
        let dirty = watcher_dirty_keys(&watcher);
        assert!(dirty.contains(&"key1".to_string()));
    }

    #[test]
    fn test_clear_dirty() {
        let mut watcher = new_config_watcher(default_watch_config());
        watcher_add_key(&mut watcher, "k", "val");
        watcher_mark_dirty(&mut watcher, "k", "val2");
        watcher_clear_dirty(&mut watcher);
        assert!(watcher_dirty_keys(&watcher).is_empty());
    }

    #[test]
    fn test_remove_key() {
        let mut watcher = new_config_watcher(default_watch_config());
        watcher_add_key(&mut watcher, "rem", "x");
        let removed = watcher_remove_key(&mut watcher, "rem");
        assert!(removed);
        assert_eq!(watcher_key_count(&watcher), 0);
    }

    #[test]
    fn test_mark_dirty_unknown_key() {
        let mut watcher = new_config_watcher(default_watch_config());
        let result = watcher_mark_dirty(&mut watcher, "nope", "val");
        assert!(!result);
    }

    #[test]
    fn test_max_keys_limit() {
        let cfg = WatchConfig { poll_interval_ms: 100, max_keys: 2 };
        let mut watcher = new_config_watcher(cfg);
        watcher_add_key(&mut watcher, "k1", "v1");
        watcher_add_key(&mut watcher, "k2", "v2");
        let added = watcher_add_key(&mut watcher, "k3", "v3");
        assert!(!added);
    }

    #[test]
    fn test_dirty_keys_only_dirty() {
        let mut watcher = new_config_watcher(default_watch_config());
        watcher_add_key(&mut watcher, "a", "1");
        watcher_add_key(&mut watcher, "b", "2");
        watcher_mark_dirty(&mut watcher, "a", "new");
        let dirty = watcher_dirty_keys(&watcher);
        assert_eq!(dirty.len(), 1);
        assert!(dirty.contains(&"a".to_string()));
    }
}
