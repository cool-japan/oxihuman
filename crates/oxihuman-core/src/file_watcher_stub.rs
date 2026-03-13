// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! File system watcher with debouncing, glob filtering, event batching,
//! recursive directory watching, and dynamic watch management.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::{Duration, Instant};

/// Errors that can occur during file-watch operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchError {
    EmptyPath,
    InvalidPath(String),
    AlreadyWatched(String),
    NotWatched(String),
    InvalidGlob(String),
}

impl std::fmt::Display for WatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WatchError::EmptyPath => write!(f, "path must not be empty"),
            WatchError::InvalidPath(p) => write!(f, "invalid path: {p}"),
            WatchError::AlreadyWatched(p) => write!(f, "path already watched: {p}"),
            WatchError::NotWatched(p) => write!(f, "path not watched: {p}"),
            WatchError::InvalidGlob(g) => write!(f, "invalid glob pattern: {g}"),
        }
    }
}

impl std::error::Error for WatchError {}

/// A file system event type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsEvent {
    Created(String),
    Modified(String),
    Deleted(String),
    Renamed { from: String, to: String },
}

impl FsEvent {
    pub fn path(&self) -> &str {
        match self {
            FsEvent::Created(p) | FsEvent::Modified(p) | FsEvent::Deleted(p) => p,
            FsEvent::Renamed { from, .. } => from,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            FsEvent::Created(_) => "created",
            FsEvent::Modified(_) => "modified",
            FsEvent::Deleted(_) => "deleted",
            FsEvent::Renamed { .. } => "renamed",
        }
    }
}

/// A batch of events that occurred within the same debounce window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventBatch {
    events: Vec<FsEvent>,
}

impl EventBatch {
    pub fn new(events: Vec<FsEvent>) -> Self {
        Self { events }
    }

    pub fn events(&self) -> &[FsEvent] {
        &self.events
    }

