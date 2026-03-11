// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Audit log entry export with actor and action fields.

/// An audit log entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub id: u64,
    pub timestamp_ms: u64,
    pub actor: String,
    pub action: String,
    pub resource: String,
    pub outcome: AuditOutcome,
    pub details: String,
}

/// Outcome of an audited action.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum AuditOutcome {
    Success,
    Failure,
    Denied,
}

impl AuditOutcome {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            AuditOutcome::Success => "SUCCESS",
            AuditOutcome::Failure => "FAILURE",
            AuditOutcome::Denied => "DENIED",
        }
    }
}

/// An audit log.
#[allow(dead_code)]
pub struct AuditLog {
    pub entries: Vec<AuditEntry>,
    next_id: u64,
}

impl AuditLog {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 0,
        }
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Append an audit entry.
#[allow(dead_code)]
pub fn log_audit(
    log: &mut AuditLog,
    timestamp_ms: u64,
    actor: &str,
    action: &str,
    resource: &str,
    outcome: AuditOutcome,
    details: &str,
) -> u64 {
    let id = log.next_id;
    log.next_id += 1;
    log.entries.push(AuditEntry {
        id,
        timestamp_ms,
        actor: actor.to_string(),
        action: action.to_string(),
        resource: resource.to_string(),
        outcome,
        details: details.to_string(),
    });
    id
}

/// Export audit log to CSV string.
#[allow(dead_code)]
pub fn export_audit_csv(log: &AuditLog) -> String {
    let mut out = String::from("id,timestamp_ms,actor,action,resource,outcome,details\n");
    for e in &log.entries {
        out.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            e.id,
            e.timestamp_ms,
            e.actor,
            e.action,
            e.resource,
            e.outcome.as_str(),
            e.details
        ));
    }
    out
}

/// Export audit log to NDJSON.
#[allow(dead_code)]
pub fn export_audit_ndjson(log: &AuditLog) -> String {
    let mut out = String::new();
    for e in &log.entries {
        out.push_str(&format!(
            "{{\"id\":{},\"ts\":{},\"actor\":\"{}\",\"action\":\"{}\",\
            \"resource\":\"{}\",\"outcome\":\"{}\",\"details\":\"{}\"}}\n",
            e.id,
            e.timestamp_ms,
            e.actor,
            e.action,
            e.resource,
            e.outcome.as_str(),
            e.details
        ));
    }
    out
}

/// Total entry count.
#[allow(dead_code)]
pub fn audit_entry_count(log: &AuditLog) -> usize {
    log.entries.len()
}

/// Count entries by outcome.
#[allow(dead_code)]
pub fn count_by_outcome(log: &AuditLog, outcome: &AuditOutcome) -> usize {
    log.entries.iter().filter(|e| &e.outcome == outcome).count()
}

/// Count entries by actor.
#[allow(dead_code)]
pub fn count_by_actor(log: &AuditLog, actor: &str) -> usize {
    log.entries.iter().filter(|e| e.actor == actor).count()
}

/// Filter entries by action.
#[allow(dead_code)]
pub fn filter_by_action<'a>(log: &'a AuditLog, action: &str) -> Vec<&'a AuditEntry> {
    log.entries.iter().filter(|e| e.action == action).collect()
}

/// Whether any denials occurred.
#[allow(dead_code)]
pub fn has_denials(log: &AuditLog) -> bool {
    log.entries
        .iter()
        .any(|e| e.outcome == AuditOutcome::Denied)
}

/// Sort entries by timestamp ascending.
#[allow(dead_code)]
pub fn sort_by_timestamp(log: &mut AuditLog) {
    log.entries.sort_by_key(|e| e.timestamp_ms);
}

/// Latest entry timestamp.
#[allow(dead_code)]
pub fn latest_timestamp(log: &AuditLog) -> Option<u64> {
    log.entries.iter().map(|e| e.timestamp_ms).max()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_log() -> AuditLog {
        let mut log = AuditLog::new();
        log_audit(
            &mut log,
            0,
            "alice",
            "login",
            "auth",
            AuditOutcome::Success,
            "",
        );
        log_audit(
            &mut log,
            100,
            "bob",
            "read",
            "mesh/head.glb",
            AuditOutcome::Success,
            "",
        );
        log_audit(
            &mut log,
            200,
            "carol",
            "delete",
            "mesh/body.glb",
            AuditOutcome::Denied,
            "permission denied",
        );
        log_audit(
            &mut log,
            300,
            "alice",
            "write",
            "mesh/hand.glb",
            AuditOutcome::Failure,
            "disk full",
        );
        log
    }

    #[test]
    fn audit_entry_count_correct() {
        let log = sample_log();
        assert_eq!(audit_entry_count(&log), 4);
    }

    #[test]
    fn count_success_correct() {
        let log = sample_log();
        assert_eq!(count_by_outcome(&log, &AuditOutcome::Success), 2);
    }

    #[test]
    fn count_denied_correct() {
        let log = sample_log();
        assert_eq!(count_by_outcome(&log, &AuditOutcome::Denied), 1);
    }

    #[test]
    fn count_by_actor_alice() {
        let log = sample_log();
        assert_eq!(count_by_actor(&log, "alice"), 2);
    }

    #[test]
    fn has_denials_true() {
        let log = sample_log();
        assert!(has_denials(&log));
    }

    #[test]
    fn has_denials_false_empty() {
        let log = AuditLog::new();
        assert!(!has_denials(&log));
    }

    #[test]
    fn csv_header_present() {
        let log = sample_log();
        let csv = export_audit_csv(&log);
        assert!(csv.starts_with("id,timestamp_ms,actor,action"));
    }

    #[test]
    fn ndjson_line_count() {
        let log = sample_log();
        let ndjson = export_audit_ndjson(&log);
        let lines: Vec<&str> = ndjson.trim().split('\n').collect();
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn filter_by_action_correct() {
        let log = sample_log();
        let reads = filter_by_action(&log, "read");
        assert_eq!(reads.len(), 1);
    }

    #[test]
    fn latest_timestamp_correct() {
        let log = sample_log();
        assert_eq!(latest_timestamp(&log), Some(300));
    }
}
