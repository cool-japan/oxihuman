// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A monotonically increasing epoch counter for versioning and change detection.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpochCounter {
    value: u64,
}

#[allow(dead_code)]
impl EpochCounter {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn from_value(value: u64) -> Self {
        Self { value }
    }

    pub fn advance(&mut self) -> u64 {
        self.value += 1;
        self.value
    }

    pub fn current(&self) -> u64 {
        self.value
    }

    pub fn is_newer_than(&self, other: &EpochCounter) -> bool {
        self.value > other.value
    }

    pub fn is_same_epoch(&self, other: &EpochCounter) -> bool {
        self.value == other.value
    }

    pub fn snapshot(&self) -> EpochSnapshot {
        EpochSnapshot {
            epoch: self.value,
        }
    }

    pub fn has_changed_since(&self, snapshot: &EpochSnapshot) -> bool {
        self.value > snapshot.epoch
    }

    pub fn reset(&mut self) {
        self.value = 0;
    }

    pub fn advance_by(&mut self, n: u64) -> u64 {
        self.value += n;
        self.value
    }
}

impl Default for EpochCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// An immutable snapshot of an epoch value for comparison.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpochSnapshot {
    pub epoch: u64,
}

#[allow(dead_code)]
impl EpochSnapshot {
    pub fn new(epoch: u64) -> Self {
        Self { epoch }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_starts_at_zero() {
        let c = EpochCounter::new();
        assert_eq!(c.current(), 0);
    }

    #[test]
    fn test_advance() {
        let mut c = EpochCounter::new();
        assert_eq!(c.advance(), 1);
        assert_eq!(c.advance(), 2);
    }

    #[test]
    fn test_is_newer_than() {
        let mut a = EpochCounter::new();
        let b = EpochCounter::new();
        a.advance();
        assert!(a.is_newer_than(&b));
        assert!(!b.is_newer_than(&a));
    }

    #[test]
    fn test_is_same_epoch() {
        let a = EpochCounter::new();
        let b = EpochCounter::new();
        assert!(a.is_same_epoch(&b));
    }

    #[test]
    fn test_snapshot() {
        let mut c = EpochCounter::new();
        c.advance();
        let snap = c.snapshot();
        assert!(!c.has_changed_since(&snap));
        c.advance();
        assert!(c.has_changed_since(&snap));
    }

    #[test]
    fn test_reset() {
        let mut c = EpochCounter::new();
        c.advance();
        c.advance();
        c.reset();
        assert_eq!(c.current(), 0);
    }

    #[test]
    fn test_from_value() {
        let c = EpochCounter::from_value(42);
        assert_eq!(c.current(), 42);
    }

    #[test]
    fn test_advance_by() {
        let mut c = EpochCounter::new();
        assert_eq!(c.advance_by(5), 5);
        assert_eq!(c.current(), 5);
    }

    #[test]
    fn test_default() {
        let c = EpochCounter::default();
        assert_eq!(c.current(), 0);
    }

    #[test]
    fn test_epoch_snapshot() {
        let snap = EpochSnapshot::new(10);
        assert_eq!(snap.epoch, 10);
    }
}