    pub fn into_events(self) -> Vec<FsEvent> {
        self.events
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn unique_paths(&self) -> HashSet<&str> {
        self.events.iter().map(|e| e.path()).collect()
    }
}

/// A compiled glob pattern used for filtering watched paths.
#[derive(Debug, Clone)]
pub struct GlobPattern {
    raw: String,
    segments: Vec<GlobSegment>,
}

#[derive(Debug, Clone)]
enum GlobSegment {
    Star,
    DoubleStar,
    Question,
    Literal(String),
}

impl GlobPattern {
    pub fn new(pattern: &str) -> Result<Self, WatchError> {
        if pattern.is_empty() {
            return Err(WatchError::InvalidGlob("empty pattern".to_string()));
        }
        let segments = Self::parse(pattern);
        Ok(Self {
            raw: pattern.to_string(),
            segments,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }

    fn parse(pattern: &str) -> Vec<GlobSegment> {
        let mut segs = Vec::new();
        let mut literal = String::new();
        let chars: Vec<char> = pattern.chars().collect();
        let len = chars.len();
        let mut i = 0;
        while i < len {
            match chars[i] {
                '*' => {
                    if !literal.is_empty() {
                        segs.push(GlobSegment::Literal(std::mem::take(&mut literal)));
                    }
                    if i + 1 < len && chars[i + 1] == '*' {
                        segs.push(GlobSegment::DoubleStar);
                        i += 2;
                        if i < len && (chars[i] == '/' || chars[i] == '\\') {
                            i += 1;
                        }
                    } else {
                        segs.push(GlobSegment::Star);
                        i += 1;
                    }
                }
                '?' => {
                    if !literal.is_empty() {
                        segs.push(GlobSegment::Literal(std::mem::take(&mut literal)));
                    }
                    segs.push(GlobSegment::Question);
                    i += 1;
                }
                c => {
                    literal.push(c);
                    i += 1;
                }
            }
        }
        if !literal.is_empty() {
            segs.push(GlobSegment::Literal(literal));
        }
        segs
    }

    pub fn matches(&self, text: &str) -> bool {
        Self::match_segments(&self.segments, text)
    }

    fn match_segments(segments: &[GlobSegment], text: &str) -> bool {
        if segments.is_empty() {
            return text.is_empty();
        }
        match &segments[0] {
            GlobSegment::Literal(lit) => {
                if let Some(rest) = text.strip_prefix(lit.as_str()) {
                    Self::match_segments(&segments[1..], rest)
                } else {
                    false
                }
            }
            GlobSegment::Question => {
                let mut chars = text.chars();
                match chars.next() {
                    Some(c) if c != '/' && c != '\\' => {
                        Self::match_segments(&segments[1..], chars.as_str())
                    }
                    _ => false,
                }
            }
            GlobSegment::Star => {
                let rest_segments = &segments[1..];
                if Self::match_segments(rest_segments, text) {
                    return true;
                }
                for (i, c) in text.char_indices() {
                    if c == '/' || c == '\\' {
                        break;
                    }
                    let after = &text[i + c.len_utf8()..];
                    if Self::match_segments(rest_segments, after) {
                        return true;
                    }
                }
                false
            }
            GlobSegment::DoubleStar => {
                let rest_segments = &segments[1..];
                if Self::match_segments(rest_segments, text) {
                    return true;
                }
                for (i, c) in text.char_indices() {
                    let after = &text[i + c.len_utf8()..];
                    if Self::match_segments(rest_segments, after) {
                        return true;
                    }
                }
                false
            }
        }
    }
}

/// Configuration for a single watched path.
#[derive(Debug, Clone)]
pub struct WatchEntry {
    pub path: String,
    pub recursive: bool,
}

/// Configuration for the debouncing behaviour.
#[derive(Debug, Clone)]
pub struct DebounceConfig {
    pub window: Duration,
}

impl Default for DebounceConfig {
    fn default() -> Self {
        Self {
            window: Duration::from_millis(100),
        }
    }
}

/// File watcher with debouncing, glob filtering, event batching,
/// recursive directory watching, and dynamic watch management.
pub struct FileWatcherStub {
    watched_paths: Vec<String>,
    watch_entries: HashMap<String, WatchEntry>,
    events: Vec<FsEvent>,
    glob_filters: Vec<GlobPattern>,
    debounce: DebounceConfig,
    last_drain: Option<Instant>,
    pending_debounce: Vec<(FsEvent, Instant)>,
    batching_enabled: bool,
    batches: Vec<EventBatch>,
    #[allow(clippy::type_complexity)]
    batch_callback: Option<Box<dyn Fn(&EventBatch) + Send + Sync>>,
}

impl std::fmt::Debug for FileWatcherStub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileWatcherStub")
            .field("watched_paths", &self.watched_paths)
            .field("events", &self.events)
            .field("batching_enabled", &self.batching_enabled)
            .finish()
    }
}

impl FileWatcherStub {
    pub fn new() -> Self {
        FileWatcherStub {
            watched_paths: Vec::new(),
            watch_entries: HashMap::new(),
            events: Vec::new(),
            glob_filters: Vec::new(),
            debounce: DebounceConfig::default(),
            last_drain: None,
            pending_debounce: Vec::new(),
            batching_enabled: false,
            batches: Vec::new(),
            batch_callback: None,
        }
    }

    pub fn with_debounce(window: Duration) -> Self {
        let mut w = Self::new();
        w.debounce = DebounceConfig { window };
        w
    }

    pub fn watch(&mut self, path: &str) {
        if !self.watched_paths.contains(&path.to_string()) {
            self.watched_paths.push(path.to_string());
            self.watch_entries.insert(
                path.to_string(),
                WatchEntry {
                    path: path.to_string(),
                    recursive: false,
                },
            );
        }
    }

    pub fn watch_recursive(&mut self, path: &str, recursive: bool) -> Result<(), WatchError> {
        Self::validate_path(path)?;
        if self.watched_paths.contains(&path.to_string()) {
            return Err(WatchError::AlreadyWatched(path.to_string()));
        }
        self.watched_paths.push(path.to_string());
        self.watch_entries.insert(
            path.to_string(),
            WatchEntry {
                path: path.to_string(),
                recursive,
            },
        );
        Ok(())
    }

