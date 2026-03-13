// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Pub/sub topic bus stub — topics, subscriptions, and message delivery.

/// A message on a topic.
#[derive(Clone, Debug)]
pub struct TopicMessage {
    pub topic: String,
    pub payload: String,
    pub sequence: u64,
}

/// A subscription entry.
#[derive(Clone, Debug)]
pub struct Subscription {
    pub id: u64,
    pub topic: String,
    pub subscriber_name: String,
}

/// Configuration for the pub/sub bus.
#[derive(Clone, Debug)]
pub struct PubSubConfig {
    pub max_topics: usize,
    pub max_messages_per_topic: usize,
}

impl Default for PubSubConfig {
    fn default() -> Self {
        Self {
            max_topics: 64,
            max_messages_per_topic: 256,
        }
    }
}

/// An in-memory pub/sub topic bus.
pub struct PubSubBus {
    pub config: PubSubConfig,
    subscriptions: Vec<Subscription>,
    messages: Vec<TopicMessage>,
    next_sub_id: u64,
    next_seq: u64,
}

/// Creates a new pub/sub bus.
pub fn new_pubsub_bus(config: PubSubConfig) -> PubSubBus {
    PubSubBus {
        config,
        subscriptions: Vec::new(),
        messages: Vec::new(),
        next_sub_id: 1,
        next_seq: 1,
    }
}

/// Subscribes a named subscriber to a topic, returning the subscription ID.
pub fn subscribe(bus: &mut PubSubBus, topic: &str, subscriber_name: &str) -> u64 {
    let id = bus.next_sub_id;
    bus.next_sub_id += 1;
    bus.subscriptions.push(Subscription {
        id,
        topic: topic.into(),
        subscriber_name: subscriber_name.into(),
    });
    id
}

/// Unsubscribes by subscription ID, returning true if found.
pub fn unsubscribe(bus: &mut PubSubBus, sub_id: u64) -> bool {
    let before = bus.subscriptions.len();
    bus.subscriptions.retain(|s| s.id != sub_id);
    bus.subscriptions.len() < before
}

/// Publishes a message to a topic, returning the sequence number.
pub fn publish(bus: &mut PubSubBus, topic: &str, payload: &str) -> u64 {
    let seq = bus.next_seq;
    bus.next_seq += 1;
    let msg = TopicMessage {
        topic: topic.into(),
        payload: payload.into(),
        sequence: seq,
    };
    /* trim oldest if over capacity */
    let count = bus.messages.iter().filter(|m| m.topic == topic).count();
    if count >= bus.config.max_messages_per_topic {
        let mut removed = false;
        bus.messages.retain(|m| {
            if !removed && m.topic == topic {
                removed = true;
                false
            } else {
                true
            }
        });
    }
    bus.messages.push(msg);
    seq
}

/// Returns all messages for a topic, in sequence order.
pub fn messages_for_topic<'a>(bus: &'a PubSubBus, topic: &str) -> Vec<&'a TopicMessage> {
    bus.messages.iter().filter(|m| m.topic == topic).collect()
}

/// Returns the number of active subscribers to a topic.
pub fn subscriber_count(bus: &PubSubBus, topic: &str) -> usize {
    bus.subscriptions
        .iter()
        .filter(|s| s.topic == topic)
        .count()
}

/// Clears all messages for a given topic.
pub fn clear_topic(bus: &mut PubSubBus, topic: &str) {
    bus.messages.retain(|m| m.topic != topic);
}

impl PubSubBus {
    /// Creates a new bus with default config.
    pub fn new(config: PubSubConfig) -> Self {
        new_pubsub_bus(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bus() -> PubSubBus {
        new_pubsub_bus(PubSubConfig::default())
    }

    #[test]
    fn test_subscribe_returns_unique_ids() {
        let mut bus = make_bus();
        let id1 = subscribe(&mut bus, "t", "s1");
        let id2 = subscribe(&mut bus, "t", "s2");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_subscriber_count_tracks_subs() {
        let mut bus = make_bus();
        subscribe(&mut bus, "topic_a", "alice");
        subscribe(&mut bus, "topic_a", "bob");
        assert_eq!(subscriber_count(&bus, "topic_a"), 2);
    }

    #[test]
    fn test_unsubscribe_removes_entry() {
        let mut bus = make_bus();
        let id = subscribe(&mut bus, "t", "s");
        assert!(unsubscribe(&mut bus, id));
        assert_eq!(subscriber_count(&bus, "t"), 0);
    }

    #[test]
    fn test_unsubscribe_nonexistent_returns_false() {
        let mut bus = make_bus();
        assert!(!unsubscribe(&mut bus, 999));
    }

    #[test]
    fn test_publish_stores_message() {
        let mut bus = make_bus();
        publish(&mut bus, "news", "hello world");
        assert_eq!(messages_for_topic(&bus, "news").len(), 1);
    }

    #[test]
    fn test_publish_returns_incrementing_sequence() {
        let mut bus = make_bus();
        let s1 = publish(&mut bus, "t", "a");
        let s2 = publish(&mut bus, "t", "b");
        assert!(s2 > s1);
    }

    #[test]
    fn test_clear_topic_removes_messages() {
        let mut bus = make_bus();
        publish(&mut bus, "news", "msg1");
        publish(&mut bus, "news", "msg2");
        clear_topic(&mut bus, "news");
        assert!(messages_for_topic(&bus, "news").is_empty());
    }

    #[test]
    fn test_messages_for_wrong_topic_empty() {
        let mut bus = make_bus();
        publish(&mut bus, "topicX", "data");
        assert!(messages_for_topic(&bus, "topicY").is_empty());
    }

    #[test]
    fn test_capacity_trims_oldest_message() {
        let mut bus = new_pubsub_bus(PubSubConfig {
            max_topics: 8,
            max_messages_per_topic: 2,
        });
        publish(&mut bus, "t", "first");
        publish(&mut bus, "t", "second");
        publish(&mut bus, "t", "third"); /* should evict "first" */
        let msgs = messages_for_topic(&bus, "t");
        assert_eq!(msgs.len(), 2);
        assert!(msgs.iter().any(|m| m.payload == "second"));
        assert!(msgs.iter().any(|m| m.payload == "third"));
    }
}
