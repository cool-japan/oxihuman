// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Routes messages to named channels based on topic patterns.

use std::collections::HashMap;

/// A message with a topic and payload.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RoutedMessage {
    pub topic: String,
    pub payload: String,
}

/// Routes messages to named channels.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChannelRouter {
    /// channel_name -> list of topic prefixes it subscribes to
    routes: HashMap<String, Vec<String>>,
    /// channel_name -> queued messages
    queues: HashMap<String, Vec<RoutedMessage>>,
    total_routed: u64,
}

#[allow(dead_code)]
impl ChannelRouter {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            queues: HashMap::new(),
            total_routed: 0,
        }
    }

    pub fn add_channel(&mut self, name: &str) {
        self.routes.entry(name.to_string()).or_default();
        self.queues.entry(name.to_string()).or_default();
    }

    pub fn subscribe(&mut self, channel: &str, topic_prefix: &str) {
        self.routes
            .entry(channel.to_string())
            .or_default()
            .push(topic_prefix.to_string());
    }

    pub fn route(&mut self, topic: &str, payload: &str) {
        let msg = RoutedMessage {
            topic: topic.to_string(),
            payload: payload.to_string(),
        };
        for (ch_name, prefixes) in &self.routes {
            if prefixes.iter().any(|p| topic.starts_with(p)) {
                self.queues
                    .entry(ch_name.clone())
                    .or_default()
                    .push(msg.clone());
                self.total_routed += 1;
            }
        }
    }

    pub fn drain_channel(&mut self, channel: &str) -> Vec<RoutedMessage> {
        self.queues
            .get_mut(channel)
            .map(std::mem::take)
            .unwrap_or_default()
    }

    pub fn channel_count(&self) -> usize {
        self.routes.len()
    }

    pub fn pending_count(&self, channel: &str) -> usize {
        self.queues.get(channel).map_or(0, |q| q.len())
    }

    pub fn total_routed(&self) -> u64 {
        self.total_routed
    }

    pub fn has_channel(&self, name: &str) -> bool {
        self.routes.contains_key(name)
    }

    pub fn remove_channel(&mut self, name: &str) {
        self.routes.remove(name);
        self.queues.remove(name);
    }

    pub fn clear_all(&mut self) {
        for q in self.queues.values_mut() {
            q.clear();
        }
    }
}

impl Default for ChannelRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_router_empty() {
        let r = ChannelRouter::new();
        assert_eq!(r.channel_count(), 0);
    }

    #[test]
    fn add_channel_and_subscribe() {
        let mut r = ChannelRouter::new();
        r.add_channel("log");
        r.subscribe("log", "system.");
        assert!(r.has_channel("log"));
    }

    #[test]
    fn route_to_matching_channel() {
        let mut r = ChannelRouter::new();
        r.add_channel("log");
        r.subscribe("log", "sys.");
        r.route("sys.info", "hello");
        assert_eq!(r.pending_count("log"), 1);
    }

    #[test]
    fn route_no_match() {
        let mut r = ChannelRouter::new();
        r.add_channel("log");
        r.subscribe("log", "sys.");
        r.route("net.error", "data");
        assert_eq!(r.pending_count("log"), 0);
    }

    #[test]
    fn drain_channel_empties() {
        let mut r = ChannelRouter::new();
        r.add_channel("ch");
        r.subscribe("ch", "t.");
        r.route("t.1", "a");
        let msgs = r.drain_channel("ch");
        assert_eq!(msgs.len(), 1);
        assert_eq!(r.pending_count("ch"), 0);
    }

    #[test]
    fn total_routed_increments() {
        let mut r = ChannelRouter::new();
        r.add_channel("a");
        r.subscribe("a", "x");
        r.route("x1", "p");
        r.route("x2", "q");
        assert_eq!(r.total_routed(), 2);
    }

    #[test]
    fn remove_channel() {
        let mut r = ChannelRouter::new();
        r.add_channel("tmp");
        r.remove_channel("tmp");
        assert!(!r.has_channel("tmp"));
    }

    #[test]
    fn clear_all_queues() {
        let mut r = ChannelRouter::new();
        r.add_channel("c");
        r.subscribe("c", "");
        r.route("anything", "data");
        r.clear_all();
        assert_eq!(r.pending_count("c"), 0);
    }

    #[test]
    fn multiple_channels_receive() {
        let mut r = ChannelRouter::new();
        r.add_channel("a");
        r.add_channel("b");
        r.subscribe("a", "shared.");
        r.subscribe("b", "shared.");
        r.route("shared.msg", "x");
        assert_eq!(r.pending_count("a"), 1);
        assert_eq!(r.pending_count("b"), 1);
    }

    #[test]
    fn default_is_empty() {
        let r = ChannelRouter::default();
        assert_eq!(r.channel_count(), 0);
    }
}
