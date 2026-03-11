// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Simple synchronous event bus for plugin notifications.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Kind of event dispatched on the bus.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum EventKind {
    TargetLoaded,
    TargetUnloaded,
    ParamChanged,
    ExportStarted,
    ExportFinished,
    PluginRegistered,
    Error,
    Custom(String),
}

/// A single event with a JSON payload and wall-clock timestamp.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Event {
    pub kind: EventKind,
    /// JSON-encoded payload (may be `"null"` if empty).
    pub payload: String,
    /// Milliseconds since some epoch (caller-provided).
    pub timestamp_ms: u64,
}

/// Type alias for a boxed event handler.
pub type EventHandler = Box<dyn Fn(&Event) + Send + Sync>;

/// Synchronous event bus: stores handlers keyed by `EventKind` and a history of all
/// published events.
pub struct EventBus {
    handlers: Vec<(EventKind, EventHandler)>,
    history: Vec<Event>,
}

// ── impl EventBus ─────────────────────────────────────────────────────────────

impl EventBus {
    /// Create a new, empty event bus.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Register a handler for a specific event kind.
    #[allow(dead_code)]
    pub fn subscribe(&mut self, kind: EventKind, handler: EventHandler) {
        self.handlers.push((kind, handler));
    }

    /// Publish an event: invoke matching handlers then append to history.
    #[allow(dead_code)]
    pub fn publish(&mut self, event: Event) {
        for (kind, handler) in &self.handlers {
            if *kind == event.kind {
                handler(&event);
            }
        }
        self.history.push(event);
    }

    /// Access the full event history (oldest first).
    #[allow(dead_code)]
    pub fn history(&self) -> &[Event] {
        &self.history
    }

