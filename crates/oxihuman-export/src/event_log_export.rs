// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Structured event log (timestamp + payload) export.

/// An event log entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EventEntry {
    pub timestamp_ms: u64,
    pub event_type: String,
    pub severity: EventSeverity,
    pub payload: String,
}

/// Severity level.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum EventSeverity {
    Debug,
    Info,
    Warning,
    Error,
}

impl EventSeverity {
    pub fn as_str(&self) -> &str {
        match self {
            EventSeverity::Debug => "DEBUG",
            EventSeverity::Info => "INFO",
            EventSeverity::Warning => "WARNING",
            EventSeverity::Error => "ERROR",
        }
    }
}

/// A structured event log.
#[allow(dead_code)]
pub struct EventLog {
    pub entries: Vec<EventEntry>,
}

impl EventLog {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl Default for EventLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Add an event entry.
#[allow(dead_code)]
pub fn log_event(
    log: &mut EventLog,
    timestamp_ms: u64,
    event_type: &str,
    severity: EventSeverity,
    payload: &str,
) {
    log.entries.push(EventEntry {
        timestamp_ms,
        event_type: event_type.to_string(),
        severity,
        payload: payload.to_string(),
    });
}

/// Export to structured CSV.
#[allow(dead_code)]
pub fn export_event_log_csv(log: &EventLog) -> String {
    let mut out = String::from("timestamp_ms,event_type,severity,payload\n");
    for e in &log.entries {
        out.push_str(&format!(
            "{},{},{},{}\n",
            e.timestamp_ms,
            e.event_type,
            e.severity.as_str(),
            e.payload
        ));
    }
    out
}

/// Export to NDJSON (newline-delimited JSON).
#[allow(dead_code)]
pub fn export_event_log_ndjson(log: &EventLog) -> String {
    let mut out = String::new();
    for e in &log.entries {
        out.push_str(&format!(
            "{{\"ts\":{},\"type\":\"{}\",\"severity\":\"{}\",\"payload\":\"{}\"}}\n",
            e.timestamp_ms,
            e.event_type,
            e.severity.as_str(),
            e.payload
        ));
    }
    out
}

/// Count events by severity.
#[allow(dead_code)]
pub fn count_by_severity(log: &EventLog, severity: &EventSeverity) -> usize {
    log.entries
        .iter()
        .filter(|e| &e.severity == severity)
        .count()
}

/// Total event count.
#[allow(dead_code)]
pub fn event_count(log: &EventLog) -> usize {
    log.entries.len()
}

/// Filter events by type.
#[allow(dead_code)]
pub fn filter_by_type<'a>(log: &'a EventLog, event_type: &str) -> Vec<&'a EventEntry> {
    log.entries
        .iter()
        .filter(|e| e.event_type == event_type)
        .collect()
}

/// Has errors.
#[allow(dead_code)]
pub fn has_errors(log: &EventLog) -> bool {
    log.entries
        .iter()
        .any(|e| e.severity == EventSeverity::Error)
}

/// Sort events by timestamp.
#[allow(dead_code)]
pub fn sort_events_by_time(log: &mut EventLog) {
    log.entries.sort_by_key(|e| e.timestamp_ms);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_log() -> EventLog {
        let mut log = EventLog::new();
        log_event(
            &mut log,
            0,
            "startup",
            EventSeverity::Info,
            "system started",
        );
        log_event(
            &mut log,
            100,
            "mesh_load",
            EventSeverity::Info,
            "mesh loaded OK",
        );
        log_event(
            &mut log,
            200,
            "export_fail",
            EventSeverity::Error,
            "write error",
        );
        log_event(
            &mut log,
            300,
            "debug_dump",
            EventSeverity::Debug,
            "vertices=100",
        );
        log
    }

    #[test]
    fn event_count_correct() {
        let log = sample_log();
        assert_eq!(event_count(&log), 4);
    }

    #[test]
    fn count_errors_one() {
        let log = sample_log();
        assert_eq!(count_by_severity(&log, &EventSeverity::Error), 1);
    }

    #[test]
    fn has_errors_true() {
        let log = sample_log();
        assert!(has_errors(&log));
    }

    #[test]
    fn no_errors_when_clean() {
        let log = EventLog::new();
        assert!(!has_errors(&log));
    }

    #[test]
    fn csv_header_present() {
        let log = sample_log();
        let csv = export_event_log_csv(&log);
        assert!(csv.starts_with("timestamp_ms,event_type,severity,payload"));
    }

    #[test]
    fn ndjson_line_count() {
        let log = sample_log();
        let ndjson = export_event_log_ndjson(&log);
        let lines: Vec<&str> = ndjson.trim().split('\n').collect();
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn filter_by_type_correct() {
        let log = sample_log();
        let entries = filter_by_type(&log, "startup");
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn sort_events_by_time_ordered() {
        let mut log = EventLog::new();
        log_event(&mut log, 300, "c", EventSeverity::Info, "");
        log_event(&mut log, 100, "a", EventSeverity::Info, "");
        log_event(&mut log, 200, "b", EventSeverity::Info, "");
        sort_events_by_time(&mut log);
        assert_eq!(log.entries[0].timestamp_ms, 100);
    }

    #[test]
    fn severity_as_str_correct() {
        assert_eq!(EventSeverity::Error.as_str(), "ERROR");
        assert_eq!(EventSeverity::Warning.as_str(), "WARNING");
    }

    #[test]
    fn count_info_two() {
        let log = sample_log();
        assert_eq!(count_by_severity(&log, &EventSeverity::Info), 2);
    }
}
