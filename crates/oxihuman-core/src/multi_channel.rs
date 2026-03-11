#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::HashMap;

/// Identifies a channel within a MultiChannel.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelId(pub u32);

/// A multi-channel message router.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MultiChannel {
    channels: HashMap<u32, Vec<String>>,
    names: HashMap<u32, String>,
    next_id: u32,
}

#[allow(dead_code)]
pub fn new_multi_channel() -> MultiChannel {
    MultiChannel {
        channels: HashMap::new(),
        names: HashMap::new(),
        next_id: 0,
    }
}

#[allow(dead_code)]
pub fn send_to_channel(mc: &mut MultiChannel, name: &str, msg: &str) -> ChannelId {
    let id = if let Some((&id, _)) = mc.names.iter().find(|(_, n)| n.as_str() == name) {
        id
    } else {
        let id = mc.next_id;
        mc.next_id += 1;
        mc.names.insert(id, name.to_string());
        id
    };
    mc.channels.entry(id).or_default().push(msg.to_string());
    ChannelId(id)
}

#[allow(dead_code)]
pub fn receive_from_channel(mc: &mut MultiChannel, ch: ChannelId) -> Option<String> {
    mc.channels.get_mut(&ch.0).and_then(|v| {
        if v.is_empty() {
            None
        } else {
            Some(v.remove(0))
        }
    })
}

#[allow(dead_code)]
pub fn channel_count_mc(mc: &MultiChannel) -> usize {
    mc.names.len()
}

#[allow(dead_code)]
pub fn channel_is_empty_mc(mc: &MultiChannel, ch: ChannelId) -> bool {
    mc.channels.get(&ch.0).is_none_or(|v| v.is_empty())
}

#[allow(dead_code)]
pub fn channel_len_mc(mc: &MultiChannel, ch: ChannelId) -> usize {
    mc.channels.get(&ch.0).map_or(0, |v| v.len())
}

#[allow(dead_code)]
pub fn clear_channel(mc: &mut MultiChannel, ch: ChannelId) {
    if let Some(v) = mc.channels.get_mut(&ch.0) {
        v.clear();
    }
}

#[allow(dead_code)]
pub fn channel_names(mc: &MultiChannel) -> Vec<String> {
    mc.names.values().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_multi_channel() {
        let mc = new_multi_channel();
        assert_eq!(channel_count_mc(&mc), 0);
    }

    #[test]
    fn test_send_receive() {
        let mut mc = new_multi_channel();
        let ch = send_to_channel(&mut mc, "events", "hello");
        assert_eq!(receive_from_channel(&mut mc, ch), Some("hello".to_string()));
    }

    #[test]
    fn test_channel_count() {
        let mut mc = new_multi_channel();
        send_to_channel(&mut mc, "a", "1");
        send_to_channel(&mut mc, "b", "2");
        assert_eq!(channel_count_mc(&mc), 2);
    }

    #[test]
    fn test_channel_is_empty() {
        let mut mc = new_multi_channel();
        let ch = send_to_channel(&mut mc, "a", "1");
        assert!(!channel_is_empty_mc(&mc, ch));
        receive_from_channel(&mut mc, ch);
        assert!(channel_is_empty_mc(&mc, ch));
    }

    #[test]
    fn test_channel_len() {
        let mut mc = new_multi_channel();
        let ch = send_to_channel(&mut mc, "a", "1");
        send_to_channel(&mut mc, "a", "2");
        assert_eq!(channel_len_mc(&mc, ch), 2);
    }

    #[test]
    fn test_clear_channel() {
        let mut mc = new_multi_channel();
        let ch = send_to_channel(&mut mc, "a", "1");
        send_to_channel(&mut mc, "a", "2");
        clear_channel(&mut mc, ch);
        assert!(channel_is_empty_mc(&mc, ch));
    }

    #[test]
    fn test_channel_names() {
        let mut mc = new_multi_channel();
        send_to_channel(&mut mc, "events", "1");
        send_to_channel(&mut mc, "logs", "2");
        let names = channel_names(&mc);
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn test_receive_empty() {
        let mut mc = new_multi_channel();
        assert!(receive_from_channel(&mut mc, ChannelId(999)).is_none());
    }

    #[test]
    fn test_same_channel_name() {
        let mut mc = new_multi_channel();
        let c1 = send_to_channel(&mut mc, "x", "1");
        let c2 = send_to_channel(&mut mc, "x", "2");
        assert_eq!(c1, c2);
        assert_eq!(channel_len_mc(&mc, c1), 2);
    }

    #[test]
    fn test_fifo_order() {
        let mut mc = new_multi_channel();
        let ch = send_to_channel(&mut mc, "q", "first");
        send_to_channel(&mut mc, "q", "second");
        assert_eq!(receive_from_channel(&mut mc, ch), Some("first".to_string()));
        assert_eq!(receive_from_channel(&mut mc, ch), Some("second".to_string()));
    }
}