    /// Clear the event history.
    #[allow(dead_code)]
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Total number of registered handlers.
    #[allow(dead_code)]
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }

    /// Total number of events published so far.
    #[allow(dead_code)]
    pub fn event_count(&self) -> usize {
        self.history.len()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

// ── Factory helpers ───────────────────────────────────────────────────────────

/// Build a `ParamChanged` event with a JSON payload.
#[allow(dead_code)]
pub fn make_param_changed_event(name: &str, value: f32) -> Event {
    Event {
        kind: EventKind::ParamChanged,
        payload: format!(r#"{{"name":"{name}","value":{value}}}"#),
        timestamp_ms: 0,
    }
}

/// Build an `ExportStarted` event.
#[allow(dead_code)]
pub fn make_export_event(path: &str, format: &str) -> Event {
    Event {
        kind: EventKind::ExportStarted,
        payload: format!(r#"{{"path":"{path}","format":"{format}"}}"#),
        timestamp_ms: 0,
    }
}

/// Build an `Error` event.
#[allow(dead_code)]
pub fn make_error_event(msg: &str) -> Event {
    Event {
        kind: EventKind::Error,
        payload: format!(r#"{{"message":"{msg}"}}"#),
        timestamp_ms: 0,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn ts() -> u64 {
        0
    }

    #[test]
    fn test_subscribe_and_publish_triggers_handler() {
        let counter = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&counter);
        let mut bus = EventBus::new();
        bus.subscribe(
            EventKind::TargetLoaded,
            Box::new(move |_| {
                *c.lock().unwrap() += 1;
            }),
        );
        bus.publish(Event {
            kind: EventKind::TargetLoaded,
            payload: "null".to_string(),
            timestamp_ms: ts(),
        });
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_wrong_kind_does_not_trigger() {
        let counter = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&counter);
        let mut bus = EventBus::new();
        bus.subscribe(
            EventKind::ExportFinished,
            Box::new(move |_| {
                *c.lock().unwrap() += 1;
            }),
        );
        bus.publish(Event {
            kind: EventKind::TargetLoaded,
            payload: "null".to_string(),
            timestamp_ms: ts(),
        });
        assert_eq!(*counter.lock().unwrap(), 0);
    }

    #[test]
    fn test_history_grows() {
        let mut bus = EventBus::new();
        for i in 0..5 {
            bus.publish(Event {
                kind: EventKind::ParamChanged,
                payload: format!("{i}"),
                timestamp_ms: ts(),
            });
        }
        assert_eq!(bus.history().len(), 5);
    }

    #[test]
    fn test_clear_history_empties() {
        let mut bus = EventBus::new();
        bus.publish(Event {
            kind: EventKind::Error,
            payload: "null".to_string(),
            timestamp_ms: ts(),
        });
        assert!(!bus.history().is_empty());
        bus.clear_history();
        assert!(bus.history().is_empty());
    }

    #[test]
    fn test_handler_count() {
        let mut bus = EventBus::new();
        bus.subscribe(EventKind::TargetLoaded, Box::new(|_| {}));
        bus.subscribe(EventKind::TargetUnloaded, Box::new(|_| {}));
        assert_eq!(bus.handler_count(), 2);
    }

    #[test]
    fn test_event_count_matches_published() {
        let mut bus = EventBus::new();
        bus.publish(Event {
            kind: EventKind::ExportStarted,
            payload: "null".to_string(),
            timestamp_ms: ts(),
        });
        bus.publish(Event {
            kind: EventKind::ExportFinished,
            payload: "null".to_string(),
            timestamp_ms: ts(),
        });
        assert_eq!(bus.event_count(), 2);
    }

    #[test]
    fn test_multiple_handlers_same_kind() {
        let counter = Arc::new(Mutex::new(0u32));
        let c1 = Arc::clone(&counter);
        let c2 = Arc::clone(&counter);
        let mut bus = EventBus::new();
        bus.subscribe(
            EventKind::Error,
            Box::new(move |_| *c1.lock().unwrap() += 1),
        );
        bus.subscribe(
            EventKind::Error,
            Box::new(move |_| *c2.lock().unwrap() += 1),
        );
        bus.publish(Event {
            kind: EventKind::Error,
            payload: "null".to_string(),
            timestamp_ms: ts(),
        });
        assert_eq!(*counter.lock().unwrap(), 2);
    }

    #[test]
    fn test_custom_variant_matching() {
        let counter = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&counter);
        let mut bus = EventBus::new();
        bus.subscribe(
            EventKind::Custom("my_event".to_string()),
            Box::new(move |_| *c.lock().unwrap() += 1),
        );
        bus.publish(Event {
            kind: EventKind::Custom("my_event".to_string()),
            payload: "null".to_string(),
            timestamp_ms: ts(),
        });
        bus.publish(Event {
            kind: EventKind::Custom("other".to_string()),
            payload: "null".to_string(),
            timestamp_ms: ts(),
        });
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_make_param_changed_event_kind() {
        let ev = make_param_changed_event("height", 1.75);
        assert_eq!(ev.kind, EventKind::ParamChanged);
        assert!(ev.payload.contains("height"));
    }

    #[test]
    fn test_make_export_event_kind() {
        let ev = make_export_event("/tmp/out.glb", "glb");
        assert_eq!(ev.kind, EventKind::ExportStarted);
        assert!(ev.payload.contains("glb"));
    }

    #[test]
    fn test_make_error_event_kind() {
        let ev = make_error_event("something went wrong");
        assert_eq!(ev.kind, EventKind::Error);
        assert!(ev.payload.contains("went wrong"));
    }

    #[test]
    fn test_new_bus_empty() {
        let bus = EventBus::new();
        assert_eq!(bus.handler_count(), 0);
        assert_eq!(bus.event_count(), 0);
    }

    #[test]
    fn test_plugin_registered_event() {
        let mut bus = EventBus::new();
        let hit = Arc::new(Mutex::new(false));
        let h = Arc::clone(&hit);
        bus.subscribe(
            EventKind::PluginRegistered,
            Box::new(move |_| *h.lock().unwrap() = true),
        );
        bus.publish(Event {
            kind: EventKind::PluginRegistered,
            payload: r#"{"name":"my-plugin"}"#.to_string(),
            timestamp_ms: ts(),
        });
        assert!(*hit.lock().unwrap());
    }

    #[test]
    fn test_history_payload_preserved() {
        let mut bus = EventBus::new();
        bus.publish(Event {
            kind: EventKind::ParamChanged,
            payload: r#"{"x":42}"#.to_string(),
            timestamp_ms: 100,
        });
        assert_eq!(bus.history()[0].payload, r#"{"x":42}"#);
        assert_eq!(bus.history()[0].timestamp_ms, 100);
    }
}
