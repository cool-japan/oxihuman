// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A manual synchronization barrier that tracks arrival of N named participants.

use std::collections::HashSet;

/// State of the barrier.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum BarrierState {
    Waiting,
    Released,
    Aborted(String),
}

/// A barrier that releases when all expected participants have arrived.
#[allow(dead_code)]
pub struct SyncBarrier {
    expected: HashSet<String>,
    arrived: HashSet<String>,
    state: BarrierState,
    generation: u64,
}

#[allow(dead_code)]
impl SyncBarrier {
    pub fn new() -> Self {
        Self {
            expected: HashSet::new(),
            arrived: HashSet::new(),
            state: BarrierState::Waiting,
            generation: 0,
        }
    }

    /// Register a participant that must arrive before release.
    pub fn register(&mut self, name: &str) {
        self.expected.insert(name.to_string());
    }

    /// Unregister a participant (before they have arrived).
    pub fn unregister(&mut self, name: &str) -> bool {
        self.expected.remove(name)
    }

    /// A participant arrives at the barrier.
    pub fn arrive(&mut self, name: &str) -> bool {
        if !self.expected.contains(name) {
            return false;
        }
        self.arrived.insert(name.to_string());
        if self.arrived == self.expected {
            self.state = BarrierState::Released;
        }
        true
    }

    /// Returns true if all expected participants have arrived.
    pub fn is_released(&self) -> bool {
        self.state == BarrierState::Released
    }

    /// Abort the barrier with a reason.
    pub fn abort(&mut self, reason: &str) {
        self.state = BarrierState::Aborted(reason.to_string());
    }

    /// Reset the barrier for reuse, incrementing generation.
    pub fn reset(&mut self) {
        self.arrived.clear();
        self.state = BarrierState::Waiting;
        self.generation += 1;
    }

    pub fn expected_count(&self) -> usize {
        self.expected.len()
    }

    pub fn arrived_count(&self) -> usize {
        self.arrived.len()
    }

    pub fn remaining_count(&self) -> usize {
        self.expected.len().saturating_sub(self.arrived.len())
    }

    pub fn state(&self) -> &BarrierState {
        &self.state
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn has_arrived(&self, name: &str) -> bool {
        self.arrived.contains(name)
    }

    pub fn missing_participants(&self) -> Vec<String> {
        let mut missing: Vec<String> = self.expected.difference(&self.arrived).cloned().collect();
        missing.sort();
        missing
    }
}

impl Default for SyncBarrier {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_sync_barrier() -> SyncBarrier {
    SyncBarrier::new()
}

pub fn barrier_register(b: &mut SyncBarrier, name: &str) {
    b.register(name);
}

pub fn barrier_arrive(b: &mut SyncBarrier, name: &str) -> bool {
    b.arrive(name)
}

pub fn barrier_is_released(b: &SyncBarrier) -> bool {
    b.is_released()
}

pub fn barrier_reset(b: &mut SyncBarrier) {
    b.reset();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_barrier_starts_waiting() {
        let b = new_sync_barrier();
        // no participants registered → state is Waiting until explicitly released
        assert_eq!(b.state(), &BarrierState::Waiting);
        assert_eq!(b.expected_count(), 0);
    }

    #[test]
    fn single_participant() {
        let mut b = new_sync_barrier();
        barrier_register(&mut b, "worker");
        assert!(!barrier_is_released(&b));
        barrier_arrive(&mut b, "worker");
        assert!(barrier_is_released(&b));
    }

    #[test]
    fn two_participants() {
        let mut b = new_sync_barrier();
        barrier_register(&mut b, "a");
        barrier_register(&mut b, "b");
        barrier_arrive(&mut b, "a");
        assert!(!barrier_is_released(&b));
        barrier_arrive(&mut b, "b");
        assert!(barrier_is_released(&b));
    }

    #[test]
    fn unknown_participant_ignored() {
        let mut b = new_sync_barrier();
        barrier_register(&mut b, "known");
        assert!(!barrier_arrive(&mut b, "unknown"));
    }

    #[test]
    fn reset_clears_arrivals() {
        let mut b = new_sync_barrier();
        barrier_register(&mut b, "x");
        barrier_arrive(&mut b, "x");
        barrier_reset(&mut b);
        assert!(!barrier_is_released(&b));
        assert_eq!(b.generation(), 1);
    }

    #[test]
    fn abort_sets_aborted_state() {
        let mut b = new_sync_barrier();
        b.abort("timeout");
        assert!(matches!(b.state(), BarrierState::Aborted(_)));
    }

    #[test]
    fn remaining_count() {
        let mut b = new_sync_barrier();
        barrier_register(&mut b, "a");
        barrier_register(&mut b, "b");
        barrier_arrive(&mut b, "a");
        assert_eq!(b.remaining_count(), 1);
    }

    #[test]
    fn missing_participants_sorted() {
        let mut b = new_sync_barrier();
        barrier_register(&mut b, "z");
        barrier_register(&mut b, "a");
        let missing = b.missing_participants();
        assert_eq!(missing, vec!["a".to_string(), "z".to_string()]);
    }

    #[test]
    fn has_arrived() {
        let mut b = new_sync_barrier();
        barrier_register(&mut b, "p");
        barrier_arrive(&mut b, "p");
        assert!(b.has_arrived("p"));
        assert!(!b.has_arrived("q"));
    }

    #[test]
    fn unregister_removes_participant() {
        let mut b = new_sync_barrier();
        barrier_register(&mut b, "x");
        b.unregister("x");
        assert_eq!(b.expected_count(), 0);
    }
}
