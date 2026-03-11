#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! File system watch stub.

use std::collections::HashSet;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct WatchEvent {
    pub path: String,
    pub kind: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FileWatcher {
    watched: HashSet<String>,
    events: Vec<WatchEvent>,
}

#[allow(dead_code)]
pub fn new_file_watcher() -> FileWatcher {
    FileWatcher {
        watched: HashSet::new(),
        events: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn watch_path(w: &mut FileWatcher, path: &str) {
    w.watched.insert(path.to_string());
}

#[allow(dead_code)]
pub fn unwatch_path(w: &mut FileWatcher, path: &str) -> bool {
    w.watched.remove(path)
}

#[allow(dead_code)]
pub fn poll_events(w: &mut FileWatcher) -> Vec<WatchEvent> {
    let events = w.events.clone();
    w.events.clear();
    events
}

#[allow(dead_code)]
pub fn watch_count(w: &FileWatcher) -> usize {
    w.watched.len()
}

#[allow(dead_code)]
pub fn event_count_fw(w: &FileWatcher) -> usize {
    w.events.len()
}

#[allow(dead_code)]
pub fn clear_events_fw(w: &mut FileWatcher) {
    w.events.clear();
}

#[allow(dead_code)]
pub fn watcher_to_json(w: &FileWatcher) -> String {
    format!(
        r#"{{"watched":{},"events":{}}}"#,
        w.watched.len(),
        w.events.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_watcher() {
        let w = new_file_watcher();
        assert_eq!(watch_count(&w), 0);
    }

    #[test]
    fn test_watch_path() {
        let mut w = new_file_watcher();
        watch_path(&mut w, "/tmp/a");
        assert_eq!(watch_count(&w), 1);
    }

    #[test]
    fn test_unwatch_path() {
        let mut w = new_file_watcher();
        watch_path(&mut w, "/tmp/a");
        assert!(unwatch_path(&mut w, "/tmp/a"));
        assert_eq!(watch_count(&w), 0);
    }

    #[test]
    fn test_unwatch_nonexistent() {
        let mut w = new_file_watcher();
        assert!(!unwatch_path(&mut w, "/tmp/nope"));
    }

    #[test]
    fn test_poll_events_empty() {
        let mut w = new_file_watcher();
        assert!(poll_events(&mut w).is_empty());
    }

    #[test]
    fn test_poll_events_with_data() {
        let mut w = new_file_watcher();
        w.events.push(WatchEvent {
            path: "/tmp/a".to_string(),
            kind: "modify".to_string(),
        });
        let evts = poll_events(&mut w);
        assert_eq!(evts.len(), 1);
        assert!(w.events.is_empty());
    }

    #[test]
    fn test_event_count() {
        let mut w = new_file_watcher();
        w.events.push(WatchEvent {
            path: "/tmp/b".to_string(),
            kind: "create".to_string(),
        });
        assert_eq!(event_count_fw(&w), 1);
    }

    #[test]
    fn test_clear_events() {
        let mut w = new_file_watcher();
        w.events.push(WatchEvent {
            path: "/tmp/c".to_string(),
            kind: "delete".to_string(),
        });
        clear_events_fw(&mut w);
        assert_eq!(event_count_fw(&w), 0);
    }

    #[test]
    fn test_watcher_to_json() {
        let w = new_file_watcher();
        let json = watcher_to_json(&w);
        assert!(json.contains("\"watched\":0"));
    }

    #[test]
    fn test_duplicate_watch() {
        let mut w = new_file_watcher();
        watch_path(&mut w, "/tmp/a");
        watch_path(&mut w, "/tmp/a");
        assert_eq!(watch_count(&w), 1);
    }
}
