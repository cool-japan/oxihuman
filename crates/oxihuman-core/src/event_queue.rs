//! Ordered event queue with priority and timestamp ordering.

/// Priority level for queued events (higher values are more urgent).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    /// Background / non-urgent events.
    Low,
    /// Default priority.
    Normal,
    /// Important events that should be processed soon.
    High,
    /// Must be processed immediately.
    Critical,
}

/// A single event stored in the queue.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct QueuedEvent {
    /// Application-defined event identifier.
    pub event_id: u64,
    /// Priority of this event.
    pub priority: EventPriority,
    /// Monotonic timestamp (e.g. seconds since startup).
    pub timestamp: f64,
    /// Sequential index for stable FIFO ordering within same priority+timestamp.
    pub(crate) seq: u64,
}

/// Configuration for the event queue.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EventQueueConfig {
    /// Maximum number of events the queue will hold (0 = unlimited).
    pub capacity: usize,
}

/// Ordered event queue backed by a `Vec` kept sorted on insertion.
#[allow(dead_code)]
pub struct EventQueue {
    /// Active configuration.
    pub config: EventQueueConfig,
    /// Internal storage, sorted: highest priority first, then earliest timestamp.
    events: Vec<QueuedEvent>,
    /// Monotonically incrementing sequence counter.
    next_seq: u64,
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Comparison key: higher priority first; on tie, earlier timestamp first;
/// on tie, lower seq first (FIFO).
fn event_sort_key(e: &QueuedEvent) -> (u8, u64, u64) {
    // Negate priority so that Critical (3) sorts before Low (0).
    let prio_inv = u8::MAX - event_priority_value(e.priority);
    // Use raw bits for f64 ordering; timestamps are expected to be non-negative.
    let ts_bits = e.timestamp.to_bits();
    (prio_inv, ts_bits, e.seq)
}

// ── public API ───────────────────────────────────────────────────────────────

/// Return a default `EventQueueConfig` with no capacity limit.
#[allow(dead_code)]
pub fn default_event_queue_config() -> EventQueueConfig {
    EventQueueConfig { capacity: 0 }
}

/// Construct a new, empty `EventQueue`.
#[allow(dead_code)]
pub fn new_event_queue(cfg: &EventQueueConfig) -> EventQueue {
    EventQueue {
        config: cfg.clone(),
        events: Vec::new(),
        next_seq: 0,
    }
}

/// Insert an event into the queue, maintaining sort order.
/// If capacity is set and full, the lowest-priority event is dropped.
#[allow(dead_code)]
pub fn enqueue_event(
    q: &mut EventQueue,
    event_id: u64,
    priority: EventPriority,
    timestamp: f64,
) {
    let seq = q.next_seq;
    q.next_seq += 1;
    let ev = QueuedEvent { event_id, priority, timestamp, seq };
    // Binary-search insertion to keep the vec sorted.
    let key = event_sort_key(&ev);
    let pos = q.events.partition_point(|e| event_sort_key(e) <= key);
    q.events.insert(pos, ev);
    // Enforce capacity limit by removing the last (lowest priority) element.
    if q.config.capacity > 0 && q.events.len() > q.config.capacity {
        q.events.pop();
    }
}

/// Remove and return the highest-priority event (front of queue).
#[allow(dead_code)]
pub fn dequeue_event(q: &mut EventQueue) -> Option<QueuedEvent> {
    if q.events.is_empty() {
        None
    } else {
        Some(q.events.remove(0))
    }
}

/// Peek at the highest-priority event without removing it.
#[allow(dead_code)]
pub fn peek_event(q: &EventQueue) -> Option<&QueuedEvent> {
    q.events.first()
}

/// Return the number of events currently in the queue.
#[allow(dead_code)]
pub fn event_queue_len(q: &EventQueue) -> usize {
    q.events.len()
}

/// Remove all events from the queue.
#[allow(dead_code)]
pub fn clear_event_queue(q: &mut EventQueue) {
    q.events.clear();
}

/// Drain all events with the given priority, returning them in order.
#[allow(dead_code)]
pub fn drain_events_by_priority(
    q: &mut EventQueue,
    priority: EventPriority,
) -> Vec<QueuedEvent> {
    let mut drained = Vec::new();
    let mut i = 0;
    while i < q.events.len() {
        if q.events[i].priority == priority {
            drained.push(q.events.remove(i));
        } else {
            i += 1;
        }
    }
    drained
}

/// Return the numeric value of a priority level (higher = more urgent).
#[allow(dead_code)]
pub fn event_priority_value(p: EventPriority) -> u8 {
    match p {
        EventPriority::Low => 0,
        EventPriority::Normal => 1,
        EventPriority::High => 2,
        EventPriority::Critical => 3,
    }
}

/// Return `true` when the queue contains no events.
#[allow(dead_code)]
pub fn event_queue_is_empty(q: &EventQueue) -> bool {
    q.events.is_empty()
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_queue() {
        let cfg = default_event_queue_config();
        let q = new_event_queue(&cfg);
        assert!(event_queue_is_empty(&q));
        assert_eq!(event_queue_len(&q), 0);
    }

    #[test]
    fn test_enqueue_dequeue_order() {
        let cfg = default_event_queue_config();
        let mut q = new_event_queue(&cfg);
        enqueue_event(&mut q, 1, EventPriority::Low, 1.0);
        enqueue_event(&mut q, 2, EventPriority::Critical, 2.0);
        enqueue_event(&mut q, 3, EventPriority::Normal, 0.5);
        // Critical should come out first.
        let first = dequeue_event(&mut q).expect("should succeed");
        assert_eq!(first.priority, EventPriority::Critical);
    }

    #[test]
    fn test_peek_does_not_remove() {
        let cfg = default_event_queue_config();
        let mut q = new_event_queue(&cfg);
        enqueue_event(&mut q, 42, EventPriority::High, 0.0);
        let _ = peek_event(&q);
        assert_eq!(event_queue_len(&q), 1);
    }

    #[test]
    fn test_clear_event_queue() {
        let cfg = default_event_queue_config();
        let mut q = new_event_queue(&cfg);
        enqueue_event(&mut q, 1, EventPriority::Normal, 0.0);
        enqueue_event(&mut q, 2, EventPriority::High, 0.0);
        clear_event_queue(&mut q);
        assert!(event_queue_is_empty(&q));
    }

    #[test]
    fn test_drain_events_by_priority() {
        let cfg = default_event_queue_config();
        let mut q = new_event_queue(&cfg);
        enqueue_event(&mut q, 1, EventPriority::Normal, 0.0);
        enqueue_event(&mut q, 2, EventPriority::High, 1.0);
        enqueue_event(&mut q, 3, EventPriority::Normal, 2.0);
        let drained = drain_events_by_priority(&mut q, EventPriority::Normal);
        assert_eq!(drained.len(), 2);
        assert_eq!(event_queue_len(&q), 1);
    }

    #[test]
    fn test_priority_values_ordered() {
        assert!(event_priority_value(EventPriority::Low) < event_priority_value(EventPriority::Normal));
        assert!(event_priority_value(EventPriority::Normal) < event_priority_value(EventPriority::High));
        assert!(event_priority_value(EventPriority::High) < event_priority_value(EventPriority::Critical));
    }

    #[test]
    fn test_capacity_enforcement() {
        let cfg = EventQueueConfig { capacity: 2 };
        let mut q = new_event_queue(&cfg);
        enqueue_event(&mut q, 1, EventPriority::Low, 0.0);
        enqueue_event(&mut q, 2, EventPriority::High, 0.0);
        enqueue_event(&mut q, 3, EventPriority::Normal, 0.0);
        // Should not exceed capacity.
        assert!(event_queue_len(&q) <= 2);
    }

    #[test]
    fn test_fifo_within_same_priority_and_timestamp() {
        let cfg = default_event_queue_config();
        let mut q = new_event_queue(&cfg);
        enqueue_event(&mut q, 10, EventPriority::Normal, 0.0);
        enqueue_event(&mut q, 20, EventPriority::Normal, 0.0);
        let first = dequeue_event(&mut q).expect("should succeed");
        let second = dequeue_event(&mut q).expect("should succeed");
        // FIFO: event 10 was enqueued first.
        assert_eq!(first.event_id, 10);
        assert_eq!(second.event_id, 20);
    }
}
