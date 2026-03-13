// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct AggregateRoot {
    pub id: u64,
    pub version: u64,
    pub pending_events: Vec<String>,
}

pub fn new_aggregate_root(id: u64) -> AggregateRoot {
    AggregateRoot {
        id,
        version: 0,
        pending_events: Vec::new(),
    }
}

pub fn aggregate_apply_event(a: &mut AggregateRoot, event: &str) {
    a.pending_events.push(event.to_string());
    a.version += 1;
}

pub fn aggregate_pending_events(a: &AggregateRoot) -> &[String] {
    &a.pending_events
}

pub fn aggregate_clear_events(a: &mut AggregateRoot) {
    a.pending_events.clear();
}

pub fn aggregate_version(a: &AggregateRoot) -> u64 {
    a.version
}

pub fn aggregate_increment_version(a: &mut AggregateRoot) {
    a.version += 1;
}

pub fn aggregate_id(a: &AggregateRoot) -> u64 {
    a.id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_aggregate() {
        /* new aggregate has version 0 and no events */
        let a = new_aggregate_root(1);
        assert_eq!(aggregate_id(&a), 1);
        assert_eq!(aggregate_version(&a), 0);
        assert!(aggregate_pending_events(&a).is_empty());
    }

    #[test]
    fn test_apply_event() {
        /* applying event adds to pending and increments version */
        let mut a = new_aggregate_root(1);
        aggregate_apply_event(&mut a, "UserCreated");
        assert_eq!(aggregate_version(&a), 1);
        assert_eq!(aggregate_pending_events(&a).len(), 1);
    }

    #[test]
    fn test_clear_events() {
        /* clear removes pending events */
        let mut a = new_aggregate_root(1);
        aggregate_apply_event(&mut a, "ev1");
        aggregate_clear_events(&mut a);
        assert!(aggregate_pending_events(&a).is_empty());
    }

    #[test]
    fn test_increment_version() {
        /* explicit version increment */
        let mut a = new_aggregate_root(1);
        aggregate_increment_version(&mut a);
        assert_eq!(aggregate_version(&a), 1);
    }

    #[test]
    fn test_multiple_events() {
        /* multiple events accumulate */
        let mut a = new_aggregate_root(2);
        aggregate_apply_event(&mut a, "ev1");
        aggregate_apply_event(&mut a, "ev2");
        aggregate_apply_event(&mut a, "ev3");
        assert_eq!(aggregate_version(&a), 3);
        assert_eq!(aggregate_pending_events(&a).len(), 3);
    }

    #[test]
    fn test_version_preserved_after_clear() {
        /* version not reset when events cleared */
        let mut a = new_aggregate_root(1);
        aggregate_apply_event(&mut a, "e");
        aggregate_clear_events(&mut a);
        assert_eq!(aggregate_version(&a), 1);
    }

    #[test]
    fn test_aggregate_id() {
        /* id stored correctly */
        let a = new_aggregate_root(42);
        assert_eq!(aggregate_id(&a), 42);
    }
}
