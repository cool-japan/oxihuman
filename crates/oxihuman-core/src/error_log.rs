// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Error log: a bounded, categorised list of runtime errors.

use std::fmt;

/// Severity level of an error entry.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Info => write!(f, "INFO"),
            ErrorSeverity::Warning => write!(f, "WARNING"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Fatal => write!(f, "FATAL"),
        }
    }
}

/// A single error entry.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ErrorEntry {
    pub severity: ErrorSeverity,
    pub category: String,
    pub message: String,
    pub code: u32,
}

/// Bounded error log.
#[derive(Debug)]
#[allow(dead_code)]
pub struct ErrorLog {
    entries: Vec<ErrorEntry>,
    capacity: usize,
    total_pushed: u64,
}

/// Create a new ErrorLog with given capacity.
#[allow(dead_code)]
pub fn new_error_log(capacity: usize) -> ErrorLog {
    ErrorLog {
        entries: Vec::new(),
        capacity,
        total_pushed: 0,
    }
}

/// Push an entry; evicts the oldest if full.
#[allow(dead_code)]
pub fn push_error(
    log: &mut ErrorLog,
    severity: ErrorSeverity,
    category: &str,
    message: &str,
    code: u32,
) {
    if log.entries.len() >= log.capacity && !log.entries.is_empty() {
        log.entries.remove(0);
    }
    log.entries.push(ErrorEntry {
        severity,
        category: category.to_string(),
        message: message.to_string(),
        code,
    });
    log.total_pushed += 1;
}

/// Count entries at or above a given severity.
#[allow(dead_code)]
pub fn count_by_severity(log: &ErrorLog, min: &ErrorSeverity) -> usize {
    log.entries.iter().filter(|e| &e.severity >= min).count()
}

/// Return all entries for a category.
#[allow(dead_code)]
pub fn entries_for_category<'a>(log: &'a ErrorLog, category: &str) -> Vec<&'a ErrorEntry> {
    log.entries
        .iter()
        .filter(|e| e.category == category)
        .collect()
}

/// Clear all entries.
#[allow(dead_code)]
pub fn clear_error_log(log: &mut ErrorLog) {
    log.entries.clear();
}

/// Total entries pushed lifetime.
#[allow(dead_code)]
pub fn total_pushed(log: &ErrorLog) -> u64 {
    log.total_pushed
}

/// Current entry count.
#[allow(dead_code)]
pub fn error_entry_count(log: &ErrorLog) -> usize {
    log.entries.len()
}

/// Whether any fatal entries exist.
#[allow(dead_code)]
pub fn has_fatal(log: &ErrorLog) -> bool {
    log.entries
        .iter()
        .any(|e| e.severity == ErrorSeverity::Fatal)
}

/// Last entry, if any.
#[allow(dead_code)]
pub fn last_error(log: &ErrorLog) -> Option<&ErrorEntry> {
    log.entries.last()
}

/// Serialize to JSON string.
#[allow(dead_code)]
pub fn error_log_to_json(log: &ErrorLog) -> String {
    let items: Vec<String> = log
        .entries
        .iter()
        .map(|e| {
            format!(
                r#"{{"severity":"{}","category":"{}","message":"{}","code":{}}}"#,
                e.severity, e.category, e.message, e.code
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_count() {
        let mut log = new_error_log(10);
        push_error(&mut log, ErrorSeverity::Error, "net", "timeout", 1);
        push_error(&mut log, ErrorSeverity::Warning, "io", "slow", 2);
        assert_eq!(error_entry_count(&log), 2);
    }

    #[test]
    fn test_capacity_eviction() {
        let mut log = new_error_log(3);
        for i in 0..5u32 {
            push_error(&mut log, ErrorSeverity::Info, "cat", "msg", i);
        }
        assert_eq!(error_entry_count(&log), 3);
        assert_eq!(log.entries[0].code, 2);
    }

    #[test]
    fn test_count_by_severity() {
        let mut log = new_error_log(10);
        push_error(&mut log, ErrorSeverity::Info, "a", "x", 0);
        push_error(&mut log, ErrorSeverity::Error, "b", "y", 1);
        push_error(&mut log, ErrorSeverity::Fatal, "c", "z", 2);
        assert_eq!(count_by_severity(&log, &ErrorSeverity::Error), 2);
    }

    #[test]
    fn test_entries_for_category() {
        let mut log = new_error_log(10);
        push_error(&mut log, ErrorSeverity::Error, "net", "a", 1);
        push_error(&mut log, ErrorSeverity::Error, "io", "b", 2);
        assert_eq!(entries_for_category(&log, "net").len(), 1);
    }

    #[test]
    fn test_has_fatal() {
        let mut log = new_error_log(10);
        push_error(&mut log, ErrorSeverity::Warning, "x", "y", 0);
        assert!(!has_fatal(&log));
        push_error(&mut log, ErrorSeverity::Fatal, "x", "z", 1);
        assert!(has_fatal(&log));
    }

    #[test]
    fn test_clear() {
        let mut log = new_error_log(10);
        push_error(&mut log, ErrorSeverity::Error, "x", "y", 0);
        clear_error_log(&mut log);
        assert_eq!(error_entry_count(&log), 0);
    }

    #[test]
    fn test_total_pushed() {
        let mut log = new_error_log(2);
        push_error(&mut log, ErrorSeverity::Info, "a", "b", 0);
        push_error(&mut log, ErrorSeverity::Info, "a", "b", 1);
        push_error(&mut log, ErrorSeverity::Info, "a", "b", 2);
        assert_eq!(total_pushed(&log), 3);
    }

    #[test]
    fn test_last_error() {
        let mut log = new_error_log(10);
        assert!(last_error(&log).is_none());
        push_error(&mut log, ErrorSeverity::Error, "cat", "last", 99);
        assert_eq!(last_error(&log).map(|e| e.code), Some(99));
    }

    #[test]
    fn test_json_output() {
        let mut log = new_error_log(10);
        push_error(&mut log, ErrorSeverity::Info, "sys", "ok", 0);
        let j = error_log_to_json(&log);
        assert!(j.contains("INFO"));
        assert!(j.contains("sys"));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(ErrorSeverity::Fatal > ErrorSeverity::Error);
        assert!(ErrorSeverity::Error > ErrorSeverity::Warning);
        assert!(ErrorSeverity::Warning > ErrorSeverity::Info);
    }
}