    pub fn unwatch(&mut self, path: &str) {
        self.watched_paths.retain(|p| p != path);
        self.watch_entries.remove(path);
    }

    pub fn unwatch_checked(&mut self, path: &str) -> Result<(), WatchError> {
        if !self.watched_paths.contains(&path.to_string()) {
            return Err(WatchError::NotWatched(path.to_string()));
        }
        self.unwatch(path);
        Ok(())
    }

    pub fn update_recursive(&mut self, path: &str, recursive: bool) -> Result<(), WatchError> {
        match self.watch_entries.get_mut(path) {
            Some(entry) => {
                entry.recursive = recursive;
                Ok(())
            }
            None => Err(WatchError::NotWatched(path.to_string())),
        }
    }

    pub fn replace_watches(&mut self, paths: &[&str]) {
        self.watched_paths.clear();
        self.watch_entries.clear();
        for p in paths {
            self.watch(p);
        }
    }

    pub fn watch_entry(&self, path: &str) -> Option<&WatchEntry> {
        self.watch_entries.get(path)
    }

    pub fn watched_count(&self) -> usize {
        self.watched_paths.len()
    }

    pub fn watched_paths(&self) -> &[String] {
        &self.watched_paths
    }

    pub fn add_glob_filter(&mut self, pattern: &str) -> Result<(), WatchError> {
        let g = GlobPattern::new(pattern)?;
        self.glob_filters.push(g);
        Ok(())
    }

    pub fn clear_glob_filters(&mut self) {
        self.glob_filters.clear();
    }

    pub fn glob_filter_count(&self) -> usize {
        self.glob_filters.len()
    }

    pub fn passes_glob_filter(&self, path: &str) -> bool {
        if self.glob_filters.is_empty() {
            return true;
        }
        self.glob_filters.iter().any(|g| g.matches(path))
    }

    pub fn set_debounce_window(&mut self, window: Duration) {
        self.debounce.window = window;
    }

    pub fn debounce_window(&self) -> Duration {
        self.debounce.window
    }

    pub fn inject_event(&mut self, event: FsEvent) {
        if !self.passes_glob_filter(event.path()) {
            return;
        }
        self.events.push(event);
    }

    pub fn inject_event_unfiltered(&mut self, event: FsEvent) {
        self.events.push(event);
    }

    pub fn inject_event_timed(&mut self, event: FsEvent, when: Instant) {
        if !self.passes_glob_filter(event.path()) {
            return;
        }
        self.pending_debounce.push((event, when));
    }

    pub fn drain_events(&mut self) -> Vec<FsEvent> {
        self.last_drain = Some(Instant::now());
        std::mem::take(&mut self.events)
    }

    pub fn drain_events_debounced(&mut self) -> Vec<FsEvent> {
        self.last_drain = Some(Instant::now());
        if !self.pending_debounce.is_empty() {
            // Coalesce non-timed events first, then append timed results
            // (which are already per-window coalesced and must not be
            // re-merged across windows).
            let raw = std::mem::take(&mut self.events);
            let mut result = Self::coalesce_events(raw);
            let timed = std::mem::take(&mut self.pending_debounce);
            let timed_events = Self::coalesce_timed_into(timed, self.debounce.window);
            result.extend(timed_events);
            return result;
        }
        let raw = std::mem::take(&mut self.events);
        Self::coalesce_events(raw)
    }

