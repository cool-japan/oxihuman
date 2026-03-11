// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Structured change log entry.

/// Kind of change recorded.
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
}

/// A single structured change log entry.
#[derive(Debug, Clone)]
pub struct ChangeEntry {
    pub id: u64,
    pub kind: ChangeKind,
    pub path: String,
    pub author: String,
    pub description: String,
    pub timestamp_ms: u64,
}

/// Append-only change log.
#[derive(Debug, Default)]
pub struct ChangeLog {
    entries: Vec<ChangeEntry>,
    next_id: u64,
}

impl ChangeLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(
        &mut self,
        kind: ChangeKind,
        path: &str,
        author: &str,
        description: &str,
        timestamp_ms: u64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(ChangeEntry {
            id,
            kind,
            path: path.to_string(),
            author: author.to_string(),
            description: description.to_string(),
            timestamp_ms,
        });
        id
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn entries(&self) -> &[ChangeEntry] {
        &self.entries
    }

    pub fn filter_kind(&self, kind: &ChangeKind) -> Vec<&ChangeEntry> {
        self.entries.iter().filter(|e| &e.kind == kind).collect()
    }

    pub fn filter_author(&self, author: &str) -> Vec<&ChangeEntry> {
        self.entries.iter().filter(|e| e.author == author).collect()
    }

    pub fn since(&self, timestamp_ms: u64) -> Vec<&ChangeEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestamp_ms >= timestamp_ms)
            .collect()
    }
}

pub fn new_change_log() -> ChangeLog {
    ChangeLog::new()
}

pub fn cl_append(
    log: &mut ChangeLog,
    kind: ChangeKind,
    path: &str,
    author: &str,
    desc: &str,
    ts: u64,
) -> u64 {
    log.append(kind, path, author, desc, ts)
}

pub fn cl_count(log: &ChangeLog) -> usize {
    log.entry_count()
}

pub fn cl_filter_kind<'a>(log: &'a ChangeLog, kind: &ChangeKind) -> Vec<&'a ChangeEntry> {
    log.filter_kind(kind)
}

pub fn cl_since(log: &ChangeLog, ts: u64) -> Vec<&ChangeEntry> {
    log.since(ts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_and_count() {
        let mut log = new_change_log();
        cl_append(
            &mut log,
            ChangeKind::Added,
            "src/main.rs",
            "alice",
            "init",
            1000,
        );
        assert_eq!(cl_count(&log), 1);
    }

    #[test]
    fn test_id_increments() {
        let mut log = new_change_log();
        let id0 = cl_append(&mut log, ChangeKind::Added, "a", "u", "d", 0);
        let id1 = cl_append(&mut log, ChangeKind::Modified, "b", "u", "d", 1);
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
    }

    #[test]
    fn test_filter_kind_added() {
        let mut log = new_change_log();
        cl_append(&mut log, ChangeKind::Added, "f1", "u", "d", 0);
        cl_append(&mut log, ChangeKind::Deleted, "f2", "u", "d", 1);
        assert_eq!(cl_filter_kind(&log, &ChangeKind::Added).len(), 1);
    }

    #[test]
    fn test_filter_author() {
        let mut log = new_change_log();
        cl_append(&mut log, ChangeKind::Added, "f", "alice", "d", 0);
        cl_append(&mut log, ChangeKind::Added, "g", "bob", "d", 1);
        assert_eq!(log.filter_author("alice").len(), 1);
    }

    #[test]
    fn test_since_filter() {
        let mut log = new_change_log();
        cl_append(&mut log, ChangeKind::Added, "f", "u", "d", 100);
        cl_append(&mut log, ChangeKind::Added, "g", "u", "d", 200);
        cl_append(&mut log, ChangeKind::Added, "h", "u", "d", 300);
        assert_eq!(cl_since(&log, 200).len(), 2);
    }

    #[test]
    fn test_renamed_kind() {
        let mut log = new_change_log();
        cl_append(&mut log, ChangeKind::Renamed, "old", "u", "renamed", 0);
        assert_eq!(cl_filter_kind(&log, &ChangeKind::Renamed).len(), 1);
    }

    #[test]
    fn test_empty_log() {
        let log = new_change_log();
        assert_eq!(cl_count(&log), 0);
    }

    #[test]
    fn test_entries_stored() {
        let mut log = new_change_log();
        cl_append(
            &mut log,
            ChangeKind::Modified,
            "cfg.toml",
            "dev",
            "tweaks",
            500,
        );
        assert_eq!(log.entries()[0].path, "cfg.toml");
    }

    #[test]
    fn test_multiple_same_kind() {
        let mut log = new_change_log();
        for i in 0..5 {
            cl_append(&mut log, ChangeKind::Modified, "f", "u", "d", i);
        }
        assert_eq!(cl_filter_kind(&log, &ChangeKind::Modified).len(), 5);
    }
}
