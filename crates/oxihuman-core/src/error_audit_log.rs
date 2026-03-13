// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A single audit event.
pub struct AuditEventEntry {
    pub timestamp_ms: u64,
    pub actor: String,
    pub action: String,
}

/// Append-only audit event log.
pub struct AuditEventLog {
    pub entries: Vec<AuditEventEntry>,
}

impl AuditEventLog {
    pub fn new() -> Self {
        AuditEventLog {
            entries: Vec::new(),
        }
    }
}

impl Default for AuditEventLog {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_audit_event_log() -> AuditEventLog {
    AuditEventLog::new()
}

pub fn audit_event_record(log: &mut AuditEventLog, ts: u64, actor: &str, action: &str) {
    log.entries.push(AuditEventEntry {
        timestamp_ms: ts,
        actor: actor.to_string(),
        action: action.to_string(),
    });
}

pub fn audit_event_count(log: &AuditEventLog) -> usize {
    log.entries.len()
}

pub fn audit_event_last(log: &AuditEventLog) -> Option<&AuditEventEntry> {
    log.entries.last()
}

pub fn audit_event_clear(log: &mut AuditEventLog) {
    log.entries.clear();
}

pub fn audit_event_by_actor<'a>(log: &'a AuditEventLog, actor: &str) -> Vec<&'a AuditEventEntry> {
    log.entries.iter().filter(|e| e.actor == actor).collect()
}

pub fn audit_event_since(log: &AuditEventLog, ts: u64) -> Vec<&AuditEventEntry> {
    log.entries
        .iter()
        .filter(|e| e.timestamp_ms >= ts)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        /* new log has no entries */
        let log = new_audit_event_log();
        assert_eq!(audit_event_count(&log), 0);
    }

    #[test]
    fn test_record_and_count() {
        /* recording increments count */
        let mut log = new_audit_event_log();
        audit_event_record(&mut log, 1000, "alice", "login");
        assert_eq!(audit_event_count(&log), 1);
    }

    #[test]
    fn test_last_entry() {
        /* last returns most recent entry */
        let mut log = new_audit_event_log();
        audit_event_record(&mut log, 100, "bob", "create");
        audit_event_record(&mut log, 200, "alice", "delete");
        let last = audit_event_last(&log).expect("should succeed");
        assert_eq!(last.actor, "alice");
        assert_eq!(last.action, "delete");
    }

    #[test]
    fn test_clear() {
        /* clear removes all entries */
        let mut log = new_audit_event_log();
        audit_event_record(&mut log, 1, "x", "y");
        audit_event_clear(&mut log);
        assert_eq!(audit_event_count(&log), 0);
    }

    #[test]
    fn test_by_actor() {
        /* by_actor filters correctly */
        let mut log = new_audit_event_log();
        audit_event_record(&mut log, 1, "alice", "a");
        audit_event_record(&mut log, 2, "bob", "b");
        audit_event_record(&mut log, 3, "alice", "c");
        let alice_events = audit_event_by_actor(&log, "alice");
        assert_eq!(alice_events.len(), 2);
    }

    #[test]
    fn test_since() {
        /* since filters by timestamp */
        let mut log = new_audit_event_log();
        audit_event_record(&mut log, 100, "x", "a");
        audit_event_record(&mut log, 200, "x", "b");
        audit_event_record(&mut log, 300, "x", "c");
        let recent = audit_event_since(&log, 200);
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn test_last_empty() {
        /* last on empty log returns None */
        let log = new_audit_event_log();
        assert!(audit_event_last(&log).is_none());
    }

    #[test]
    fn test_multiple_actors() {
        /* by_actor returns empty for unknown actor */
        let mut log = new_audit_event_log();
        audit_event_record(&mut log, 1, "alice", "x");
        let result = audit_event_by_actor(&log, "ghost");
        assert!(result.is_empty());
    }
}
