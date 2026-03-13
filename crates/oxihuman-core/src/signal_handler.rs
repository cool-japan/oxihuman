// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Named signal handler registry for hooking OS-style signals or custom events.

use std::collections::HashMap;

/// A registered handler entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandlerEntry {
    pub name: String,
    pub signal: String,
    pub priority: i32,
    pub enabled: bool,
    pub call_count: u64,
}

/// Registry of named signal handlers.
#[allow(dead_code)]
pub struct SignalHandler {
    handlers: HashMap<String, HandlerEntry>,
    dispatch_log: Vec<(String, u64)>,
    total_dispatched: u64,
}

#[allow(dead_code)]
impl SignalHandler {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            dispatch_log: Vec::new(),
            total_dispatched: 0,
        }
    }

    /// Register a handler for a signal.
    pub fn register(&mut self, name: &str, signal: &str, priority: i32) {
        self.handlers.insert(
            name.to_string(),
            HandlerEntry {
                name: name.to_string(),
                signal: signal.to_string(),
                priority,
                enabled: true,
                call_count: 0,
            },
        );
    }

    /// Unregister a handler.
    pub fn unregister(&mut self, name: &str) -> bool {
        self.handlers.remove(name).is_some()
    }

    /// Enable or disable a handler.
    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> bool {
        if let Some(h) = self.handlers.get_mut(name) {
            h.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// Dispatch a signal; calls all enabled handlers sorted by priority desc.
    pub fn dispatch(&mut self, signal: &str, timestamp: u64) -> usize {
        self.total_dispatched += 1;
        self.dispatch_log.push((signal.to_string(), timestamp));
        let mut count = 0;
        let names: Vec<String> = self.handlers.keys().cloned().collect();
        let mut sorted: Vec<(i32, String)> = names
            .iter()
            .filter_map(|n| {
                let h = &self.handlers[n];
                if h.signal == signal && h.enabled {
                    Some((h.priority, n.clone()))
                } else {
                    None
                }
            })
            .collect();
        sorted.sort_by(|a, b| b.0.cmp(&a.0));
        for (_, name) in sorted {
            if let Some(h) = self.handlers.get_mut(&name) {
                h.call_count += 1;
                count += 1;
            }
        }
        count
    }

    pub fn get(&self, name: &str) -> Option<&HandlerEntry> {
        self.handlers.get(name)
    }

    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }

    pub fn total_dispatched(&self) -> u64 {
        self.total_dispatched
    }

    pub fn dispatch_log_len(&self) -> usize {
        self.dispatch_log.len()
    }

    pub fn clear_log(&mut self) {
        self.dispatch_log.clear();
    }

    pub fn clear_all(&mut self) {
        self.handlers.clear();
        self.dispatch_log.clear();
        self.total_dispatched = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_signal_handler() -> SignalHandler {
    SignalHandler::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_dispatch() {
        let mut sh = new_signal_handler();
        sh.register("h1", "resize", 0);
        let count = sh.dispatch("resize", 100);
        assert_eq!(count, 1);
    }

    #[test]
    fn unregister_removes() {
        let mut sh = new_signal_handler();
        sh.register("h1", "exit", 0);
        assert!(sh.unregister("h1"));
        assert_eq!(sh.dispatch("exit", 0), 0);
    }

    #[test]
    fn disabled_handler_not_called() {
        let mut sh = new_signal_handler();
        sh.register("h1", "pause", 5);
        sh.set_enabled("h1", false);
        assert_eq!(sh.dispatch("pause", 1), 0);
    }

    #[test]
    fn re_enable_handler() {
        let mut sh = new_signal_handler();
        sh.register("h1", "resume", 1);
        sh.set_enabled("h1", false);
        sh.set_enabled("h1", true);
        assert_eq!(sh.dispatch("resume", 1), 1);
    }

    #[test]
    fn call_count_increments() {
        let mut sh = new_signal_handler();
        sh.register("h1", "tick", 0);
        sh.dispatch("tick", 1);
        sh.dispatch("tick", 2);
        assert_eq!(sh.get("h1").expect("should succeed").call_count, 2);
    }

    #[test]
    fn total_dispatched_increments() {
        let mut sh = new_signal_handler();
        sh.dispatch("x", 0);
        sh.dispatch("x", 1);
        assert_eq!(sh.total_dispatched(), 2);
    }

    #[test]
    fn handler_count() {
        let mut sh = new_signal_handler();
        sh.register("a", "s", 0);
        sh.register("b", "s", 0);
        assert_eq!(sh.handler_count(), 2);
    }

    #[test]
    fn wrong_signal_not_dispatched() {
        let mut sh = new_signal_handler();
        sh.register("h1", "usr1", 0);
        assert_eq!(sh.dispatch("usr2", 0), 0);
    }

    #[test]
    fn clear_all() {
        let mut sh = new_signal_handler();
        sh.register("h1", "s", 0);
        sh.dispatch("s", 0);
        sh.clear_all();
        assert!(sh.is_empty());
        assert_eq!(sh.total_dispatched(), 0);
    }

    #[test]
    fn clear_log() {
        let mut sh = new_signal_handler();
        sh.dispatch("s", 1);
        assert_eq!(sh.dispatch_log_len(), 1);
        sh.clear_log();
        assert_eq!(sh.dispatch_log_len(), 0);
    }
}
