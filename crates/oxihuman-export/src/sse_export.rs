// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Server-Sent Events export stub — formats mesh/animation data as SSE event streams.

/// A single SSE event.
#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event_type: Option<String>,
    pub data: String,
    pub id: Option<String>,
    pub retry_ms: Option<u32>,
}

/// An SSE export session (a sequence of events).
#[derive(Debug, Default)]
pub struct SseExport {
    pub events: Vec<SseEvent>,
    pub endpoint: String,
}

/// Create a new SSE export session.
pub fn new_sse_export(endpoint: &str) -> SseExport {
    SseExport {
        events: Vec::new(),
        endpoint: endpoint.to_owned(),
    }
}

/// Add a data event.
pub fn sse_add_data(
    export: &mut SseExport,
    data: &str,
    event_type: Option<&str>,
    id: Option<&str>,
) {
    export.events.push(SseEvent {
        event_type: event_type.map(str::to_owned),
        data: data.to_owned(),
        id: id.map(str::to_owned),
        retry_ms: None,
    });
}

/// Add a keep-alive comment line (empty data event).
pub fn sse_keep_alive(export: &mut SseExport) {
    export.events.push(SseEvent {
        event_type: None,
        data: String::new(),
        id: None,
        retry_ms: None,
    });
}

/// Number of events.
pub fn sse_event_count(export: &SseExport) -> usize {
    export.events.len()
}

/// Count events of a specific type.
pub fn events_of_type(export: &SseExport, event_type: &str) -> usize {
    export
        .events
        .iter()
        .filter(|e| e.event_type.as_deref() == Some(event_type))
        .count()
}

/// Total data bytes across all events.
pub fn total_sse_bytes(export: &SseExport) -> usize {
    export.events.iter().map(|e| e.data.len()).sum()
}

/// Render an event to SSE wire format.
pub fn render_sse_event(ev: &SseEvent) -> String {
    let mut out = String::new();
    if let Some(id) = &ev.id {
        out.push_str(&format!("id: {}\n", id));
    }
    if let Some(et) = &ev.event_type {
        out.push_str(&format!("event: {}\n", et));
    }
    if let Some(r) = ev.retry_ms {
        out.push_str(&format!("retry: {}\n", r));
    }
    for line in ev.data.lines() {
        out.push_str(&format!("data: {}\n", line));
    }
    out.push('\n');
    out
}

/// Serialize metadata to JSON-style string.
pub fn sse_export_to_json(export: &SseExport) -> String {
    format!(
        r#"{{"endpoint":"{}", "event_count":{}, "total_bytes":{}}}"#,
        export.endpoint,
        sse_event_count(export),
        total_sse_bytes(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_is_empty() {
        /* fresh export has no events */
        let e = new_sse_export("/events");
        assert_eq!(sse_event_count(&e), 0);
    }

    #[test]
    fn add_data_increments_count() {
        /* adding an event increments count */
        let mut e = new_sse_export("/events");
        sse_add_data(&mut e, "hello", None, None);
        assert_eq!(sse_event_count(&e), 1);
    }

    #[test]
    fn keep_alive_increments_count() {
        /* keep-alive also increments count */
        let mut e = new_sse_export("/events");
        sse_keep_alive(&mut e);
        assert_eq!(sse_event_count(&e), 1);
    }

    #[test]
    fn total_bytes_counted() {
        /* 5-char data counted */
        let mut e = new_sse_export("/events");
        sse_add_data(&mut e, "hello", None, None);
        assert_eq!(total_sse_bytes(&e), 5);
    }

    #[test]
    fn events_of_type_counted() {
        /* typed events counted correctly */
        let mut e = new_sse_export("/events");
        sse_add_data(&mut e, "d", Some("mesh"), None);
        sse_add_data(&mut e, "d", Some("pose"), None);
        assert_eq!(events_of_type(&e, "mesh"), 1);
    }

    #[test]
    fn render_event_contains_data_prefix() {
        /* rendered event includes "data:" prefix */
        let ev = SseEvent {
            event_type: None,
            data: "test".into(),
            id: None,
            retry_ms: None,
        };
        assert!(render_sse_event(&ev).contains("data: test"));
    }

    #[test]
    fn render_event_contains_event_type() {
        /* rendered event includes "event:" when type set */
        let ev = SseEvent {
            event_type: Some("mesh".into()),
            data: "x".into(),
            id: None,
            retry_ms: None,
        };
        assert!(render_sse_event(&ev).contains("event: mesh"));
    }

    #[test]
    fn json_contains_endpoint() {
        /* JSON includes endpoint */
        let e = new_sse_export("/my/stream");
        assert!(sse_export_to_json(&e).contains("/my/stream"));
    }

    #[test]
    fn event_id_stored() {
        /* ID should be stored on the event */
        let mut e = new_sse_export("/events");
        sse_add_data(&mut e, "v", None, Some("42"));
        assert_eq!(e.events[0].id.as_deref(), Some("42"));
    }
}
