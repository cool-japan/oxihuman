// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct DomainEvent {
    pub id: u64,
    pub aggregate_id: u64,
    pub event_type: String,
    pub payload: String,
    pub timestamp_ms: u64,
}

pub fn new_domain_event(
    id: u64,
    agg_id: u64,
    event_type: &str,
    payload: &str,
    ts: u64,
) -> DomainEvent {
    DomainEvent {
        id,
        aggregate_id: agg_id,
        event_type: event_type.to_string(),
        payload: payload.to_string(),
        timestamp_ms: ts,
    }
}

pub fn event_to_json(e: &DomainEvent) -> String {
    format!(
        "{{\"id\":{},\"aggregate_id\":{},\"event_type\":\"{}\",\"payload\":\"{}\",\"timestamp_ms\":{}}}",
        e.id, e.aggregate_id, e.event_type, e.payload, e.timestamp_ms
    )
}

pub fn event_is_type(e: &DomainEvent, t: &str) -> bool {
    e.event_type == t
}

pub fn events_by_aggregate(events: &[DomainEvent], agg_id: u64) -> Vec<&DomainEvent> {
    events.iter().filter(|e| e.aggregate_id == agg_id).collect()
}

pub fn events_after(events: &[DomainEvent], ts: u64) -> Vec<&DomainEvent> {
    events.iter().filter(|e| e.timestamp_ms > ts).collect()
}

pub fn events_of_type<'a>(events: &'a [DomainEvent], t: &str) -> Vec<&'a DomainEvent> {
    events.iter().filter(|e| e.event_type == t).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_domain_event() {
        /* create domain event with all fields */
        let e = new_domain_event(1, 10, "Created", "{}", 1000);
        assert_eq!(e.id, 1);
        assert_eq!(e.aggregate_id, 10);
        assert!(event_is_type(&e, "Created"));
    }

    #[test]
    fn test_event_to_json() {
        /* serializes to JSON string */
        let e = new_domain_event(1, 2, "Test", "data", 500);
        let j = event_to_json(&e);
        assert!(j.contains("\"id\":1"));
        assert!(j.contains("\"event_type\":\"Test\""));
    }

    #[test]
    fn test_events_by_aggregate() {
        /* filter by aggregate_id */
        let events = vec![
            new_domain_event(1, 10, "A", "", 100),
            new_domain_event(2, 20, "B", "", 200),
            new_domain_event(3, 10, "C", "", 300),
        ];
        let r = events_by_aggregate(&events, 10);
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_events_after() {
        /* filter by timestamp */
        let events = vec![
            new_domain_event(1, 1, "A", "", 100),
            new_domain_event(2, 1, "B", "", 200),
            new_domain_event(3, 1, "C", "", 300),
        ];
        let r = events_after(&events, 150);
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_events_of_type() {
        /* filter by event type */
        let events = vec![
            new_domain_event(1, 1, "Created", "", 100),
            new_domain_event(2, 1, "Updated", "", 200),
            new_domain_event(3, 1, "Created", "", 300),
        ];
        let r = events_of_type(&events, "Created");
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_event_is_type_false() {
        /* wrong type returns false */
        let e = new_domain_event(1, 1, "A", "", 0);
        assert!(!event_is_type(&e, "B"));
    }

    #[test]
    fn test_events_empty() {
        /* empty slice returns empty results */
        let events: Vec<DomainEvent> = vec![];
        assert!(events_by_aggregate(&events, 1).is_empty());
    }
}
