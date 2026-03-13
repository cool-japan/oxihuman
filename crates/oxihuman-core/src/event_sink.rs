// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// An event sink that collects events for later processing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EventRecord {
    pub name: String,
    pub timestamp: f64,
    pub data: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EventSink {
    events: Vec<EventRecord>,
    capacity: usize,
    dropped: u64,
}

#[allow(dead_code)]
impl EventSink {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: Vec::new(),
            capacity,
            dropped: 0,
        }
    }

    pub fn emit(&mut self, name: &str, timestamp: f64, data: &str) {
        if self.events.len() >= self.capacity {
            self.dropped += 1;
            return;
        }
        self.events.push(EventRecord {
            name: name.to_string(),
            timestamp,
            data: data.to_string(),
        });
    }

    pub fn drain(&mut self) -> Vec<EventRecord> {
        std::mem::take(&mut self.events)
    }

    pub fn peek(&self) -> Option<&EventRecord> {
        self.events.last()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn filter_by_name(&self, name: &str) -> Vec<&EventRecord> {
        self.events.iter().filter(|e| e.name == name).collect()
    }

    pub fn latest_by_name(&self, name: &str) -> Option<&EventRecord> {
        self.events.iter().rev().find(|e| e.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let sink = EventSink::new(100);
        assert!(sink.is_empty());
        assert_eq!(sink.capacity(), 100);
    }

    #[test]
    fn test_emit() {
        let mut sink = EventSink::new(10);
        sink.emit("click", 1.0, "button_a");
        assert_eq!(sink.len(), 1);
    }

    #[test]
    fn test_drain() {
        let mut sink = EventSink::new(10);
        sink.emit("e1", 0.0, "d1");
        sink.emit("e2", 1.0, "d2");
        let events = sink.drain();
        assert_eq!(events.len(), 2);
        assert!(sink.is_empty());
    }

    #[test]
    fn test_capacity_drop() {
        let mut sink = EventSink::new(2);
        sink.emit("a", 0.0, "");
        sink.emit("b", 1.0, "");
        sink.emit("c", 2.0, "");
        assert_eq!(sink.len(), 2);
        assert_eq!(sink.dropped(), 1);
    }

    #[test]
    fn test_peek() {
        let mut sink = EventSink::new(10);
        sink.emit("first", 0.0, "");
        sink.emit("second", 1.0, "");
        assert_eq!(sink.peek().expect("should succeed").name, "second");
    }

    #[test]
    fn test_clear() {
        let mut sink = EventSink::new(10);
        sink.emit("e", 0.0, "");
        sink.clear();
        assert!(sink.is_empty());
    }

    #[test]
    fn test_filter_by_name() {
        let mut sink = EventSink::new(10);
        sink.emit("click", 0.0, "a");
        sink.emit("hover", 1.0, "b");
        sink.emit("click", 2.0, "c");
        let clicks = sink.filter_by_name("click");
        assert_eq!(clicks.len(), 2);
    }

    #[test]
    fn test_latest_by_name() {
        let mut sink = EventSink::new(10);
        sink.emit("tick", 1.0, "first");
        sink.emit("tick", 2.0, "second");
        let latest = sink.latest_by_name("tick").expect("should succeed");
        assert_eq!(latest.data, "second");
    }

    #[test]
    fn test_latest_by_name_missing() {
        let sink = EventSink::new(10);
        assert!(sink.latest_by_name("nope").is_none());
    }

    #[test]
    fn test_event_record_fields() {
        let mut sink = EventSink::new(10);
        sink.emit("test", 2.78, "payload");
        let e = sink.peek().expect("should succeed");
        assert_eq!(e.name, "test");
        assert!((e.timestamp - 2.78).abs() < 1e-12);
        assert_eq!(e.data, "payload");
    }
}
