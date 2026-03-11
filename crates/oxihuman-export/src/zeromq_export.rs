// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! ZeroMQ message export stub — packages mesh/animation data as ZeroMQ frame sequences.

/// ZeroMQ socket pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ZmqPattern {
    #[default]
    PubSub,
    PushPull,
    ReqRep,
    Dealer,
    Router,
}

/// A ZeroMQ multi-part message stub.
#[derive(Debug, Clone)]
pub struct ZmqMessage {
    pub topic: Option<String>,
    pub frames: Vec<Vec<u8>>,
    pub pattern: ZmqPattern,
}

/// A ZeroMQ export session.
#[derive(Debug, Default)]
pub struct ZmqExport {
    pub messages: Vec<ZmqMessage>,
    pub endpoint: String,
    pub pattern: ZmqPattern,
}

/// Create a new ZeroMQ export session.
pub fn new_zmq_export(endpoint: &str, pattern: ZmqPattern) -> ZmqExport {
    ZmqExport {
        messages: Vec::new(),
        endpoint: endpoint.to_owned(),
        pattern,
    }
}

/// Add a pub-sub message with a topic prefix.
pub fn zmq_publish(export: &mut ZmqExport, topic: &str, data: Vec<u8>) {
    export.messages.push(ZmqMessage {
        topic: Some(topic.to_owned()),
        frames: vec![topic.as_bytes().to_vec(), data],
        pattern: ZmqPattern::PubSub,
    });
}

/// Add a push message (no topic).
pub fn zmq_push(export: &mut ZmqExport, data: Vec<u8>) {
    export.messages.push(ZmqMessage {
        topic: None,
        frames: vec![data],
        pattern: ZmqPattern::PushPull,
    });
}

/// Number of messages.
pub fn zmq_message_count(export: &ZmqExport) -> usize {
    export.messages.len()
}

/// Total frame bytes across all messages.
pub fn total_zmq_bytes(export: &ZmqExport) -> usize {
    export
        .messages
        .iter()
        .flat_map(|m| &m.frames)
        .map(|f| f.len())
        .sum()
}

/// Count messages for a specific topic.
pub fn messages_for_topic(export: &ZmqExport, topic: &str) -> usize {
    export
        .messages
        .iter()
        .filter(|m| m.topic.as_deref() == Some(topic))
        .count()
}

/// Pattern name string.
pub fn pattern_name(p: ZmqPattern) -> &'static str {
    match p {
        ZmqPattern::PubSub => "pub_sub",
        ZmqPattern::PushPull => "push_pull",
        ZmqPattern::ReqRep => "req_rep",
        ZmqPattern::Dealer => "dealer",
        ZmqPattern::Router => "router",
    }
}

/// Serialize to JSON-style string.
pub fn zmq_export_to_json(export: &ZmqExport) -> String {
    format!(
        r#"{{"endpoint":"{}", "pattern":"{}", "message_count":{}}}"#,
        export.endpoint,
        pattern_name(export.pattern),
        zmq_message_count(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        /* fresh export has no messages */
        let e = new_zmq_export("tcp://localhost:5555", ZmqPattern::PubSub);
        assert_eq!(zmq_message_count(&e), 0);
    }

    #[test]
    fn publish_increments_count() {
        /* publishing increases count */
        let mut e = new_zmq_export("tcp://localhost:5555", ZmqPattern::PubSub);
        zmq_publish(&mut e, "mesh", vec![1, 2, 3]);
        assert_eq!(zmq_message_count(&e), 1);
    }

    #[test]
    fn push_increments_count() {
        /* push also increases count */
        let mut e = new_zmq_export("tcp://localhost:5556", ZmqPattern::PushPull);
        zmq_push(&mut e, vec![0; 4]);
        assert_eq!(zmq_message_count(&e), 1);
    }

    #[test]
    fn total_bytes_correct() {
        /* frame bytes should be summed */
        let mut e = new_zmq_export("tcp://localhost:5555", ZmqPattern::PubSub);
        zmq_push(&mut e, vec![0; 8]);
        assert!(total_zmq_bytes(&e) >= 8);
    }

    #[test]
    fn messages_for_topic_counted() {
        /* only messages with the given topic are counted */
        let mut e = new_zmq_export("tcp://localhost:5555", ZmqPattern::PubSub);
        zmq_publish(&mut e, "mesh.pos", vec![1]);
        zmq_publish(&mut e, "mesh.normal", vec![2]);
        assert_eq!(messages_for_topic(&e, "mesh.pos"), 1);
    }

    #[test]
    fn pattern_name_pub_sub() {
        /* PubSub pattern name is "pub_sub" */
        assert_eq!(pattern_name(ZmqPattern::PubSub), "pub_sub");
    }

    #[test]
    fn pattern_name_push_pull() {
        /* PushPull name is "push_pull" */
        assert_eq!(pattern_name(ZmqPattern::PushPull), "push_pull");
    }

    #[test]
    fn json_contains_endpoint() {
        /* JSON includes endpoint */
        let e = new_zmq_export("tcp://zmq.example.com:5555", ZmqPattern::PubSub);
        assert!(zmq_export_to_json(&e).contains("zmq.example.com"));
    }

    #[test]
    fn published_message_has_topic() {
        /* published message stores the topic */
        let mut e = new_zmq_export("tcp://localhost:5555", ZmqPattern::PubSub);
        zmq_publish(&mut e, "my.topic", vec![]);
        assert_eq!(e.messages[0].topic.as_deref(), Some("my.topic"));
    }
}
