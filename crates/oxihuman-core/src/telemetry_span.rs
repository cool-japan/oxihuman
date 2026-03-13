// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! OpenTelemetry-style trace span stub.

use std::collections::HashMap;

/// Status of a trace span.
#[derive(Debug, Clone, PartialEq)]
pub enum SpanStatus {
    Unset,
    Ok,
    Error(String),
}

/// A single trace span.
#[derive(Debug, Clone)]
pub struct TelemetrySpan {
    pub span_id: u64,
    pub name: String,
    pub start_us: u64,
    pub end_us: Option<u64>,
    pub attributes: HashMap<String, String>,
    pub status: SpanStatus,
}

impl TelemetrySpan {
    pub fn new(span_id: u64, name: &str, start_us: u64) -> Self {
        Self {
            span_id,
            name: name.to_string(),
            start_us,
            end_us: None,
            attributes: HashMap::new(),
            status: SpanStatus::Unset,
        }
    }

    pub fn end(&mut self, end_us: u64) {
        self.end_us = Some(end_us);
    }

    pub fn set_attribute(&mut self, key: &str, value: &str) {
        self.attributes.insert(key.to_string(), value.to_string());
    }

    pub fn set_ok(&mut self) {
        self.status = SpanStatus::Ok;
    }

    pub fn set_error(&mut self, msg: &str) {
        self.status = SpanStatus::Error(msg.to_string());
    }

    pub fn duration_us(&self) -> Option<u64> {
        self.end_us.map(|e| e.saturating_sub(self.start_us))
    }

    pub fn is_finished(&self) -> bool {
        self.end_us.is_some()
    }
}

pub fn new_telemetry_span(span_id: u64, name: &str, start_us: u64) -> TelemetrySpan {
    TelemetrySpan::new(span_id, name, start_us)
}

pub fn span_end(span: &mut TelemetrySpan, end_us: u64) {
    span.end(end_us);
}

pub fn span_set_attr(span: &mut TelemetrySpan, key: &str, value: &str) {
    span.set_attribute(key, value);
}

pub fn span_set_ok(span: &mut TelemetrySpan) {
    span.set_ok();
}

pub fn span_set_error(span: &mut TelemetrySpan, msg: &str) {
    span.set_error(msg);
}

pub fn span_duration_us(span: &TelemetrySpan) -> Option<u64> {
    span.duration_us()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_span() {
        let s = new_telemetry_span(1, "my_op", 1000);
        assert_eq!(s.name, "my_op");
        assert!(!s.is_finished());
    }

    #[test]
    fn test_end_span() {
        let mut s = new_telemetry_span(1, "op", 1000);
        span_end(&mut s, 2000);
        assert!(s.is_finished());
    }

    #[test]
    fn test_duration() {
        let mut s = new_telemetry_span(1, "op", 500);
        span_end(&mut s, 1500);
        assert_eq!(span_duration_us(&s), Some(1000));
    }

    #[test]
    fn test_duration_none_if_not_ended() {
        let s = new_telemetry_span(1, "op", 0);
        assert_eq!(span_duration_us(&s), None);
    }

    #[test]
    fn test_set_attribute() {
        let mut s = new_telemetry_span(1, "op", 0);
        span_set_attr(&mut s, "service.name", "auth");
        assert_eq!(s.attributes["service.name"], "auth");
    }

    #[test]
    fn test_set_ok_status() {
        let mut s = new_telemetry_span(1, "op", 0);
        span_set_ok(&mut s);
        assert_eq!(s.status, SpanStatus::Ok);
    }

    #[test]
    fn test_set_error_status() {
        let mut s = new_telemetry_span(1, "op", 0);
        span_set_error(&mut s, "timeout");
        assert!(matches!(s.status, SpanStatus::Error(_)));
    }

    #[test]
    fn test_multiple_attributes() {
        let mut s = new_telemetry_span(1, "op", 0);
        span_set_attr(&mut s, "http.method", "GET");
        span_set_attr(&mut s, "http.status", "200");
        assert_eq!(s.attributes.len(), 2);
    }

    #[test]
    fn test_initial_status_is_unset() {
        let s = new_telemetry_span(1, "op", 0);
        assert_eq!(s.status, SpanStatus::Unset);
    }
}
