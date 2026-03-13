#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Named channel message bus.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChannelBus {
    channels: HashMap<String, Vec<String>>,
}

#[allow(dead_code)]
pub fn new_channel_bus() -> ChannelBus {
    ChannelBus {
        channels: HashMap::new(),
    }
}

#[allow(dead_code)]
pub fn create_channel(bus: &mut ChannelBus, name: &str) {
    bus.channels.entry(name.to_string()).or_default();
}

#[allow(dead_code)]
pub fn send_on_channel(bus: &mut ChannelBus, name: &str, msg: &str) -> bool {
    if let Some(ch) = bus.channels.get_mut(name) {
        ch.push(msg.to_string());
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn receive_on_channel(bus: &mut ChannelBus, name: &str) -> Option<String> {
    bus.channels.get_mut(name).and_then(|ch| {
        if !ch.is_empty() {
            Some(ch.remove(0))
        } else {
            None
        }
    })
}

#[allow(dead_code)]
pub fn channel_count_cb(bus: &ChannelBus) -> usize {
    bus.channels.len()
}

#[allow(dead_code)]
pub fn channel_names_cb(bus: &ChannelBus) -> Vec<String> {
    bus.channels.keys().cloned().collect()
}

#[allow(dead_code)]
pub fn bus_to_json(bus: &ChannelBus) -> String {
    let entries: Vec<String> = bus
        .channels
        .iter()
        .map(|(k, v)| format!(r#""{}":{}msgs"#, k, v.len()))
        .collect();
    format!("{{{}}}", entries.join(","))
}

#[allow(dead_code)]
pub fn bus_clear(bus: &mut ChannelBus) {
    bus.channels.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_channel_bus() {
        let bus = new_channel_bus();
        assert_eq!(channel_count_cb(&bus), 0);
    }

    #[test]
    fn test_create_channel() {
        let mut bus = new_channel_bus();
        create_channel(&mut bus, "ch1");
        assert_eq!(channel_count_cb(&bus), 1);
    }

    #[test]
    fn test_send_receive() {
        let mut bus = new_channel_bus();
        create_channel(&mut bus, "ch1");
        assert!(send_on_channel(&mut bus, "ch1", "hello"));
        assert_eq!(receive_on_channel(&mut bus, "ch1"), Some("hello".to_string()));
    }

    #[test]
    fn test_send_to_nonexistent() {
        let mut bus = new_channel_bus();
        assert!(!send_on_channel(&mut bus, "nope", "msg"));
    }

    #[test]
    fn test_receive_empty() {
        let mut bus = new_channel_bus();
        create_channel(&mut bus, "ch1");
        assert_eq!(receive_on_channel(&mut bus, "ch1"), None);
    }

    #[test]
    fn test_channel_names() {
        let mut bus = new_channel_bus();
        create_channel(&mut bus, "a");
        create_channel(&mut bus, "b");
        let names = channel_names_cb(&bus);
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn test_bus_clear() {
        let mut bus = new_channel_bus();
        create_channel(&mut bus, "a");
        bus_clear(&mut bus);
        assert_eq!(channel_count_cb(&bus), 0);
    }

    #[test]
    fn test_bus_to_json() {
        let bus = new_channel_bus();
        let json = bus_to_json(&bus);
        assert!(json.starts_with('{'));
    }

    #[test]
    fn test_fifo_order() {
        let mut bus = new_channel_bus();
        create_channel(&mut bus, "q");
        send_on_channel(&mut bus, "q", "first");
        send_on_channel(&mut bus, "q", "second");
        assert_eq!(receive_on_channel(&mut bus, "q"), Some("first".to_string()));
        assert_eq!(receive_on_channel(&mut bus, "q"), Some("second".to_string()));
    }

    #[test]
    fn test_multiple_channels() {
        let mut bus = new_channel_bus();
        create_channel(&mut bus, "a");
        create_channel(&mut bus, "b");
        send_on_channel(&mut bus, "a", "ma");
        send_on_channel(&mut bus, "b", "mb");
        assert_eq!(receive_on_channel(&mut bus, "a"), Some("ma".to_string()));
        assert_eq!(receive_on_channel(&mut bus, "b"), Some("mb".to_string()));
    }
}
