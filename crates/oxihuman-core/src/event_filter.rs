// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Event stream filter and transformer.

/// A raw event with a kind tag and payload.
#[derive(Debug, Clone, PartialEq)]
pub struct FilterEvent {
    pub kind: String,
    pub payload: String,
}

impl FilterEvent {
    /// Create a new event.
    pub fn new(kind: impl Into<String>, payload: impl Into<String>) -> Self {
        FilterEvent { kind: kind.into(), payload: payload.into() }
    }
}

/// An event filter/transformer pipeline.
pub struct EventFilter {
    allowed_kinds: Vec<String>,
    blocked_kinds: Vec<String>,
    transforms: Vec<Box<dyn Fn(FilterEvent) -> FilterEvent + Send + Sync>>,
}

impl EventFilter {
    /// Create a new event filter (passes all events by default).
    pub fn new() -> Self {
        EventFilter {
            allowed_kinds: Vec::new(),
            blocked_kinds: Vec::new(),
            transforms: Vec::new(),
        }
    }

    /// Only allow events whose kind is in the allow list.
    pub fn allow_kind(mut self, kind: impl Into<String>) -> Self {
        self.allowed_kinds.push(kind.into());
        self
    }

    /// Block events of a specific kind.
    pub fn block_kind(mut self, kind: impl Into<String>) -> Self {
        self.blocked_kinds.push(kind.into());
        self
    }

    /// Add a transform applied to every passing event.
    pub fn transform(mut self, f: impl Fn(FilterEvent) -> FilterEvent + Send + Sync + 'static) -> Self {
        self.transforms.push(Box::new(f));
        self
    }

    /// Process a batch of events through the filter.
    pub fn process(&self, events: Vec<FilterEvent>) -> Vec<FilterEvent> {
        events
            .into_iter()
            .filter(|e| {
                if !self.blocked_kinds.is_empty() && self.blocked_kinds.contains(&e.kind) {
                    return false;
                }
                if !self.allowed_kinds.is_empty() && !self.allowed_kinds.contains(&e.kind) {
                    return false;
                }
                true
            })
            .map(|mut e| {
                for t in &self.transforms {
                    e = t(e);
                }
                e
            })
            .collect()
    }
}

impl Default for EventFilter {
    fn default() -> Self {
        EventFilter::new()
    }
}

/// Create a new event filter.
pub fn new_event_filter() -> EventFilter {
    EventFilter::new()
}

/// Process events through a filter.
pub fn ef_process(filter: &EventFilter, events: Vec<FilterEvent>) -> Vec<FilterEvent> {
    filter.process(events)
}

/// Count events that pass the filter.
pub fn ef_count_passing(filter: &EventFilter, events: Vec<FilterEvent>) -> usize {
    filter.process(events).len()
}

/// Build a simple kind-allowlist filter.
pub fn ef_allow_kinds(kinds: &[&str]) -> EventFilter {
    let mut f = EventFilter::new();
    for k in kinds {
        f = f.allow_kind(*k);
    }
    f
}

/// Build a simple kind-blocklist filter.
pub fn ef_block_kinds(kinds: &[&str]) -> EventFilter {
    let mut f = EventFilter::new();
    for k in kinds {
        f = f.block_kind(*k);
    }
    f
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(kind: &str, payload: &str) -> FilterEvent {
        FilterEvent::new(kind, payload)
    }

    #[test]
    fn test_pass_all() {
        let f = new_event_filter();
        let events = vec![ev("click", "x"), ev("hover", "y")];
        assert_eq!(ef_process(&f, events).len(), 2 /* all pass */);
    }

    #[test]
    fn test_allow_kind() {
        let f = ef_allow_kinds(&["click"]);
        let events = vec![ev("click", "x"), ev("hover", "y")];
        let out = ef_process(&f, events);
        assert_eq!(out.len(), 1 /* only click passes */);
        assert_eq!(out[0].kind, "click");
    }

    #[test]
    fn test_block_kind() {
        let f = ef_block_kinds(&["spam"]);
        let events = vec![ev("ok", "a"), ev("spam", "b")];
        let out = ef_process(&f, events);
        assert_eq!(out.len(), 1 /* spam blocked */);
    }

    #[test]
    fn test_transform() {
        let f = new_event_filter()
            .transform(|mut e| { e.payload = e.payload.to_uppercase(); e });
        let events = vec![ev("t", "hello")];
        let out = ef_process(&f, events);
        assert_eq!(out[0].payload, "HELLO" /* transformed */);
    }

    #[test]
    fn test_combined_allow_and_transform() {
        let f = new_event_filter()
            .allow_kind("A")
            .transform(|mut e| { e.kind = "processed".into(); e });
        let events = vec![ev("A", "x"), ev("B", "y")];
        let out = ef_process(&f, events);
        assert_eq!(out.len(), 1 /* only A */);
        assert_eq!(out[0].kind, "processed" /* transformed */);
    }

    #[test]
    fn test_count_passing() {
        let f = ef_allow_kinds(&["x"]);
        let events = vec![ev("x", ""), ev("x", ""), ev("y", "")];
        assert_eq!(ef_count_passing(&f, events), 2 /* two x events */);
    }

    #[test]
    fn test_empty_events() {
        let f = new_event_filter();
        assert_eq!(ef_process(&f, vec![]).len(), 0 /* empty input */);
    }

    #[test]
    fn test_block_all() {
        let f = ef_block_kinds(&["a", "b", "c"]);
        let events = vec![ev("a", ""), ev("b", ""), ev("c", "")];
        assert_eq!(ef_process(&f, events).len(), 0 /* all blocked */);
    }

    #[test]
    fn test_filter_event_new() {
        let e = FilterEvent::new("click", "pos");
        assert_eq!(e.kind, "click" /* kind set */);
        assert_eq!(e.payload, "pos");
    }

    #[test]
    fn test_multiple_transforms() {
        let f = new_event_filter()
            .transform(|mut e| { e.payload = format!("{}!", e.payload); e })
            .transform(|mut e| { e.payload = format!("{}!", e.payload); e });
        let events = vec![ev("t", "hi")];
        let out = ef_process(&f, events);
        assert_eq!(out[0].payload, "hi!!" /* double transform */);
    }
}
