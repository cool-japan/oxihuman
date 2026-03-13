// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ring-buffer trace for recording named span events.

use std::collections::VecDeque;

/// A single trace event.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct TraceEvent {
    pub name: String,
    pub tick: u64,
    pub duration_us: u64,
    pub tag: String,
}

/// Trace buffer holding up to `capacity` events.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TraceBuffer {
    events: VecDeque<TraceEvent>,
    capacity: usize,
    tick: u64,
}

/// Create a new `TraceBuffer`.
#[allow(dead_code)]
pub fn new_trace_buffer(capacity: usize) -> TraceBuffer {
    TraceBuffer {
        events: VecDeque::new(),
        capacity: capacity.max(1),
        tick: 0,
    }
}

/// Record an event. Evicts oldest if at capacity.
#[allow(dead_code)]
pub fn tb_record(buf: &mut TraceBuffer, name: &str, duration_us: u64, tag: &str) {
    if buf.events.len() >= buf.capacity {
        buf.events.pop_front();
    }
    buf.events.push_back(TraceEvent {
        name: name.to_string(),
        tick: buf.tick,
        duration_us,
        tag: tag.to_string(),
    });
    buf.tick += 1;
}

/// Number of events stored.
#[allow(dead_code)]
pub fn tb_len(buf: &TraceBuffer) -> usize {
    buf.events.len()
}

/// Whether the buffer is empty.
#[allow(dead_code)]
pub fn tb_is_empty(buf: &TraceBuffer) -> bool {
    buf.events.is_empty()
}

/// Retrieve event by index (0 = oldest).
#[allow(dead_code)]
pub fn tb_get(buf: &TraceBuffer, index: usize) -> Option<&TraceEvent> {
    buf.events.get(index)
}

/// Clear all events.
#[allow(dead_code)]
pub fn tb_clear(buf: &mut TraceBuffer) {
    buf.events.clear();
    buf.tick = 0;
}

/// Filter events by tag.
#[allow(dead_code)]
pub fn tb_by_tag<'a>(buf: &'a TraceBuffer, tag: &str) -> Vec<&'a TraceEvent> {
    buf.events.iter().filter(|e| e.tag == tag).collect()
}

/// Average duration of all events in microseconds.
#[allow(dead_code)]
pub fn tb_avg_duration_us(buf: &TraceBuffer) -> f64 {
    if buf.events.is_empty() {
        return 0.0;
    }
    let sum: u64 = buf.events.iter().map(|e| e.duration_us).sum();
    sum as f64 / buf.events.len() as f64
}

/// Maximum duration event.
#[allow(dead_code)]
pub fn tb_max_event(buf: &TraceBuffer) -> Option<&TraceEvent> {
    buf.events.iter().max_by_key(|e| e.duration_us)
}

/// Total tick count (monotone counter).
#[allow(dead_code)]
pub fn tb_tick(buf: &TraceBuffer) -> u64 {
    buf.tick
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let buf = new_trace_buffer(10);
        assert!(tb_is_empty(&buf));
        assert_eq!(tb_len(&buf), 0);
    }

    #[test]
    fn test_record_and_len() {
        let mut buf = new_trace_buffer(10);
        tb_record(&mut buf, "render", 100, "gpu");
        assert_eq!(tb_len(&buf), 1);
    }

    #[test]
    fn test_capacity_evicts() {
        let mut buf = new_trace_buffer(3);
        tb_record(&mut buf, "a", 1, "t");
        tb_record(&mut buf, "b", 2, "t");
        tb_record(&mut buf, "c", 3, "t");
        tb_record(&mut buf, "d", 4, "t");
        assert_eq!(tb_len(&buf), 3);
        assert_eq!(
            tb_get(&buf, 0).expect("should succeed").name,
            "b".to_string()
        );
    }

    #[test]
    fn test_get_event() {
        let mut buf = new_trace_buffer(10);
        tb_record(&mut buf, "physics", 50, "cpu");
        let ev = tb_get(&buf, 0).expect("should succeed");
        assert_eq!(ev.name, "physics".to_string());
        assert_eq!(ev.duration_us, 50);
    }

    #[test]
    fn test_by_tag() {
        let mut buf = new_trace_buffer(10);
        tb_record(&mut buf, "a", 10, "gpu");
        tb_record(&mut buf, "b", 20, "cpu");
        tb_record(&mut buf, "c", 30, "gpu");
        let gpu = tb_by_tag(&buf, "gpu");
        assert_eq!(gpu.len(), 2);
    }

    #[test]
    fn test_avg_duration() {
        let mut buf = new_trace_buffer(10);
        tb_record(&mut buf, "a", 100, "t");
        tb_record(&mut buf, "b", 200, "t");
        let avg = tb_avg_duration_us(&buf);
        assert!((avg - 150.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_event() {
        let mut buf = new_trace_buffer(10);
        tb_record(&mut buf, "slow", 999, "t");
        tb_record(&mut buf, "fast", 1, "t");
        let max = tb_max_event(&buf).expect("should succeed");
        assert_eq!(max.name, "slow".to_string());
    }

    #[test]
    fn test_clear() {
        let mut buf = new_trace_buffer(10);
        tb_record(&mut buf, "x", 5, "t");
        tb_clear(&mut buf);
        assert!(tb_is_empty(&buf));
        assert_eq!(tb_tick(&buf), 0);
    }

    #[test]
    fn test_tick_increments() {
        let mut buf = new_trace_buffer(10);
        tb_record(&mut buf, "a", 1, "t");
        tb_record(&mut buf, "b", 2, "t");
        assert_eq!(tb_tick(&buf), 2);
    }
}