    fn coalesce_timed_into(mut timed: Vec<(FsEvent, Instant)>, window: Duration) -> Vec<FsEvent> {
        if timed.is_empty() {
            return Vec::new();
        }
        timed.sort_by_key(|(_, t)| *t);
        let mut groups: Vec<Vec<(FsEvent, Instant)>> = Vec::new();
        let mut current_group: Vec<(FsEvent, Instant)> = Vec::new();
        let mut group_start: Option<Instant> = None;
        for item in timed {
            let start = match group_start {
                Some(s) => s,
                None => {
                    group_start = Some(item.1);
                    current_group.push(item);
                    continue;
                }
            };
            if item.1.duration_since(start) <= window {
                current_group.push(item);
            } else {
                groups.push(std::mem::take(&mut current_group));
                group_start = Some(item.1);
                current_group.push(item);
            }
        }
        if !current_group.is_empty() {
            groups.push(current_group);
        }
        let mut result = Vec::new();
        for group in groups {
            let mut latest: HashMap<String, FsEvent> = HashMap::new();
            for (ev, _) in group {
                latest.insert(ev.path().to_string(), ev);
            }
            result.extend(latest.into_values());
        }
        result
    }

    fn coalesce_events(events: Vec<FsEvent>) -> Vec<FsEvent> {
        let mut seen: HashMap<String, usize> = HashMap::new();
        let mut result: Vec<FsEvent> = Vec::new();
        for ev in events {
            let key = ev.path().to_string();
            if let Some(&idx) = seen.get(&key) {
                result[idx] = ev;
            } else {
                seen.insert(key, result.len());
                result.push(ev);
            }
        }
        result
    }

    pub fn set_batching(&mut self, enabled: bool) {
        self.batching_enabled = enabled;
    }

    pub fn batching_enabled(&self) -> bool {
        self.batching_enabled
    }

    pub fn set_batch_callback<F>(&mut self, cb: F)
    where
        F: Fn(&EventBatch) + Send + Sync + 'static,
    {
        self.batch_callback = Some(Box::new(cb));
    }

    pub fn clear_batch_callback(&mut self) {
        self.batch_callback = None;
    }

    pub fn flush_batches(&mut self) -> Vec<EventBatch> {
        let events = self.drain_events();
        if events.is_empty() {
            return Vec::new();
        }
        let batch = EventBatch::new(events);
        if let Some(cb) = &self.batch_callback {
            cb(&batch);
        }
        self.batches.push(batch.clone());
        vec![batch]
    }

    pub fn flush_batches_debounced(&mut self) -> Vec<EventBatch> {
        let events = self.drain_events_debounced();
        if events.is_empty() {
            return Vec::new();
        }
        let batch = EventBatch::new(events);
        if let Some(cb) = &self.batch_callback {
            cb(&batch);
        }
        self.batches.push(batch.clone());
        vec![batch]
    }

    pub fn batches(&self) -> &[EventBatch] {
        &self.batches
    }

    pub fn clear_batches(&mut self) {
        self.batches.clear();
    }

    fn validate_path(path: &str) -> Result<(), WatchError> {
        if path.is_empty() {
            return Err(WatchError::EmptyPath);
        }
        if path.contains('\0') {
            return Err(WatchError::InvalidPath(path.to_string()));
        }
        Ok(())
    }

    pub fn check_path(path: &str) -> Result<(), WatchError> {
        Self::validate_path(path)
    }

