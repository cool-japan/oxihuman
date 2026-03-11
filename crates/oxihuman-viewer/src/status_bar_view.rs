// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Status bar info view.

/// An info segment in the status bar.
#[derive(Debug, Clone)]
pub struct StatusSegment {
    pub key: String,
    pub value: String,
}

/// State for the status bar view.
#[derive(Debug, Clone)]
pub struct StatusBarView {
    pub segments: Vec<StatusSegment>,
    pub message: String,
    pub show_memory: bool,
    pub enabled: bool,
}

/// Create a new status bar view.
pub fn new_status_bar_view() -> StatusBarView {
    StatusBarView {
        segments: Vec::new(),
        message: String::new(),
        show_memory: true,
        enabled: true,
    }
}

/// Set the status message.
pub fn sbv_set_message(v: &mut StatusBarView, msg: &str) {
    v.message = msg.to_string();
}

/// Add an info segment.
pub fn sbv_add_segment(v: &mut StatusBarView, key: &str, value: &str) {
    v.segments.push(StatusSegment {
        key: key.to_string(),
        value: value.to_string(),
    });
}

/// Update a segment value by key.
pub fn sbv_update_segment(v: &mut StatusBarView, key: &str, value: &str) {
    if let Some(s) = v.segments.iter_mut().find(|s| s.key == key) {
        s.value = value.to_string();
    }
}

/// Build a display string from all segments.
pub fn sbv_display_string(v: &StatusBarView) -> String {
    v.segments
        .iter()
        .map(|s| format!("{}: {}", s.key, s.value))
        .collect::<Vec<_>>()
        .join(" | ")
}

/// Serialise to JSON.
pub fn sbv_to_json(v: &StatusBarView) -> String {
    format!(
        r#"{{"message":"{}","segment_count":{},"enabled":{}}}"#,
        v.message,
        v.segments.len(),
        v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_empty_message() {
        let v = new_status_bar_view();
        assert!(v.message.is_empty() /* empty message */);
    }

    #[test]
    fn set_message() {
        let mut v = new_status_bar_view();
        sbv_set_message(&mut v, "Ready");
        assert_eq!(v.message, "Ready" /* message set */);
    }

    #[test]
    fn add_segment() {
        let mut v = new_status_bar_view();
        sbv_add_segment(&mut v, "FPS", "60");
        assert_eq!(v.segments.len(), 1 /* one segment */);
    }

    #[test]
    fn update_segment() {
        let mut v = new_status_bar_view();
        sbv_add_segment(&mut v, "FPS", "60");
        sbv_update_segment(&mut v, "FPS", "30");
        assert_eq!(v.segments[0].value, "30" /* updated */);
    }

    #[test]
    fn display_string_joined() {
        let mut v = new_status_bar_view();
        sbv_add_segment(&mut v, "FPS", "60");
        sbv_add_segment(&mut v, "Tris", "1000");
        let s = sbv_display_string(&v);
        assert!(s.contains("FPS: 60") && s.contains("Tris: 1000") /* both in string */);
    }

    #[test]
    fn json_has_message() {
        let mut v = new_status_bar_view();
        sbv_set_message(&mut v, "Loading");
        assert!(sbv_to_json(&v).contains("Loading") /* message in json */);
    }

    #[test]
    fn show_memory_default() {
        let v = new_status_bar_view();
        assert!(v.show_memory /* memory shown by default */);
    }

    #[test]
    fn enabled_default() {
        let v = new_status_bar_view();
        assert!(v.enabled /* enabled */);
    }
}
