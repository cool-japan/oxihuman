// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct ObserverEventBus {
    events: Vec<(String, String)>,
}

pub fn new_event_bus_observer() -> ObserverEventBus {
    ObserverEventBus { events: Vec::new() }
}

pub fn bus_emit(b: &mut ObserverEventBus, event_type: &str, payload: &str) {
    b.events.push((event_type.to_string(), payload.to_string()));
}

pub fn bus_events_of_type<'a>(b: &'a ObserverEventBus, event_type: &str) -> Vec<&'a str> {
    b.events
        .iter()
        .filter(|(t, _)| t == event_type)
        .map(|(_, p)| p.as_str())
        .collect()
}

pub fn bus_event_count(b: &ObserverEventBus) -> usize {
    b.events.len()
}

pub fn bus_clear(b: &mut ObserverEventBus) {
    b.events.clear();
}

pub fn bus_has_event_type(b: &ObserverEventBus, t: &str) -> bool {
    b.events.iter().any(|(et, _)| et == t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_and_count() {
        /* emit events and count them */
        let mut b = new_event_bus_observer();
        bus_emit(&mut b, "click", "button1");
        bus_emit(&mut b, "hover", "button2");
        assert_eq!(bus_event_count(&b), 2);
    }

    #[test]
    fn test_events_of_type() {
        /* filter events by type */
        let mut b = new_event_bus_observer();
        bus_emit(&mut b, "click", "a");
        bus_emit(&mut b, "click", "b");
        bus_emit(&mut b, "hover", "c");
        let clicks = bus_events_of_type(&b, "click");
        assert_eq!(clicks.len(), 2);
        assert!(clicks.contains(&"a"));
    }

    #[test]
    fn test_has_event_type() {
        /* check if event type exists */
        let mut b = new_event_bus_observer();
        bus_emit(&mut b, "keydown", "enter");
        assert!(bus_has_event_type(&b, "keydown"));
        assert!(!bus_has_event_type(&b, "keyup"));
    }

    #[test]
    fn test_clear() {
        /* clear removes all events */
        let mut b = new_event_bus_observer();
        bus_emit(&mut b, "x", "y");
        bus_clear(&mut b);
        assert_eq!(bus_event_count(&b), 0);
    }

    #[test]
    fn test_empty_bus() {
        /* new bus has no events */
        let b = new_event_bus_observer();
        assert_eq!(bus_event_count(&b), 0);
    }

    #[test]
    fn test_events_of_type_empty() {
        /* no matching events returns empty */
        let b = new_event_bus_observer();
        let r = bus_events_of_type(&b, "click");
        assert!(r.is_empty());
    }

    #[test]
    fn test_multiple_types() {
        /* multiple event types tracked correctly */
        let mut b = new_event_bus_observer();
        for _ in 0..3 {
            bus_emit(&mut b, "A", "p");
        }
        bus_emit(&mut b, "B", "q");
        assert_eq!(bus_events_of_type(&b, "A").len(), 3);
        assert_eq!(bus_events_of_type(&b, "B").len(), 1);
    }
}
