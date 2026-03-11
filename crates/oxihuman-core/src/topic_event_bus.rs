//! Publish-subscribe event bus for decoupled component communication.
//!
//! This module provides a topic-based pub/sub event bus with priority queuing,
//! subscriber management, and JSON-serializable event records. It is distinct
//! from the simpler kind-based `event_bus` module.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Priority level for an event.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    /// Processed before normal and low events.
    High,
    /// Default priority.
    Normal,
    /// Processed last.
    Low,
}

/// A record of a single published event stored in the bus queue.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct EventRecord {
    /// Unique sequential id assigned at publish time.
    pub id: u64,
    /// Topic string the event was published on.
    pub topic: String,
    /// JSON-encoded payload (may be `"null"` if empty).
    pub payload: String,
    /// Priority of this event.
    pub priority: EventPriority,
    /// Monotonic timestamp in milliseconds (caller-provided).
    pub timestamp_ms: u64,
}

/// Type alias for a subscriber handler id.
#[allow(dead_code)]
pub type SubscriberId = u64;

/// Type alias for the pending events list.
#[allow(dead_code)]
pub type PendingEvents = Vec<EventRecord>;

/// Publish-subscribe event bus with topic-based routing and priority support.
#[allow(dead_code)]
pub struct EventBusTopic {
    /// Pending (unprocessed) events, in insertion order.
    pub pending: Vec<EventRecord>,
    /// All dispatched events, oldest first.
    pub dispatched: Vec<EventRecord>,
    /// Subscribers: topic → list of subscriber ids.
    pub subscribers: HashMap<String, Vec<SubscriberId>>,
    /// Next subscriber id to assign.
    pub next_sub_id: SubscriberId,
    /// Next event id to assign.
    pub next_event_id: u64,
    /// Timestamp of the most recently published event (0 if none).
    pub last_event_ts: u64,
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Create a new, empty event bus.
#[allow(dead_code)]
pub fn new_event_bus() -> EventBusTopic {
    EventBusTopic {
        pending: Vec::new(),
        dispatched: Vec::new(),
        subscribers: HashMap::new(),
        next_sub_id: 1,
        next_event_id: 1,
        last_event_ts: 0,
    }
}

// ---------------------------------------------------------------------------
// Publishing
// ---------------------------------------------------------------------------

/// Publish an event with Normal priority on the given topic.
/// Returns the assigned event id.
#[allow(dead_code)]
pub fn publish(bus: &mut EventBusTopic, topic: &str, payload: &str, timestamp_ms: u64) -> u64 {
    publish_priority(bus, topic, payload, EventPriority::Normal, timestamp_ms)
}

/// Publish an event with an explicit priority on the given topic.
/// Returns the assigned event id.
#[allow(dead_code)]
pub fn publish_priority(
    bus: &mut EventBusTopic,
    topic: &str,
    payload: &str,
    priority: EventPriority,
    timestamp_ms: u64,
) -> u64 {
    let id = bus.next_event_id;
    bus.next_event_id += 1;
    bus.last_event_ts = timestamp_ms;
    let record = EventRecord {
        id,
        topic: topic.to_string(),
        payload: payload.to_string(),
        priority,
        timestamp_ms,
    };
    bus.pending.push(record);
    id
}

// ---------------------------------------------------------------------------
// Subscription management
// ---------------------------------------------------------------------------

/// Register a subscriber for the given topic.
/// Returns the new subscriber id.
#[allow(dead_code)]
pub fn subscribe(bus: &mut EventBusTopic, topic: &str) -> SubscriberId {
    let id = bus.next_sub_id;
    bus.next_sub_id += 1;
    bus.subscribers
        .entry(topic.to_string())
        .or_default()
        .push(id);
    id
}

/// Remove a subscriber by id from the given topic.
/// Returns `true` if the subscriber was found and removed.
#[allow(dead_code)]
pub fn unsubscribe(bus: &mut EventBusTopic, topic: &str, sub_id: SubscriberId) -> bool {
    if let Some(subs) = bus.subscribers.get_mut(topic) {
        if let Some(pos) = subs.iter().position(|&s| s == sub_id) {
            subs.remove(pos);
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

/// Dispatch all pending events in priority order (High → Normal → Low).
/// Moves them from `pending` to `dispatched`.
/// Returns the number of events dispatched.
#[allow(dead_code)]
pub fn dispatch_pending(bus: &mut EventBusTopic) -> usize {
    // Sort pending by priority (High first = lowest enum ordinal).
    bus.pending.sort_by_key(|e| e.priority);
    let count = bus.pending.len();
    let drained: Vec<EventRecord> = bus.pending.drain(..).collect();
    bus.dispatched.extend(drained);
    count
}

/// Drain all pending events for a specific topic, returning them.
#[allow(dead_code)]
pub fn drain_topic(bus: &mut EventBusTopic, topic: &str) -> PendingEvents {
    let mut drained = Vec::new();
    let mut remaining = Vec::new();
    for ev in bus.pending.drain(..) {
        if ev.topic == topic {
            drained.push(ev);
        } else {
            remaining.push(ev);
        }
    }
    bus.pending = remaining;
    drained
}

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

/// Return the number of pending (not yet dispatched) events.
#[allow(dead_code)]
pub fn pending_count(bus: &EventBusTopic) -> usize {
    bus.pending.len()
}

/// Return the number of subscribers for a given topic.
#[allow(dead_code)]
pub fn topic_subscriber_count(bus: &EventBusTopic, topic: &str) -> usize {
    bus.subscribers.get(topic).map_or(0, |v| v.len())
}

/// Return the total number of dispatched events since creation.
#[allow(dead_code)]
pub fn event_count_total(bus: &EventBusTopic) -> usize {
    bus.dispatched.len()
}

/// Return `true` if at least one subscriber exists for the given topic.
#[allow(dead_code)]
pub fn has_subscribers(bus: &EventBusTopic, topic: &str) -> bool {
    bus.subscribers.get(topic).is_some_and(|v| !v.is_empty())
}

/// Return the timestamp of the most recently published event (0 if none).
#[allow(dead_code)]
pub fn last_event_time(bus: &EventBusTopic) -> u64 {
    bus.last_event_ts
}

// ---------------------------------------------------------------------------
// Clear
// ---------------------------------------------------------------------------

/// Clear pending events and dispatched history, keeping subscribers.
#[allow(dead_code)]
pub fn clear_event_bus(bus: &mut EventBusTopic) {
    bus.pending.clear();
    bus.dispatched.clear();
    bus.last_event_ts = 0;
}

// ---------------------------------------------------------------------------
// JSON serialisation (minimal, no external deps)
// ---------------------------------------------------------------------------

/// Serialise the entire bus state to a compact JSON string.
#[allow(dead_code)]
pub fn event_bus_to_json(bus: &EventBusTopic) -> String {
    let pending_json: Vec<String> = bus
        .pending
        .iter()
        .map(|e| {
            format!(
                r#"{{"id":{},"topic":"{}","priority":"{:?}","ts":{}}}"#,
                e.id, e.topic, e.priority, e.timestamp_ms
            )
        })
        .collect();

    let dispatched_json: Vec<String> = bus
        .dispatched
        .iter()
        .map(|e| {
            format!(
                r#"{{"id":{},"topic":"{}","priority":"{:?}","ts":{}}}"#,
                e.id, e.topic, e.priority, e.timestamp_ms
            )
        })
        .collect();

    format!(
        r#"{{"pending_count":{},"dispatched_count":{},"pending":[{}],"dispatched":[{}]}}"#,
        bus.pending.len(),
        bus.dispatched.len(),
        pending_json.join(","),
        dispatched_json.join(",")
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_event_bus_empty() {
        let bus = new_event_bus();
        assert_eq!(pending_count(&bus), 0);
        assert_eq!(event_count_total(&bus), 0);
    }

    #[test]
    fn test_publish_increments_pending() {
        let mut bus = new_event_bus();
        publish(&mut bus, "topic_a", "null", 0);
        publish(&mut bus, "topic_a", "null", 1);
        assert_eq!(pending_count(&bus), 2);
    }

    #[test]
    fn test_publish_returns_sequential_ids() {
        let mut bus = new_event_bus();
        let id1 = publish(&mut bus, "t", "null", 0);
        let id2 = publish(&mut bus, "t", "null", 0);
        assert_eq!(id2, id1 + 1);
    }

    #[test]
    fn test_subscribe_returns_unique_ids() {
        let mut bus = new_event_bus();
        let s1 = subscribe(&mut bus, "topic_x");
        let s2 = subscribe(&mut bus, "topic_x");
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_topic_subscriber_count() {
        let mut bus = new_event_bus();
        subscribe(&mut bus, "topic_a");
        subscribe(&mut bus, "topic_a");
        subscribe(&mut bus, "topic_b");
        assert_eq!(topic_subscriber_count(&bus, "topic_a"), 2);
        assert_eq!(topic_subscriber_count(&bus, "topic_b"), 1);
    }

    #[test]
    fn test_has_subscribers_false_on_empty() {
        let bus = new_event_bus();
        assert!(!has_subscribers(&bus, "no_topic"));
    }

    #[test]
    fn test_has_subscribers_true_after_subscribe() {
        let mut bus = new_event_bus();
        subscribe(&mut bus, "events");
        assert!(has_subscribers(&bus, "events"));
    }

    #[test]
    fn test_unsubscribe_removes_subscriber() {
        let mut bus = new_event_bus();
        let sid = subscribe(&mut bus, "topic");
        assert!(unsubscribe(&mut bus, "topic", sid));
        assert!(!has_subscribers(&bus, "topic"));
    }

    #[test]
    fn test_unsubscribe_returns_false_for_unknown() {
        let mut bus = new_event_bus();
        assert!(!unsubscribe(&mut bus, "unknown", 999));
    }

    #[test]
    fn test_dispatch_pending_moves_to_dispatched() {
        let mut bus = new_event_bus();
        publish(&mut bus, "t", "null", 0);
        publish(&mut bus, "t", "null", 1);
        let n = dispatch_pending(&mut bus);
        assert_eq!(n, 2);
        assert_eq!(pending_count(&bus), 0);
        assert_eq!(event_count_total(&bus), 2);
    }

    #[test]
    fn test_dispatch_priority_ordering() {
        let mut bus = new_event_bus();
        publish_priority(&mut bus, "t", "low", EventPriority::Low, 0);
        publish_priority(&mut bus, "t", "high", EventPriority::High, 1);
        publish_priority(&mut bus, "t", "normal", EventPriority::Normal, 2);
        dispatch_pending(&mut bus);
        // After dispatch, dispatched order: High, Normal, Low
        assert_eq!(bus.dispatched[0].payload, "high");
        assert_eq!(bus.dispatched[1].payload, "normal");
        assert_eq!(bus.dispatched[2].payload, "low");
    }

    #[test]
    fn test_clear_event_bus_resets_counts() {
        let mut bus = new_event_bus();
        publish(&mut bus, "t", "null", 5);
        dispatch_pending(&mut bus);
        clear_event_bus(&mut bus);
        assert_eq!(pending_count(&bus), 0);
        assert_eq!(event_count_total(&bus), 0);
        assert_eq!(last_event_time(&bus), 0);
    }

    #[test]
    fn test_last_event_time_updated() {
        let mut bus = new_event_bus();
        publish(&mut bus, "t", "null", 42);
        assert_eq!(last_event_time(&bus), 42);
    }

    #[test]
    fn test_drain_topic_removes_only_matching() {
        let mut bus = new_event_bus();
        publish(&mut bus, "alpha", "a1", 0);
        publish(&mut bus, "beta", "b1", 1);
        publish(&mut bus, "alpha", "a2", 2);
        let drained = drain_topic(&mut bus, "alpha");
        assert_eq!(drained.len(), 2);
        assert_eq!(pending_count(&bus), 1);
        assert_eq!(bus.pending[0].topic, "beta");
    }

    #[test]
    fn test_event_bus_to_json_contains_counts() {
        let mut bus = new_event_bus();
        publish(&mut bus, "t", "null", 0);
        let json = event_bus_to_json(&bus);
        assert!(json.contains("pending_count"));
        assert!(json.contains("dispatched_count"));
    }

    #[test]
    fn test_publish_priority_high() {
        let mut bus = new_event_bus();
        let id = publish_priority(&mut bus, "t", "hi", EventPriority::High, 99);
        assert_eq!(bus.pending[0].priority, EventPriority::High);
        assert_eq!(bus.pending[0].id, id);
    }

    #[test]
    fn test_topic_subscriber_count_zero_for_unknown() {
        let bus = new_event_bus();
        assert_eq!(topic_subscriber_count(&bus, "ghost"), 0);
    }

    #[test]
    fn test_multiple_topics_independent() {
        let mut bus = new_event_bus();
        subscribe(&mut bus, "a");
        subscribe(&mut bus, "b");
        assert_eq!(topic_subscriber_count(&bus, "a"), 1);
        assert_eq!(topic_subscriber_count(&bus, "b"), 1);
        assert_eq!(topic_subscriber_count(&bus, "c"), 0);
    }
}
