// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A sequence ID generator that produces unique, monotonically increasing IDs.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SequenceIdGen {
    next: u64,
    prefix: String,
}

#[allow(dead_code)]
impl SequenceIdGen {
    pub fn new() -> Self {
        Self {
            next: 1,
            prefix: String::new(),
        }
    }

    pub fn with_prefix(prefix: &str) -> Self {
        Self {
            next: 1,
            prefix: prefix.to_string(),
        }
    }

    pub fn with_start(start: u64) -> Self {
        Self {
            next: start,
            prefix: String::new(),
        }
    }

    pub fn next_id(&mut self) -> u64 {
        let id = self.next;
        self.next += 1;
        id
    }

    pub fn next_string_id(&mut self) -> String {
        let id = self.next_id();
        if self.prefix.is_empty() {
            format!("{id}")
        } else {
            format!("{}_{id}", self.prefix)
        }
    }

    pub fn peek_next(&self) -> u64 {
        self.next
    }

    pub fn total_generated(&self) -> u64 {
        self.next.saturating_sub(1)
    }

    pub fn reset(&mut self) {
        self.next = 1;
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }
}

impl Default for SequenceIdGen {
    fn default() -> Self {
        Self::new()
    }
}

/// A typed wrapper providing compile-time type safety for IDs.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SequenceId(pub u64);

#[allow(dead_code)]
impl SequenceId {
    pub fn value(self) -> u64 {
        self.0
    }

    pub fn is_valid(self) -> bool {
        self.0 > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_starts_at_one() {
        let mut gen = SequenceIdGen::new();
        assert_eq!(gen.next_id(), 1);
    }

    #[test]
    fn test_monotonic() {
        let mut gen = SequenceIdGen::new();
        let a = gen.next_id();
        let b = gen.next_id();
        let c = gen.next_id();
        assert!(a < b);
        assert!(b < c);
    }

    #[test]
    fn test_with_prefix() {
        let mut gen = SequenceIdGen::with_prefix("obj");
        let s = gen.next_string_id();
        assert_eq!(s, "obj_1");
    }

    #[test]
    fn test_string_id_no_prefix() {
        let mut gen = SequenceIdGen::new();
        assert_eq!(gen.next_string_id(), "1");
    }

    #[test]
    fn test_with_start() {
        let mut gen = SequenceIdGen::with_start(100);
        assert_eq!(gen.next_id(), 100);
    }

    #[test]
    fn test_peek_next() {
        let mut gen = SequenceIdGen::new();
        gen.next_id();
        assert_eq!(gen.peek_next(), 2);
    }

    #[test]
    fn test_total_generated() {
        let mut gen = SequenceIdGen::new();
        gen.next_id();
        gen.next_id();
        gen.next_id();
        assert_eq!(gen.total_generated(), 3);
    }

    #[test]
    fn test_reset() {
        let mut gen = SequenceIdGen::new();
        gen.next_id();
        gen.next_id();
        gen.reset();
        assert_eq!(gen.next_id(), 1);
    }

    #[test]
    fn test_sequence_id_value() {
        let id = SequenceId(42);
        assert_eq!(id.value(), 42);
        assert!(id.is_valid());
    }

    #[test]
    fn test_sequence_id_invalid() {
        let id = SequenceId(0);
        assert!(!id.is_valid());
    }
}
