// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Fixed-capacity ring-buffer log.

/// Log level for ring log entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RingLogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// A single log entry.
#[derive(Debug, Clone)]
pub struct RingLogEntry {
    pub level: RingLogLevel,
    pub message: String,
    pub seq: u64,
}

/// Fixed-size ring-buffer log.
pub struct RingLog {
    buf: Vec<Option<RingLogEntry>>,
    capacity: usize,
    head: usize,
    count: usize,
    seq: u64,
    error_count: usize,
    warn_count: usize,
}

#[allow(dead_code)]
impl RingLog {
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.max(1);
        RingLog {
            buf: (0..cap).map(|_| None).collect(),
            capacity: cap,
            head: 0,
            count: 0,
            seq: 0,
            error_count: 0,
            warn_count: 0,
        }
    }

    pub fn push(&mut self, level: RingLogLevel, message: &str) {
        match level {
            RingLogLevel::Error => self.error_count += 1,
            RingLogLevel::Warn => self.warn_count += 1,
            _ => {}
        }
        let entry = RingLogEntry {
            level,
            message: message.to_string(),
            seq: self.seq,
        };
        self.seq += 1;
        let slot = self.head % self.capacity;
        self.buf[slot] = Some(entry);
        self.head += 1;
        if self.count < self.capacity {
            self.count += 1;
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn error_count(&self) -> usize {
        self.error_count
    }

    pub fn warn_count(&self) -> usize {
        self.warn_count
    }

    pub fn entries(&self) -> Vec<&RingLogEntry> {
        let start = if self.count < self.capacity {
            0
        } else {
            self.head % self.capacity
        };
        let mut result = Vec::with_capacity(self.count);
        for i in 0..self.count {
            let idx = (start + i) % self.capacity;
            if let Some(e) = &self.buf[idx] {
                result.push(e);
            }
        }
        result
    }

    pub fn last(&self) -> Option<&RingLogEntry> {
        if self.count == 0 {
            return None;
        }
        let idx = (self.head + self.capacity - 1) % self.capacity;
        self.buf[idx].as_ref()
    }

    pub fn by_level(&self, level: &RingLogLevel) -> Vec<&RingLogEntry> {
        self.entries()
            .into_iter()
            .filter(|e| &e.level == level)
            .collect()
    }

    pub fn clear(&mut self) {
        for slot in &mut self.buf {
            *slot = None;
        }
        self.head = 0;
        self.count = 0;
        self.error_count = 0;
        self.warn_count = 0;
    }

    pub fn total_written(&self) -> u64 {
        self.seq
    }
}

pub fn new_ring_log(capacity: usize) -> RingLog {
    RingLog::new(capacity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_len() {
        let mut log = new_ring_log(10);
        log.push(RingLogLevel::Info, "hello");
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn wrap_around() {
        let mut log = new_ring_log(3);
        log.push(RingLogLevel::Info, "a");
        log.push(RingLogLevel::Info, "b");
        log.push(RingLogLevel::Info, "c");
        log.push(RingLogLevel::Info, "d");
        assert_eq!(log.len(), 3);
        let entries = log.entries();
        assert_eq!(entries.last().expect("should succeed").message, "d");
    }

    #[test]
    fn error_count_tracked() {
        let mut log = new_ring_log(10);
        log.push(RingLogLevel::Error, "oops");
        log.push(RingLogLevel::Warn, "meh");
        assert_eq!(log.error_count(), 1);
        assert_eq!(log.warn_count(), 1);
    }

    #[test]
    fn last_entry() {
        let mut log = new_ring_log(5);
        log.push(RingLogLevel::Info, "first");
        log.push(RingLogLevel::Debug, "last");
        assert_eq!(log.last().expect("should succeed").message, "last");
    }

    #[test]
    fn by_level_filter() {
        let mut log = new_ring_log(10);
        log.push(RingLogLevel::Info, "i");
        log.push(RingLogLevel::Error, "e");
        log.push(RingLogLevel::Info, "i2");
        let errors = log.by_level(&RingLogLevel::Info);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn clear_resets() {
        let mut log = new_ring_log(5);
        log.push(RingLogLevel::Error, "e");
        log.clear();
        assert!(log.is_empty());
        assert_eq!(log.error_count(), 0);
    }

    #[test]
    fn total_written_increases() {
        let mut log = new_ring_log(2);
        log.push(RingLogLevel::Info, "a");
        log.push(RingLogLevel::Info, "b");
        log.push(RingLogLevel::Info, "c");
        assert_eq!(log.total_written(), 3);
    }

    #[test]
    fn capacity_respected() {
        let log = new_ring_log(8);
        assert_eq!(log.capacity(), 8);
    }

    #[test]
    fn seq_numbers_increment() {
        let mut log = new_ring_log(10);
        log.push(RingLogLevel::Info, "a");
        log.push(RingLogLevel::Info, "b");
        let entries = log.entries();
        assert_eq!(entries[0].seq, 0);
        assert_eq!(entries[1].seq, 1);
    }
}
