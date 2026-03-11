// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Synchronous event dispatcher with typed handler table.

use std::collections::HashMap;

/// Opaque handler identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub struct HandlerId(u32);

/// A record of a dispatched event.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DispatchRecord {
    pub event_type: String,
    pub handled: bool,
    pub handler_count: usize,
}

/// Event dispatcher.
#[derive(Debug)]
#[allow(dead_code)]
pub struct EventDispatcher {
    next_id: u32,
    handlers: HashMap<String, Vec<(HandlerId, String)>>,
    dispatch_count: u64,
}

/// Create a new EventDispatcher.
#[allow(dead_code)]
pub fn new_dispatcher() -> EventDispatcher {
    EventDispatcher {
        next_id: 1,
        handlers: HashMap::new(),
        dispatch_count: 0,
    }
}

/// Register a named handler for an event type. Returns HandlerId.
#[allow(dead_code)]
pub fn register_handler(
    d: &mut EventDispatcher,
    event_type: &str,
    handler_name: &str,
) -> HandlerId {
    let id = HandlerId(d.next_id);
    d.next_id += 1;
    d.handlers
        .entry(event_type.to_string())
        .or_default()
        .push((id, handler_name.to_string()));
    id
}

/// Unregister a handler by id.
#[allow(dead_code)]
pub fn unregister_handler(d: &mut EventDispatcher, event_type: &str, id: HandlerId) -> bool {
    if let Some(list) = d.handlers.get_mut(event_type) {
        let before = list.len();
        list.retain(|(hid, _)| *hid != id);
        return list.len() < before;
    }
    false
}

/// Dispatch an event; returns number of handlers invoked.
#[allow(dead_code)]
pub fn dispatch(d: &mut EventDispatcher, event_type: &str) -> DispatchRecord {
    let count = d.handlers.get(event_type).map(|v| v.len()).unwrap_or(0);
    d.dispatch_count += 1;
    DispatchRecord {
        event_type: event_type.to_string(),
        handled: count > 0,
        handler_count: count,
    }
}

/// Number of handlers for a given event type.
#[allow(dead_code)]
pub fn handler_count(d: &EventDispatcher, event_type: &str) -> usize {
    d.handlers.get(event_type).map(|v| v.len()).unwrap_or(0)
}

/// Total number of dispatches executed.
#[allow(dead_code)]
pub fn dispatch_count(d: &EventDispatcher) -> u64 {
    d.dispatch_count
}

/// All registered event types.
#[allow(dead_code)]
pub fn registered_event_types(d: &EventDispatcher) -> Vec<String> {
    d.handlers.keys().cloned().collect()
}

/// Clear all handlers for an event type.
#[allow(dead_code)]
pub fn clear_handlers(d: &mut EventDispatcher, event_type: &str) {
    d.handlers.remove(event_type);
}

/// Clear all handlers across all event types.
#[allow(dead_code)]
pub fn clear_all_handlers(d: &mut EventDispatcher) {
    d.handlers.clear();
}

/// Names of all handlers for a type.
#[allow(dead_code)]
pub fn handler_names(d: &EventDispatcher, event_type: &str) -> Vec<String> {
    d.handlers
        .get(event_type)
        .map(|v| v.iter().map(|(_, n)| n.clone()).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_dispatch() {
        let mut d = new_dispatcher();
        register_handler(&mut d, "click", "on_click");
        let rec = dispatch(&mut d, "click");
        assert!(rec.handled);
        assert_eq!(rec.handler_count, 1);
    }

    #[test]
    fn test_dispatch_no_handlers() {
        let mut d = new_dispatcher();
        let rec = dispatch(&mut d, "unknown");
        assert!(!rec.handled);
        assert_eq!(rec.handler_count, 0);
    }

    #[test]
    fn test_unregister() {
        let mut d = new_dispatcher();
        let id = register_handler(&mut d, "resize", "h1");
        assert!(unregister_handler(&mut d, "resize", id));
        assert_eq!(handler_count(&d, "resize"), 0);
    }

    #[test]
    fn test_multiple_handlers() {
        let mut d = new_dispatcher();
        register_handler(&mut d, "key", "h1");
        register_handler(&mut d, "key", "h2");
        assert_eq!(handler_count(&d, "key"), 2);
    }

    #[test]
    fn test_dispatch_count() {
        let mut d = new_dispatcher();
        dispatch(&mut d, "a");
        dispatch(&mut d, "b");
        assert_eq!(dispatch_count(&d), 2);
    }

    #[test]
    fn test_clear_handlers() {
        let mut d = new_dispatcher();
        register_handler(&mut d, "e", "h");
        clear_handlers(&mut d, "e");
        assert_eq!(handler_count(&d, "e"), 0);
    }

    #[test]
    fn test_registered_event_types() {
        let mut d = new_dispatcher();
        register_handler(&mut d, "a", "h1");
        register_handler(&mut d, "b", "h2");
        let types = registered_event_types(&d);
        assert_eq!(types.len(), 2);
    }

    #[test]
    fn test_handler_names() {
        let mut d = new_dispatcher();
        register_handler(&mut d, "ev", "alpha");
        register_handler(&mut d, "ev", "beta");
        let names = handler_names(&d, "ev");
        assert!(names.contains(&"alpha".to_string()));
        assert!(names.contains(&"beta".to_string()));
    }

    #[test]
    fn test_clear_all() {
        let mut d = new_dispatcher();
        register_handler(&mut d, "x", "h");
        register_handler(&mut d, "y", "h");
        clear_all_handlers(&mut d);
        assert_eq!(registered_event_types(&d).len(), 0);
    }
}
