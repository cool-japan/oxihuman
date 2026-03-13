#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Signal-slot pattern for decoupled event handling.

/// An opaque slot identifier.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotId(pub u64);

/// A signal that can have connected slots.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Signal {
    name: String,
    slots: Vec<SlotId>,
    next_id: u64,
    emit_count: u64,
}

#[allow(dead_code)]
pub fn new_signal(name: &str) -> Signal {
    Signal {
        name: name.to_string(),
        slots: Vec::new(),
        next_id: 1,
        emit_count: 0,
    }
}

#[allow(dead_code)]
pub fn connect_slot(signal: &mut Signal) -> SlotId {
    let id = SlotId(signal.next_id);
    signal.next_id += 1;
    signal.slots.push(id);
    id
}

#[allow(dead_code)]
pub fn disconnect_slot(signal: &mut Signal, slot: SlotId) -> bool {
    let before = signal.slots.len();
    signal.slots.retain(|s| *s != slot);
    signal.slots.len() < before
}

#[allow(dead_code)]
pub fn emit_signal(signal: &mut Signal) -> usize {
    signal.emit_count += 1;
    signal.slots.len()
}

#[allow(dead_code)]
pub fn slot_count(signal: &Signal) -> usize {
    signal.slots.len()
}

#[allow(dead_code)]
pub fn signal_is_empty(signal: &Signal) -> bool {
    signal.slots.is_empty()
}

#[allow(dead_code)]
pub fn clear_signal(signal: &mut Signal) {
    signal.slots.clear();
}

#[allow(dead_code)]
pub fn signal_name(signal: &Signal) -> &str {
    &signal.name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let s = new_signal("clicked");
        assert_eq!(signal_name(&s), "clicked");
        assert!(signal_is_empty(&s));
    }

    #[test]
    fn test_connect() {
        let mut s = new_signal("test");
        let id = connect_slot(&mut s);
        assert_eq!(id.0, 1);
        assert_eq!(slot_count(&s), 1);
    }

    #[test]
    fn test_disconnect() {
        let mut s = new_signal("test");
        let id = connect_slot(&mut s);
        assert!(disconnect_slot(&mut s, id));
        assert!(signal_is_empty(&s));
    }

    #[test]
    fn test_disconnect_missing() {
        let mut s = new_signal("test");
        assert!(!disconnect_slot(&mut s, SlotId(999)));
    }

    #[test]
    fn test_emit() {
        let mut s = new_signal("test");
        connect_slot(&mut s);
        connect_slot(&mut s);
        assert_eq!(emit_signal(&mut s), 2);
    }

    #[test]
    fn test_clear() {
        let mut s = new_signal("test");
        connect_slot(&mut s);
        clear_signal(&mut s);
        assert_eq!(slot_count(&s), 0);
    }

    #[test]
    fn test_unique_ids() {
        let mut s = new_signal("test");
        let a = connect_slot(&mut s);
        let b = connect_slot(&mut s);
        assert_ne!(a, b);
    }

    #[test]
    fn test_slot_count() {
        let mut s = new_signal("test");
        connect_slot(&mut s);
        connect_slot(&mut s);
        connect_slot(&mut s);
        assert_eq!(slot_count(&s), 3);
    }

    #[test]
    fn test_emit_empty() {
        let mut s = new_signal("test");
        assert_eq!(emit_signal(&mut s), 0);
    }

    #[test]
    fn test_name() {
        let s = new_signal("my_signal");
        assert_eq!(signal_name(&s), "my_signal");
    }
}