    pub fn path_exists(path: &str) -> bool {
        Path::new(path).exists()
    }
}

impl Default for FileWatcherStub {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new file watcher.
pub fn new_file_watcher() -> FileWatcherStub {
    FileWatcherStub::new()
}

/// Watch multiple paths at once.
pub fn watch_paths(watcher: &mut FileWatcherStub, paths: &[&str]) {
    for p in paths {
        watcher.watch(p);
    }
}

/// Drain and count events.
pub fn drain_and_count(watcher: &mut FileWatcherStub) -> (Vec<FsEvent>, usize) {
    let events = watcher.drain_events();
    let n = events.len();
    (events, n)
}

/// Check if a path is currently watched.
pub fn is_watched(watcher: &FileWatcherStub, path: &str) -> bool {
    watcher.watched_paths.contains(&path.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_watcher_empty() {
        let w = FileWatcherStub::new();
        assert_eq!(w.watched_count(), 0);
    }

    #[test]
    fn test_watch_path() {
        let mut w = new_file_watcher();
        w.watch("/tmp/foo");
        assert_eq!(w.watched_count(), 1);
        assert!(is_watched(&w, "/tmp/foo"));
    }

    #[test]
    fn test_unwatch_path() {
        let mut w = new_file_watcher();
        w.watch("/tmp/bar");
        w.unwatch("/tmp/bar");
        assert_eq!(w.watched_count(), 0);
    }

    #[test]
    fn test_inject_and_drain_events() {
        let mut w = new_file_watcher();
        w.inject_event(FsEvent::Created("/tmp/x".to_string()));
        let evs = w.drain_events();
        assert_eq!(evs.len(), 1);
        assert!(matches!(&evs[0], FsEvent::Created(p) if p == "/tmp/x"));
    }

    #[test]
    fn test_drain_clears_events() {
        let mut w = new_file_watcher();
        w.inject_event(FsEvent::Deleted("/tmp/y".to_string()));
        let _ = w.drain_events();
        assert!(w.drain_events().is_empty());
    }

    #[test]
    fn test_watch_multiple() {
        let mut w = new_file_watcher();
        watch_paths(&mut w, &["/a", "/b", "/c"]);
        assert_eq!(w.watched_count(), 3);
    }

    #[test]
    fn test_drain_and_count() {
        let mut w = new_file_watcher();
        w.inject_event(FsEvent::Modified("/tmp/z".to_string()));
        w.inject_event(FsEvent::Modified("/tmp/z".to_string()));
        let (_, n) = drain_and_count(&mut w);
        assert_eq!(n, 2);
    }

    #[test]
    fn test_rename_event() {
        let mut w = new_file_watcher();
        w.inject_event(FsEvent::Renamed {
            from: "/a".to_string(),
            to: "/b".to_string(),
        });
        let evs = w.drain_events();
        assert!(matches!(&evs[0], FsEvent::Renamed { .. }));
    }

    #[test]
    fn test_no_duplicate_watch() {
        let mut w = new_file_watcher();
        w.watch("/same");
        w.watch("/same");
        assert_eq!(w.watched_count(), 1);
    }

    #[test]
    fn test_is_not_watched() {
        let w = new_file_watcher();
        assert!(!is_watched(&w, "/nonexistent"));
    }

    #[test]
    fn test_glob_pattern_star() {
        let g = GlobPattern::new("*.rs").expect("should succeed");
        assert!(g.matches("main.rs"));
        assert!(g.matches("lib.rs"));
        assert!(!g.matches("main.py"));
        assert!(!g.matches("src/main.rs"));
    }

    #[test]
    fn test_glob_pattern_double_star() {
        let g = GlobPattern::new("**/*.rs").expect("should succeed");
        assert!(g.matches("src/main.rs"));
        assert!(g.matches("a/b/c/lib.rs"));
        assert!(!g.matches("main.py"));
    }

    #[test]
    fn test_glob_pattern_question() {
        let g = GlobPattern::new("?.rs").expect("should succeed");
        assert!(g.matches("a.rs"));
        assert!(!g.matches("ab.rs"));
    }

    #[test]
    fn test_glob_filter_on_events() {
        let mut w = new_file_watcher();
        w.add_glob_filter("*.rs").expect("should succeed");
        w.inject_event(FsEvent::Modified("main.rs".to_string()));
        w.inject_event(FsEvent::Modified("main.py".to_string()));
        let evs = w.drain_events();
        assert_eq!(evs.len(), 1);
        assert!(matches!(&evs[0], FsEvent::Modified(p) if p == "main.rs"));
    }

    #[test]
    fn test_glob_filter_cleared() {
        let mut w = new_file_watcher();
        w.add_glob_filter("*.rs").expect("should succeed");
        w.clear_glob_filters();
        w.inject_event(FsEvent::Modified("main.py".to_string()));
        assert_eq!(w.drain_events().len(), 1);
    }

    #[test]
    fn test_invalid_glob() {
        let r = GlobPattern::new("");
        assert!(r.is_err());
    }

    #[test]
    fn test_watch_recursive() {
        let mut w = new_file_watcher();
        w.watch_recursive("/src", true).expect("should succeed");
        let entry = w.watch_entry("/src").expect("should succeed");
        assert!(entry.recursive);
    }

    #[test]
    fn test_watch_recursive_duplicate() {
        let mut w = new_file_watcher();
        w.watch_recursive("/src", true).expect("should succeed");
        let r = w.watch_recursive("/src", false);
        assert!(matches!(r, Err(WatchError::AlreadyWatched(_))));
    }

    #[test]
    fn test_update_recursive() {
        let mut w = new_file_watcher();
        w.watch_recursive("/src", false).expect("should succeed");
        w.update_recursive("/src", true).expect("should succeed");
        assert!(w.watch_entry("/src").expect("should succeed").recursive);
    }

    #[test]
    fn test_update_recursive_not_watched() {
        let mut w = new_file_watcher();
        let r = w.update_recursive("/nope", true);
        assert!(matches!(r, Err(WatchError::NotWatched(_))));
    }

    #[test]
    fn test_unwatch_checked_ok() {
        let mut w = new_file_watcher();
        w.watch("/a");
        assert!(w.unwatch_checked("/a").is_ok());
        assert_eq!(w.watched_count(), 0);
    }

    #[test]
    fn test_unwatch_checked_err() {
        let mut w = new_file_watcher();
        assert!(matches!(
            w.unwatch_checked("/nope"),
            Err(WatchError::NotWatched(_))
        ));
    }

    #[test]
    fn test_replace_watches() {
        let mut w = new_file_watcher();
        w.watch("/old");
        w.replace_watches(&["/new1", "/new2"]);
        assert_eq!(w.watched_count(), 2);
        assert!(!is_watched(&w, "/old"));
        assert!(is_watched(&w, "/new1"));
    }

    #[test]
    fn test_validate_empty_path() {
        assert!(matches!(
            FileWatcherStub::check_path(""),
            Err(WatchError::EmptyPath)
        ));
    }

    #[test]
    fn test_validate_null_byte_path() {
        assert!(matches!(
            FileWatcherStub::check_path("/foo\0bar"),
            Err(WatchError::InvalidPath(_))
        ));
    }

    #[test]
    fn test_debounce_coalescing() {
        let mut w = FileWatcherStub::with_debounce(Duration::from_millis(200));
        let now = Instant::now();
        w.inject_event_timed(FsEvent::Modified("/a".to_string()), now);
        w.inject_event_timed(
            FsEvent::Modified("/a".to_string()),
            now + Duration::from_millis(50),
        );
        let evs = w.drain_events_debounced();
        assert_eq!(evs.len(), 1);
    }

    #[test]
    fn test_debounce_separate_windows() {
        let mut w = FileWatcherStub::with_debounce(Duration::from_millis(100));
        let now = Instant::now();
        w.inject_event_timed(FsEvent::Modified("/a".to_string()), now);
        w.inject_event_timed(
            FsEvent::Modified("/a".to_string()),
            now + Duration::from_millis(200),
        );
        let evs = w.drain_events_debounced();
        assert_eq!(evs.len(), 2);
    }

    #[test]
    fn test_simple_coalesce() {
        let mut w = new_file_watcher();
        w.inject_event(FsEvent::Modified("/a".to_string()));
        w.inject_event(FsEvent::Created("/a".to_string()));
        let evs = w.drain_events_debounced();
        assert_eq!(evs.len(), 1);
        assert!(matches!(&evs[0], FsEvent::Created(_)));
    }

    #[test]
    fn test_batching_flush() {
        let mut w = new_file_watcher();
        w.set_batching(true);
        w.inject_event(FsEvent::Created("/x".to_string()));
        w.inject_event(FsEvent::Modified("/y".to_string()));
        let batches = w.flush_batches();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 2);
    }

    #[test]
    fn test_batching_callback() {
        use std::sync::{Arc, Mutex};
        let seen = Arc::new(Mutex::new(Vec::new()));
        let seen2 = Arc::clone(&seen);
        let mut w = new_file_watcher();
        w.set_batch_callback(move |batch| {
            if let Ok(mut v) = seen2.lock() {
                v.push(batch.len());
            }
        });
        w.inject_event(FsEvent::Created("/a".to_string()));
        let _ = w.flush_batches();
        let locked = seen.lock().expect("should succeed");
        assert_eq!(locked.len(), 1);
        assert_eq!(locked[0], 1);
    }

    #[test]
    fn test_batch_unique_paths() {
        let batch = EventBatch::new(vec![
            FsEvent::Modified("/a".to_string()),
            FsEvent::Modified("/a".to_string()),
            FsEvent::Created("/b".to_string()),
        ]);
        let paths = batch.unique_paths();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains("/a"));
        assert!(paths.contains("/b"));
    }

