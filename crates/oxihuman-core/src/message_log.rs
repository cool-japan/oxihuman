// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Message log: tagged message queue with filtering and JSON export.

/// Message priority.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
#[allow(dead_code)]
pub enum MsgPriority {
    Low,
    Normal,
    High,
}

/// A single log message.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Message {
    pub id: u64,
    pub tag: String,
    pub text: String,
    pub priority: MsgPriority,
}

/// Message log.
#[derive(Debug)]
#[allow(dead_code)]
pub struct MessageLog {
    messages: Vec<Message>,
    next_id: u64,
    capacity: usize,
}

/// Create a new MessageLog with given capacity.
#[allow(dead_code)]
pub fn new_message_log(capacity: usize) -> MessageLog {
    MessageLog {
        messages: Vec::new(),
        next_id: 1,
        capacity,
    }
}

/// Append a message; evicts oldest if at capacity.
#[allow(dead_code)]
pub fn ml_push(log: &mut MessageLog, tag: &str, text: &str, priority: MsgPriority) -> u64 {
    if log.messages.len() >= log.capacity && !log.messages.is_empty() {
        log.messages.remove(0);
    }
    let id = log.next_id;
    log.next_id += 1;
    log.messages.push(Message {
        id,
        tag: tag.to_string(),
        text: text.to_string(),
        priority,
    });
    id
}

/// Get message by id.
#[allow(dead_code)]
pub fn ml_get(log: &MessageLog, id: u64) -> Option<&Message> {
    log.messages.iter().find(|m| m.id == id)
}

/// Messages with a given tag.
#[allow(dead_code)]
pub fn ml_by_tag<'a>(log: &'a MessageLog, tag: &str) -> Vec<&'a Message> {
    log.messages.iter().filter(|m| m.tag == tag).collect()
}

/// Messages at or above a priority.
#[allow(dead_code)]
pub fn ml_by_priority(log: &MessageLog, min: MsgPriority) -> Vec<&Message> {
    log.messages.iter().filter(|m| m.priority >= min).collect()
}

/// Total message count.
#[allow(dead_code)]
pub fn ml_len(log: &MessageLog) -> usize {
    log.messages.len()
}

/// Clear all messages.
#[allow(dead_code)]
pub fn ml_clear(log: &mut MessageLog) {
    log.messages.clear();
}

/// Latest message.
#[allow(dead_code)]
pub fn ml_last(log: &MessageLog) -> Option<&Message> {
    log.messages.last()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn ml_to_json(log: &MessageLog) -> String {
    let items: Vec<String> = log
        .messages
        .iter()
        .map(|m| format!(r#"{{"id":{},"tag":"{}","text":"{}"}}"#, m.id, m.tag, m.text))
        .collect();
    format!("[{}]", items.join(","))
}

/// Remove messages with a given tag.
#[allow(dead_code)]
pub fn ml_remove_tag(log: &mut MessageLog, tag: &str) {
    log.messages.retain(|m| m.tag != tag);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_len() {
        let mut log = new_message_log(10);
        ml_push(&mut log, "sys", "hello", MsgPriority::Normal);
        assert_eq!(ml_len(&log), 1);
    }

    #[test]
    fn test_capacity_eviction() {
        let mut log = new_message_log(3);
        let id1 = ml_push(&mut log, "a", "1", MsgPriority::Low);
        ml_push(&mut log, "b", "2", MsgPriority::Low);
        ml_push(&mut log, "c", "3", MsgPriority::Low);
        ml_push(&mut log, "d", "4", MsgPriority::Low);
        assert_eq!(ml_len(&log), 3);
        assert!(ml_get(&log, id1).is_none());
    }

    #[test]
    fn test_get_by_id() {
        let mut log = new_message_log(10);
        let id = ml_push(&mut log, "net", "connect", MsgPriority::High);
        assert!(ml_get(&log, id).is_some_and(|m| m.tag == "net"));
    }

    #[test]
    fn test_by_tag() {
        let mut log = new_message_log(10);
        ml_push(&mut log, "io", "read", MsgPriority::Normal);
        ml_push(&mut log, "net", "send", MsgPriority::Normal);
        assert_eq!(ml_by_tag(&log, "io").len(), 1);
    }

    #[test]
    fn test_by_priority() {
        let mut log = new_message_log(10);
        ml_push(&mut log, "a", "x", MsgPriority::Low);
        ml_push(&mut log, "b", "y", MsgPriority::High);
        assert_eq!(ml_by_priority(&log, MsgPriority::High).len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut log = new_message_log(10);
        ml_push(&mut log, "t", "m", MsgPriority::Normal);
        ml_clear(&mut log);
        assert_eq!(ml_len(&log), 0);
    }

    #[test]
    fn test_last() {
        let mut log = new_message_log(10);
        ml_push(&mut log, "a", "first", MsgPriority::Low);
        ml_push(&mut log, "b", "last", MsgPriority::High);
        assert!(ml_last(&log).is_some_and(|m| m.text == "last"));
    }

    #[test]
    fn test_json() {
        let mut log = new_message_log(10);
        ml_push(&mut log, "sys", "ok", MsgPriority::Normal);
        let j = ml_to_json(&log);
        assert!(j.contains("sys"));
    }

    #[test]
    fn test_remove_tag() {
        let mut log = new_message_log(10);
        ml_push(&mut log, "del", "x", MsgPriority::Normal);
        ml_push(&mut log, "keep", "y", MsgPriority::Normal);
        ml_remove_tag(&mut log, "del");
        assert_eq!(ml_len(&log), 1);
    }
}
