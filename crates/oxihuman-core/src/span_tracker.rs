// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Lightweight span tracker for timing named code regions.

use std::collections::HashMap;

/// A completed span record.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpanRecord {
    pub name: String,
    pub start_ns: u64,
    pub end_ns: u64,
    pub duration_ns: u64,
    pub tag: String,
}

/// Tracker that records start/end of named spans and accumulates stats.
#[allow(dead_code)]
pub struct SpanTracker {
    open: HashMap<String, u64>,
    completed: Vec<SpanRecord>,
    total_ns_by_name: HashMap<String, u64>,
    call_count_by_name: HashMap<String, u64>,
}

#[allow(dead_code)]
impl SpanTracker {
    pub fn new() -> Self {
        Self {
            open: HashMap::new(),
            completed: Vec::new(),
            total_ns_by_name: HashMap::new(),
            call_count_by_name: HashMap::new(),
        }
    }

    /// Open a span at `start_ns`.
    pub fn begin(&mut self, name: &str, start_ns: u64) {
        self.open.insert(name.to_string(), start_ns);
    }

    /// Close a span at `end_ns`; returns the duration or None if not open.
    pub fn end(&mut self, name: &str, end_ns: u64, tag: &str) -> Option<u64> {
        let start = self.open.remove(name)?;
        let dur = end_ns.saturating_sub(start);
        *self.total_ns_by_name.entry(name.to_string()).or_insert(0) += dur;
        *self.call_count_by_name.entry(name.to_string()).or_insert(0) += 1;
        self.completed.push(SpanRecord {
            name: name.to_string(),
            start_ns: start,
            end_ns,
            duration_ns: dur,
            tag: tag.to_string(),
        });
        Some(dur)
    }

    pub fn completed_count(&self) -> usize {
        self.completed.len()
    }

    pub fn open_count(&self) -> usize {
        self.open.len()
    }

    pub fn total_ns(&self, name: &str) -> u64 {
        self.total_ns_by_name.get(name).copied().unwrap_or(0)
    }

    pub fn call_count(&self, name: &str) -> u64 {
        self.call_count_by_name.get(name).copied().unwrap_or(0)
    }

    pub fn mean_ns(&self, name: &str) -> u64 {
        let count = self.call_count(name);
        if count == 0 {
            0
        } else {
            self.total_ns(name) / count
        }
    }

    pub fn last_completed(&self) -> Option<&SpanRecord> {
        self.completed.last()
    }

    pub fn spans_for_tag<'a>(&'a self, tag: &str) -> Vec<&'a SpanRecord> {
        self.completed.iter().filter(|s| s.tag == tag).collect()
    }

    pub fn clear(&mut self) {
        self.open.clear();
        self.completed.clear();
        self.total_ns_by_name.clear();
        self.call_count_by_name.clear();
    }

    pub fn is_open(&self, name: &str) -> bool {
        self.open.contains_key(name)
    }
}

impl Default for SpanTracker {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_span_tracker() -> SpanTracker {
    SpanTracker::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_and_end() {
        let mut st = new_span_tracker();
        st.begin("render", 1000);
        let dur = st.end("render", 2000, "frame");
        assert_eq!(dur, Some(1000));
    }

    #[test]
    fn end_without_begin_returns_none() {
        let mut st = new_span_tracker();
        assert!(st.end("ghost", 100, "").is_none());
    }

    #[test]
    fn total_ns_accumulates() {
        let mut st = new_span_tracker();
        st.begin("a", 0);
        st.end("a", 500, "");
        st.begin("a", 500);
        st.end("a", 1000, "");
        assert_eq!(st.total_ns("a"), 1000);
    }

    #[test]
    fn call_count() {
        let mut st = new_span_tracker();
        st.begin("x", 0);
        st.end("x", 10, "");
        st.begin("x", 10);
        st.end("x", 20, "");
        assert_eq!(st.call_count("x"), 2);
    }

    #[test]
    fn mean_ns() {
        let mut st = new_span_tracker();
        st.begin("y", 0);
        st.end("y", 100, "");
        st.begin("y", 100);
        st.end("y", 200, "");
        assert_eq!(st.mean_ns("y"), 100);
    }

    #[test]
    fn open_count() {
        let mut st = new_span_tracker();
        st.begin("a", 0);
        st.begin("b", 0);
        assert_eq!(st.open_count(), 2);
    }

    #[test]
    fn spans_for_tag() {
        let mut st = new_span_tracker();
        st.begin("a", 0);
        st.end("a", 10, "render");
        st.begin("b", 10);
        st.end("b", 20, "physics");
        assert_eq!(st.spans_for_tag("render").len(), 1);
    }

    #[test]
    fn is_open() {
        let mut st = new_span_tracker();
        st.begin("z", 0);
        assert!(st.is_open("z"));
        st.end("z", 1, "");
        assert!(!st.is_open("z"));
    }

    #[test]
    fn clear() {
        let mut st = new_span_tracker();
        st.begin("a", 0);
        st.end("a", 1, "");
        st.clear();
        assert_eq!(st.completed_count(), 0);
    }

    #[test]
    fn last_completed() {
        let mut st = new_span_tracker();
        st.begin("a", 0);
        st.end("a", 5, "t");
        assert_eq!(st.last_completed().unwrap().name, "a");
    }
}
