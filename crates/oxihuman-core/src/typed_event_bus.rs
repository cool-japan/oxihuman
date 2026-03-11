// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A single typed event record.
#[allow(dead_code)]
pub struct TypedEventRecord {
    pub event_type: String,
    pub payload: String,
    pub timestamp: u64,
}

/// A type-safe, string-keyed event bus.
#[allow(dead_code)]
pub struct TypedEventBus {
    pub events: Vec<TypedEventRecord>,
    pub clock: u64,
}

/// Create a new empty `TypedEventBus`.
#[allow(dead_code)]
pub fn new_typed_event_bus() -> TypedEventBus {
    TypedEventBus {
        events: Vec::new(),
        clock: 0,
    }
}

/// Publish an event of `event_type` with the given `payload`.
#[allow(dead_code)]
pub fn publish(bus: &mut TypedEventBus, event_type: &str, payload: &str) {
    let ts = bus.clock;
    bus.clock += 1;
    bus.events.push(TypedEventRecord {
        event_type: event_type.to_string(),
        payload: payload.to_string(),
        timestamp: ts,
    });
}

/// Return all events of the given type.
#[allow(dead_code)]
pub fn events_of_type<'a>(bus: &'a TypedEventBus, event_type: &str) -> Vec<&'a TypedEventRecord> {
    bus.events
        .iter()
        .filter(|e| e.event_type == event_type)
        .collect()
}

/// Return the total number of events in the bus.
#[allow(dead_code)]
pub fn event_count(bus: &TypedEventBus) -> usize {
    bus.events.len()
}

/// Clear all events from the bus.
#[allow(dead_code)]
pub fn clear_events(bus: &mut TypedEventBus) {
    bus.events.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bus_is_empty() {
        let bus = new_typed_event_bus();
        assert_eq!(event_count(&bus), 0);
    }

    #[test]
    fn publish_increments_count() {
        let mut bus = new_typed_event_bus();
        publish(&mut bus, "click", "btn1");
        assert_eq!(event_count(&bus), 1);
    }

    #[test]
    fn events_of_type_filters_correctly() {
        let mut bus = new_typed_event_bus();
        publish(&mut bus, "click", "a");
        publish(&mut bus, "hover", "b");
        publish(&mut bus, "click", "c");
        let clicks = events_of_type(&bus, "click");
        assert_eq!(clicks.len(), 2);
    }

    #[test]
    fn events_of_type_unknown_returns_empty() {
        let mut bus = new_typed_event_bus();
        publish(&mut bus, "click", "x");
        assert!(events_of_type(&bus, "scroll").is_empty());
    }

    #[test]
    fn clear_events_empties_bus() {
        let mut bus = new_typed_event_bus();
        publish(&mut bus, "ev", "data");
        clear_events(&mut bus);
        assert_eq!(event_count(&bus), 0);
    }

    #[test]
    fn timestamps_are_monotone() {
        let mut bus = new_typed_event_bus();
        publish(&mut bus, "a", "");
        publish(&mut bus, "b", "");
        assert!(bus.events[1].timestamp > bus.events[0].timestamp);
    }

    #[test]
    fn payload_preserved() {
        let mut bus = new_typed_event_bus();
        publish(&mut bus, "msg", "hello world");
        assert_eq!(bus.events[0].payload, "hello world");
    }

    #[test]
    fn event_type_preserved() {
        let mut bus = new_typed_event_bus();
        publish(&mut bus, "my_type", "data");
        assert_eq!(bus.events[0].event_type, "my_type");
    }

    #[test]
    fn multiple_types_independent() {
        let mut bus = new_typed_event_bus();
        for i in 0..5 {
            publish(&mut bus, "alpha", &i.to_string());
            publish(&mut bus, "beta", &i.to_string());
        }
        assert_eq!(events_of_type(&bus, "alpha").len(), 5);
        assert_eq!(events_of_type(&bus, "beta").len(), 5);
    }

    #[test]
    fn clear_then_publish_works() {
        let mut bus = new_typed_event_bus();
        publish(&mut bus, "t", "old");
        clear_events(&mut bus);
        publish(&mut bus, "t", "new");
        assert_eq!(event_count(&bus), 1);
        assert_eq!(bus.events[0].payload, "new");
    }
}
