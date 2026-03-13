// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Streaming morph target loader stub.

/// State of a streaming morph load operation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StreamState {
    Idle,
    Loading,
    Ready,
    Error,
}

/// A streaming morph target entry in the queue.
#[derive(Debug, Clone)]
pub struct StreamEntry {
    pub name: String,
    pub priority: u8,
    pub state: StreamState,
}

/// Streaming morph target loader.
#[derive(Debug, Clone)]
pub struct StreamingMorph {
    pub queue: Vec<StreamEntry>,
    pub max_concurrent: usize,
    pub enabled: bool,
}

impl StreamingMorph {
    pub fn new(max_concurrent: usize) -> Self {
        StreamingMorph {
            queue: Vec::new(),
            max_concurrent,
            enabled: true,
        }
    }
}

/// Create a new streaming morph loader.
pub fn new_streaming_morph(max_concurrent: usize) -> StreamingMorph {
    StreamingMorph::new(max_concurrent)
}

/// Enqueue a morph target for streaming.
pub fn sm_enqueue(loader: &mut StreamingMorph, name: impl Into<String>, priority: u8) {
    loader.queue.push(StreamEntry {
        name: name.into(),
        priority,
        state: StreamState::Idle,
    });
}

/// Tick the loader: marks pending entries as Loading (stub).
pub fn sm_tick(loader: &mut StreamingMorph) {
    /* Stub: advance up to max_concurrent entries from Idle to Loading */
    let mut loading_count = loader
        .queue
        .iter()
        .filter(|e| e.state == StreamState::Loading)
        .count();
    for entry in &mut loader.queue {
        if loading_count >= loader.max_concurrent {
            break;
        }
        if entry.state == StreamState::Idle {
            entry.state = StreamState::Loading;
            loading_count += 1;
        }
    }
}

/// Return queue length.
pub fn sm_queue_len(loader: &StreamingMorph) -> usize {
    loader.queue.len()
}

/// Enable or disable the loader.
pub fn sm_set_enabled(loader: &mut StreamingMorph, enabled: bool) {
    loader.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn sm_to_json(loader: &StreamingMorph) -> String {
    format!(
        r#"{{"queue_len":{},"max_concurrent":{},"enabled":{}}}"#,
        loader.queue.len(),
        loader.max_concurrent,
        loader.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty_queue() {
        let l = new_streaming_morph(4);
        assert_eq!(sm_queue_len(&l), 0 /* queue must be empty initially */,);
    }

    #[test]
    fn test_enqueue() {
        let mut l = new_streaming_morph(4);
        sm_enqueue(&mut l, "brow_raise", 10);
        assert_eq!(sm_queue_len(&l), 1 /* one entry after enqueue */,);
    }

    #[test]
    fn test_tick_sets_loading() {
        let mut l = new_streaming_morph(2);
        sm_enqueue(&mut l, "a", 1);
        sm_enqueue(&mut l, "b", 2);
        sm_enqueue(&mut l, "c", 3);
        sm_tick(&mut l);
        let loading = l
            .queue
            .iter()
            .filter(|e| e.state == StreamState::Loading)
            .count();
        assert_eq!(
            loading,
            2, /* only max_concurrent entries should be loading */
        );
    }

    #[test]
    fn test_tick_respects_max() {
        let mut l = new_streaming_morph(1);
        sm_enqueue(&mut l, "x", 5);
        sm_enqueue(&mut l, "y", 3);
        sm_tick(&mut l);
        let loading = l
            .queue
            .iter()
            .filter(|e| e.state == StreamState::Loading)
            .count();
        assert_eq!(loading, 1 /* only 1 entry loading at a time */,);
    }

    #[test]
    fn test_initial_state_idle() {
        let mut l = new_streaming_morph(2);
        sm_enqueue(&mut l, "z", 0);
        assert_eq!(
            l.queue[0].state,
            StreamState::Idle, /* must start as Idle */
        );
    }

    #[test]
    fn test_set_enabled() {
        let mut l = new_streaming_morph(2);
        sm_set_enabled(&mut l, false);
        assert!(!l.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_queue_len() {
        let l = new_streaming_morph(4);
        let j = sm_to_json(&l);
        assert!(j.contains("\"queue_len\""), /* json must contain queue_len */);
    }

    #[test]
    fn test_priority_stored() {
        let mut l = new_streaming_morph(4);
        sm_enqueue(&mut l, "hi", 255);
        assert_eq!(l.queue[0].priority, 255 /* priority must be stored */,);
    }

    #[test]
    fn test_max_concurrent_stored() {
        let l = new_streaming_morph(8);
        assert_eq!(l.max_concurrent, 8 /* max_concurrent must match */,);
    }

    #[test]
    fn test_enabled_default() {
        let l = new_streaming_morph(2);
        assert!(l.enabled /* must be enabled by default */,);
    }
}