    #[test]
    fn test_inject_unfiltered_bypasses_glob() {
        let mut w = new_file_watcher();
        w.add_glob_filter("*.rs").expect("should succeed");
        w.inject_event_unfiltered(FsEvent::Modified("main.py".to_string()));
        assert_eq!(w.drain_events().len(), 1);
    }

    #[test]
    fn test_event_path_and_kind() {
        let e = FsEvent::Renamed {
            from: "/old".to_string(),
            to: "/new".to_string(),
        };
        assert_eq!(e.path(), "/old");
        assert_eq!(e.kind(), "renamed");
    }

    #[test]
    fn test_flush_batches_debounced() {
        let mut w = new_file_watcher();
        w.inject_event(FsEvent::Modified("/a".to_string()));
        w.inject_event(FsEvent::Modified("/a".to_string()));
        let batches = w.flush_batches_debounced();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 1);
    }

    #[test]
    fn test_watched_paths_snapshot() {
        let mut w = new_file_watcher();
        w.watch("/x");
        w.watch("/y");
        let paths = w.watched_paths();
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_default_impl() {
        let w = FileWatcherStub::default();
        assert_eq!(w.watched_count(), 0);
    }

    #[test]
    fn test_debounce_window_setter() {
        let mut w = new_file_watcher();
        w.set_debounce_window(Duration::from_secs(1));
        assert_eq!(w.debounce_window(), Duration::from_secs(1));
    }

    #[test]
    fn test_clear_batch_callback() {
        let mut w = new_file_watcher();
        w.set_batch_callback(|_| {});
        w.clear_batch_callback();
        w.inject_event(FsEvent::Created("/a".to_string()));
        let _ = w.flush_batches();
    }

    #[test]
    fn test_clear_batches() {
        let mut w = new_file_watcher();
        w.inject_event(FsEvent::Created("/a".to_string()));
        let _ = w.flush_batches();
        assert_eq!(w.batches().len(), 1);
        w.clear_batches();
        assert!(w.batches().is_empty());
    }

    #[test]
    fn test_watch_error_display() {
        let e = WatchError::EmptyPath;
        assert_eq!(format!("{e}"), "path must not be empty");
        let e2 = WatchError::InvalidGlob("bad".to_string());
        assert!(format!("{e2}").contains("bad"));
    }

    #[test]
    fn test_multiple_glob_filters() {
        let mut w = new_file_watcher();
        w.add_glob_filter("*.rs").expect("should succeed");
        w.add_glob_filter("*.toml").expect("should succeed");
        assert_eq!(w.glob_filter_count(), 2);
        assert!(w.passes_glob_filter("lib.rs"));
        assert!(w.passes_glob_filter("Cargo.toml"));
        assert!(!w.passes_glob_filter("main.py"));
    }

    #[test]
    fn test_event_batch_into_events() {
        let batch = EventBatch::new(vec![FsEvent::Created("/a".to_string())]);
        let evs = batch.into_events();
        assert_eq!(evs.len(), 1);
    }

    #[test]
    fn test_empty_batch() {
        let batch = EventBatch::new(vec![]);
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn test_glob_no_filters_passes_everything() {
        let w = new_file_watcher();
        assert!(w.passes_glob_filter("anything.txt"));
    }
}
