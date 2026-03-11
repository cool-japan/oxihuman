// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Log level aggregator and filter.

/// Log level severity ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AggLogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

/// An aggregated log entry.
#[derive(Debug, Clone)]
pub struct AggLogEntry {
    pub level: AggLogLevel,
    pub message: String,
    pub source: String,
}

/// Aggregates log entries with level filtering.
#[derive(Debug)]
pub struct LogAggregator {
    min_level: AggLogLevel,
    entries: Vec<AggLogEntry>,
}

impl LogAggregator {
    pub fn new(min_level: AggLogLevel) -> Self {
        Self {
            min_level,
            entries: Vec::new(),
        }
    }

    pub fn push(&mut self, level: AggLogLevel, source: &str, message: &str) {
        if level >= self.min_level {
            self.entries.push(AggLogEntry {
                level,
                message: message.to_string(),
                source: source.to_string(),
            });
        }
    }

    pub fn entries(&self) -> &[AggLogEntry] {
        &self.entries
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn count_at_level(&self, level: AggLogLevel) -> usize {
        self.entries.iter().filter(|e| e.level == level).count()
    }

    pub fn set_min_level(&mut self, level: AggLogLevel) {
        self.min_level = level;
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn filter_source<'a>(&'a self, source: &str) -> Vec<&'a AggLogEntry> {
        self.entries.iter().filter(|e| e.source == source).collect()
    }
}

pub fn new_log_aggregator(min_level: AggLogLevel) -> LogAggregator {
    LogAggregator::new(min_level)
}

pub fn agg_push(agg: &mut LogAggregator, level: AggLogLevel, source: &str, msg: &str) {
    agg.push(level, source, msg);
}

pub fn agg_count(agg: &LogAggregator) -> usize {
    agg.count()
}

pub fn agg_count_level(agg: &LogAggregator, level: AggLogLevel) -> usize {
    agg.count_at_level(level)
}

pub fn agg_set_min(agg: &mut LogAggregator, level: AggLogLevel) {
    agg.set_min_level(level);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_above_min() {
        let mut agg = new_log_aggregator(AggLogLevel::Info);
        agg_push(&mut agg, AggLogLevel::Info, "srv", "started");
        assert_eq!(agg_count(&agg), 1);
    }

    #[test]
    fn test_filter_below_min() {
        let mut agg = new_log_aggregator(AggLogLevel::Warn);
        agg_push(&mut agg, AggLogLevel::Debug, "srv", "verbose");
        assert_eq!(agg_count(&agg), 0);
    }

    #[test]
    fn test_count_at_level() {
        let mut agg = new_log_aggregator(AggLogLevel::Debug);
        agg_push(&mut agg, AggLogLevel::Error, "src", "boom");
        agg_push(&mut agg, AggLogLevel::Info, "src", "ok");
        assert_eq!(agg_count_level(&agg, AggLogLevel::Error), 1);
    }

    #[test]
    fn test_clear() {
        let mut agg = new_log_aggregator(AggLogLevel::Info);
        agg_push(&mut agg, AggLogLevel::Info, "s", "msg");
        agg.clear();
        assert_eq!(agg_count(&agg), 0);
    }

    #[test]
    fn test_set_min_level_changes_filtering() {
        let mut agg = new_log_aggregator(AggLogLevel::Error);
        agg_set_min(&mut agg, AggLogLevel::Debug);
        agg_push(&mut agg, AggLogLevel::Debug, "s", "now visible");
        assert_eq!(agg_count(&agg), 1);
    }

    #[test]
    fn test_filter_source() {
        let mut agg = new_log_aggregator(AggLogLevel::Info);
        agg_push(&mut agg, AggLogLevel::Info, "auth", "login");
        agg_push(&mut agg, AggLogLevel::Info, "api", "req");
        let auth_entries = agg.filter_source("auth");
        assert_eq!(auth_entries.len(), 1);
    }

    #[test]
    fn test_level_ordering() {
        assert!(AggLogLevel::Error > AggLogLevel::Info);
    }

    #[test]
    fn test_fatal_always_captured() {
        let mut agg = new_log_aggregator(AggLogLevel::Error);
        agg_push(&mut agg, AggLogLevel::Fatal, "sys", "crashed");
        assert_eq!(agg_count(&agg), 1);
    }

    #[test]
    fn test_multiple_sources() {
        let mut agg = new_log_aggregator(AggLogLevel::Info);
        agg_push(&mut agg, AggLogLevel::Info, "a", "msg1");
        agg_push(&mut agg, AggLogLevel::Info, "b", "msg2");
        agg_push(&mut agg, AggLogLevel::Info, "a", "msg3");
        assert_eq!(agg.filter_source("a").len(), 2);
        assert_eq!(agg.filter_source("b").len(), 1);
    }
}
