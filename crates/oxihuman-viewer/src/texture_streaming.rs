// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// State of a texture streaming request.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamingState {
    Pending,
    Loading,
    Loaded,
    Failed,
}

/// Manages texture streaming requests.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TextureStreaming {
    pub requests: HashMap<String, StreamingState>,
}

/// Create a new texture streaming manager.
#[allow(dead_code)]
pub fn new_texture_streaming() -> TextureStreaming {
    TextureStreaming {
        requests: HashMap::new(),
    }
}

/// Request a texture to be streamed in.
#[allow(dead_code)]
pub fn request_texture(ts: &mut TextureStreaming, name: &str) {
    ts.requests
        .entry(name.to_string())
        .or_insert(StreamingState::Pending);
}

/// Check if a texture is loaded.
#[allow(dead_code)]
pub fn texture_is_loaded(ts: &TextureStreaming, name: &str) -> bool {
    ts.requests.get(name) == Some(&StreamingState::Loaded)
}

/// Return the total number of requests.
#[allow(dead_code)]
pub fn streaming_count(ts: &TextureStreaming) -> usize {
    ts.requests.len()
}

/// Return the number of pending requests.
#[allow(dead_code)]
pub fn pending_count(ts: &TextureStreaming) -> usize {
    ts.requests
        .values()
        .filter(|s| **s == StreamingState::Pending || **s == StreamingState::Loading)
        .count()
}

/// Return the number of loaded textures.
#[allow(dead_code)]
pub fn loaded_count(ts: &TextureStreaming) -> usize {
    ts.requests
        .values()
        .filter(|s| **s == StreamingState::Loaded)
        .count()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn streaming_to_json(ts: &TextureStreaming) -> String {
    let mut entries: Vec<String> = ts
        .requests
        .iter()
        .map(|(k, v)| {
            let state = match v {
                StreamingState::Pending => "pending",
                StreamingState::Loading => "loading",
                StreamingState::Loaded => "loaded",
                StreamingState::Failed => "failed",
            };
            format!("\"{}\":\"{}\"", k, state)
        })
        .collect();
    entries.sort();
    format!("{{{}}}", entries.join(","))
}

/// Cancel a pending request.
#[allow(dead_code)]
pub fn cancel_request(ts: &mut TextureStreaming, name: &str) {
    ts.requests.remove(name);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_streaming() {
        let ts = new_texture_streaming();
        assert_eq!(streaming_count(&ts), 0);
    }

    #[test]
    fn request_adds_entry() {
        let mut ts = new_texture_streaming();
        request_texture(&mut ts, "diffuse.png");
        assert_eq!(streaming_count(&ts), 1);
    }

    #[test]
    fn not_loaded_initially() {
        let mut ts = new_texture_streaming();
        request_texture(&mut ts, "diffuse.png");
        assert!(!texture_is_loaded(&ts, "diffuse.png"));
    }

    #[test]
    fn pending_count_works() {
        let mut ts = new_texture_streaming();
        request_texture(&mut ts, "a");
        request_texture(&mut ts, "b");
        assert_eq!(pending_count(&ts), 2);
    }

    #[test]
    fn loaded_count_works() {
        let mut ts = new_texture_streaming();
        request_texture(&mut ts, "a");
        ts.requests.insert("a".to_string(), StreamingState::Loaded);
        assert_eq!(loaded_count(&ts), 1);
    }

    #[test]
    fn cancel_removes() {
        let mut ts = new_texture_streaming();
        request_texture(&mut ts, "a");
        cancel_request(&mut ts, "a");
        assert_eq!(streaming_count(&ts), 0);
    }

    #[test]
    fn to_json() {
        let mut ts = new_texture_streaming();
        request_texture(&mut ts, "test");
        let j = streaming_to_json(&ts);
        assert!(j.contains("test"));
    }

    #[test]
    fn duplicate_request_no_change() {
        let mut ts = new_texture_streaming();
        request_texture(&mut ts, "a");
        request_texture(&mut ts, "a");
        assert_eq!(streaming_count(&ts), 1);
    }

    #[test]
    fn loaded_check_missing() {
        let ts = new_texture_streaming();
        assert!(!texture_is_loaded(&ts, "missing"));
    }

    #[test]
    fn cancel_nonexistent() {
        let mut ts = new_texture_streaming();
        cancel_request(&mut ts, "nope"); // should not panic
        assert_eq!(streaming_count(&ts), 0);
    }
}
