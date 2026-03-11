// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Audit event log with tamper hash.

/// A single audit event entry.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub sequence: u64,
    pub actor: String,
    pub action: String,
    pub resource: String,
    pub timestamp_ms: u64,
    /// Simple rolling hash for tamper detection.
    pub entry_hash: u64,
}

/// Append-only audit log with a chain hash.
#[derive(Debug, Default)]
pub struct AuditLog {
    entries: Vec<AuditEntry>,
    prev_hash: u64,
    next_seq: u64,
}

fn simple_hash(prev: u64, seq: u64, actor: &str, action: &str, ts: u64) -> u64 {
    let mut h = prev.wrapping_add(seq.wrapping_mul(0x517cc1b727220a95));
    for b in actor.bytes().chain(action.bytes()) {
        h = h.wrapping_mul(6364136223846793005).wrapping_add(b as u64);
    }
    h = h.wrapping_add(ts);
    h
}

impl AuditLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, actor: &str, action: &str, resource: &str, timestamp_ms: u64) {
        let seq = self.next_seq;
        let hash = simple_hash(self.prev_hash, seq, actor, action, timestamp_ms);
        self.prev_hash = hash;
        self.next_seq += 1;
        self.entries.push(AuditEntry {
            sequence: seq,
            actor: actor.to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
            timestamp_ms,
            entry_hash: hash,
        });
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    pub fn chain_hash(&self) -> u64 {
        self.prev_hash
    }

    pub fn filter_actor(&self, actor: &str) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.actor == actor).collect()
    }

    pub fn filter_action(&self, action: &str) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.action == action).collect()
    }
}

pub fn new_audit_log() -> AuditLog {
    AuditLog::new()
}

pub fn audit_record(log: &mut AuditLog, actor: &str, action: &str, resource: &str, ts: u64) {
    log.record(actor, action, resource, ts);
}

pub fn audit_count(log: &AuditLog) -> usize {
    log.entry_count()
}

pub fn audit_chain_hash(log: &AuditLog) -> u64 {
    log.chain_hash()
}

pub fn audit_filter_actor<'a>(log: &'a AuditLog, actor: &str) -> Vec<&'a AuditEntry> {
    log.filter_actor(actor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_count() {
        let mut log = new_audit_log();
        audit_record(&mut log, "alice", "login", "system", 1000);
        assert_eq!(audit_count(&log), 1);
    }

    #[test]
    fn test_sequence_increments() {
        let mut log = new_audit_log();
        audit_record(&mut log, "a", "x", "r", 0);
        audit_record(&mut log, "b", "y", "r", 1);
        assert_eq!(log.entries()[0].sequence, 0);
        assert_eq!(log.entries()[1].sequence, 1);
    }

    #[test]
    fn test_hash_changes_with_each_entry() {
        let mut log = new_audit_log();
        let h0 = audit_chain_hash(&log);
        audit_record(&mut log, "u", "act", "r", 1000);
        let h1 = audit_chain_hash(&log);
        assert_ne!(h0, h1);
    }

    #[test]
    fn test_filter_actor() {
        let mut log = new_audit_log();
        audit_record(&mut log, "alice", "login", "sys", 0);
        audit_record(&mut log, "bob", "logout", "sys", 1);
        let alices = audit_filter_actor(&log, "alice");
        assert_eq!(alices.len(), 1);
    }

    #[test]
    fn test_filter_action() {
        let mut log = new_audit_log();
        audit_record(&mut log, "u1", "delete", "file1", 0);
        audit_record(&mut log, "u2", "delete", "file2", 1);
        audit_record(&mut log, "u3", "read", "file3", 2);
        assert_eq!(log.filter_action("delete").len(), 2);
    }

    #[test]
    fn test_entry_has_actor_stored() {
        let mut log = new_audit_log();
        audit_record(&mut log, "carol", "update", "profile", 9999);
        assert_eq!(log.entries()[0].actor, "carol");
    }

    #[test]
    fn test_initial_chain_hash_is_zero() {
        let log = new_audit_log();
        assert_eq!(audit_chain_hash(&log), 0);
    }

    #[test]
    fn test_different_timestamps_different_hashes() {
        let mut log1 = new_audit_log();
        let mut log2 = new_audit_log();
        audit_record(&mut log1, "u", "act", "r", 100);
        audit_record(&mut log2, "u", "act", "r", 200);
        assert_ne!(audit_chain_hash(&log1), audit_chain_hash(&log2));
    }

    #[test]
    fn test_resource_stored() {
        let mut log = new_audit_log();
        audit_record(&mut log, "u", "read", "database", 0);
        assert_eq!(log.entries()[0].resource, "database");
    }
}
