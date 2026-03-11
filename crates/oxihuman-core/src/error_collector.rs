// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Error/diagnostic collector.

#![allow(dead_code)]

/// Severity level of a diagnostic entry.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorLevel {
    Info,
    Warning,
    Error,
}

/// A single diagnostic entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ErrorEntry {
    pub level: ErrorLevel,
    pub code: u32,
    pub message: String,
}

/// Collects diagnostic messages.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ErrorCollector {
    entries: Vec<ErrorEntry>,
}

/// Create a new error collector.
#[allow(dead_code)]
pub fn new_error_collector() -> ErrorCollector {
    ErrorCollector { entries: Vec::new() }
}

/// Collect an error entry.
#[allow(dead_code)]
pub fn collect_error(collector: &mut ErrorCollector, code: u32, message: &str) {
    collector.entries.push(ErrorEntry {
        level: ErrorLevel::Error,
        code,
        message: message.to_string(),
    });
}

/// Collect a warning entry.
#[allow(dead_code)]
pub fn collect_warning(collector: &mut ErrorCollector, code: u32, message: &str) {
    collector.entries.push(ErrorEntry {
        level: ErrorLevel::Warning,
        code,
        message: message.to_string(),
    });
}

/// Collect an info entry.
#[allow(dead_code)]
pub fn collect_info(collector: &mut ErrorCollector, code: u32, message: &str) {
    collector.entries.push(ErrorEntry {
        level: ErrorLevel::Info,
        code,
        message: message.to_string(),
    });
}

/// Return the number of error-level entries.
#[allow(dead_code)]
pub fn error_count(collector: &ErrorCollector) -> usize {
    collector.entries.iter().filter(|e| e.level == ErrorLevel::Error).count()
}

/// Return the number of warning-level entries.
#[allow(dead_code)]
pub fn warning_count(collector: &ErrorCollector) -> usize {
    collector.entries.iter().filter(|e| e.level == ErrorLevel::Warning).count()
}

/// Return the number of info-level entries.
#[allow(dead_code)]
pub fn info_count(collector: &ErrorCollector) -> usize {
    collector.entries.iter().filter(|e| e.level == ErrorLevel::Info).count()
}

/// Return the total number of entries.
#[allow(dead_code)]
pub fn total_count(collector: &ErrorCollector) -> usize {
    collector.entries.len()
}

/// Return true if any errors were collected.
#[allow(dead_code)]
pub fn has_errors(collector: &ErrorCollector) -> bool {
    collector.entries.iter().any(|e| e.level == ErrorLevel::Error)
}

/// Clear all collected entries.
#[allow(dead_code)]
pub fn clear_errors(collector: &mut ErrorCollector) {
    collector.entries.clear();
}

/// Return a reference to all collected entries.
#[allow(dead_code)]
pub fn get_errors(collector: &ErrorCollector) -> &[ErrorEntry] {
    &collector.entries
}

/// Serialize all entries to a JSON string.
#[allow(dead_code)]
pub fn errors_to_json(collector: &ErrorCollector) -> String {
    let mut parts = Vec::new();
    for e in &collector.entries {
        let level = match e.level {
            ErrorLevel::Error => "error",
            ErrorLevel::Warning => "warning",
            ErrorLevel::Info => "info",
        };
        let msg = e.message.replace('"', "\\\"");
        parts.push(format!(
            "{{\"level\":\"{}\",\"code\":{},\"message\":\"{}\"}}",
            level, e.code, msg
        ));
    }
    format!("[{}]", parts.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_collector_empty() {
        let c = new_error_collector();
        assert_eq!(total_count(&c), 0);
        assert!(!has_errors(&c));
    }

    #[test]
    fn test_collect_error() {
        let mut c = new_error_collector();
        collect_error(&mut c, 100, "something failed");
        assert_eq!(error_count(&c), 1);
        assert!(has_errors(&c));
    }

    #[test]
    fn test_collect_warning() {
        let mut c = new_error_collector();
        collect_warning(&mut c, 200, "be careful");
        assert_eq!(warning_count(&c), 1);
        assert!(!has_errors(&c));
    }

    #[test]
    fn test_collect_info() {
        let mut c = new_error_collector();
        collect_info(&mut c, 300, "fyi");
        assert_eq!(info_count(&c), 1);
    }

    #[test]
    fn test_total_count() {
        let mut c = new_error_collector();
        collect_error(&mut c, 1, "e");
        collect_warning(&mut c, 2, "w");
        collect_info(&mut c, 3, "i");
        assert_eq!(total_count(&c), 3);
    }

    #[test]
    fn test_clear() {
        let mut c = new_error_collector();
        collect_error(&mut c, 1, "x");
        clear_errors(&mut c);
        assert_eq!(total_count(&c), 0);
    }

    #[test]
    fn test_get_errors_slice() {
        let mut c = new_error_collector();
        collect_error(&mut c, 42, "msg");
        let entries = get_errors(&c);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].code, 42);
    }

    #[test]
    fn test_errors_to_json() {
        let mut c = new_error_collector();
        collect_error(&mut c, 1, "oops");
        let json = errors_to_json(&c);
        assert!(json.contains("\"level\":\"error\""));
        assert!(json.contains("\"code\":1"));
        assert!(json.contains("\"message\":\"oops\""));
    }

    #[test]
    fn test_errors_to_json_empty() {
        let c = new_error_collector();
        assert_eq!(errors_to_json(&c), "[]");
    }

    #[test]
    fn test_error_level_clone() {
        let level = ErrorLevel::Warning;
        assert_eq!(level.clone(), ErrorLevel::Warning);
    }
}
