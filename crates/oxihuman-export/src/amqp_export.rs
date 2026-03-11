// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! AMQP message export stub — produces AMQP-compatible message envelopes for mesh data.

/// AMQP delivery mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmqpDeliveryMode {
    NonPersistent,
    Persistent,
}

/// A single AMQP message stub.
#[derive(Debug, Clone)]
pub struct AmqpMessage {
    pub exchange: String,
    pub routing_key: String,
    pub body: Vec<u8>,
    pub delivery_mode: AmqpDeliveryMode,
    pub content_type: String,
}

/// An AMQP export session.
#[derive(Debug, Default)]
pub struct AmqpExport {
    pub messages: Vec<AmqpMessage>,
    pub connection_url: String,
}

/// Create a new AMQP export session.
pub fn new_amqp_export(connection_url: &str) -> AmqpExport {
    AmqpExport {
        messages: Vec::new(),
        connection_url: connection_url.to_owned(),
    }
}

/// Publish a text message.
pub fn publish_amqp_text(
    export: &mut AmqpExport,
    exchange: &str,
    routing_key: &str,
    body: &str,
    persistent: bool,
) {
    export.messages.push(AmqpMessage {
        exchange: exchange.to_owned(),
        routing_key: routing_key.to_owned(),
        body: body.as_bytes().to_vec(),
        delivery_mode: if persistent {
            AmqpDeliveryMode::Persistent
        } else {
            AmqpDeliveryMode::NonPersistent
        },
        content_type: "text/plain".to_owned(),
    });
}

/// Publish a JSON payload message.
pub fn publish_amqp_json(export: &mut AmqpExport, exchange: &str, routing_key: &str, json: &str) {
    export.messages.push(AmqpMessage {
        exchange: exchange.to_owned(),
        routing_key: routing_key.to_owned(),
        body: json.as_bytes().to_vec(),
        delivery_mode: AmqpDeliveryMode::Persistent,
        content_type: "application/json".to_owned(),
    });
}

/// Number of messages.
pub fn amqp_message_count(export: &AmqpExport) -> usize {
    export.messages.len()
}

/// Count persistent messages.
pub fn persistent_message_count(export: &AmqpExport) -> usize {
    export
        .messages
        .iter()
        .filter(|m| m.delivery_mode == AmqpDeliveryMode::Persistent)
        .count()
}

/// Find a message by routing key.
pub fn find_by_routing_key<'a>(export: &'a AmqpExport, key: &str) -> Option<&'a AmqpMessage> {
    export.messages.iter().find(|m| m.routing_key == key)
}

/// Total body bytes.
pub fn total_amqp_bytes(export: &AmqpExport) -> usize {
    export.messages.iter().map(|m| m.body.len()).sum()
}

/// Serialize metadata to JSON-style string.
pub fn amqp_export_to_json(export: &AmqpExport) -> String {
    format!(
        r#"{{"url":"{}", "message_count":{}, "total_bytes":{}}}"#,
        export.connection_url,
        amqp_message_count(export),
        total_amqp_bytes(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        /* fresh export has no messages */
        let e = new_amqp_export("amqp://localhost");
        assert_eq!(amqp_message_count(&e), 0);
    }

    #[test]
    fn publish_text_increments_count() {
        /* publishing a text message increments count */
        let mut e = new_amqp_export("amqp://localhost");
        publish_amqp_text(&mut e, "ex", "rk", "body", false);
        assert_eq!(amqp_message_count(&e), 1);
    }

    #[test]
    fn persistent_count_correct() {
        /* persistent flag is tracked */
        let mut e = new_amqp_export("amqp://localhost");
        publish_amqp_text(&mut e, "ex", "rk1", "a", true);
        publish_amqp_text(&mut e, "ex", "rk2", "b", false);
        assert_eq!(persistent_message_count(&e), 1);
    }

    #[test]
    fn find_by_routing_key_success() {
        /* find returns message with matching routing key */
        let mut e = new_amqp_export("amqp://localhost");
        publish_amqp_text(&mut e, "ex", "mesh.pos", "data", false);
        assert!(find_by_routing_key(&e, "mesh.pos").is_some());
    }

    #[test]
    fn find_missing_key_returns_none() {
        /* missing key returns None */
        let e = new_amqp_export("amqp://localhost");
        assert!(find_by_routing_key(&e, "gone").is_none());
    }

    #[test]
    fn total_bytes_correct() {
        /* 4 byte body is tracked */
        let mut e = new_amqp_export("amqp://localhost");
        publish_amqp_text(&mut e, "ex", "rk", "test", false);
        assert_eq!(total_amqp_bytes(&e), 4);
    }

    #[test]
    fn json_content_type_set_for_json_publish() {
        /* JSON publish sets application/json content type */
        let mut e = new_amqp_export("amqp://localhost");
        publish_amqp_json(&mut e, "ex", "rk.json", r#"{"a":1}"#);
        assert_eq!(e.messages[0].content_type, "application/json");
    }

    #[test]
    fn json_export_contains_url() {
        /* serialised JSON includes the connection URL */
        let e = new_amqp_export("amqp://broker.example.com");
        assert!(amqp_export_to_json(&e).contains("broker.example.com"));
    }

    #[test]
    fn multiple_messages_counted() {
        /* three messages gives count 3 */
        let mut e = new_amqp_export("amqp://localhost");
        for i in 0..3 {
            publish_amqp_text(&mut e, "ex", &format!("rk.{i}"), "x", false);
        }
        assert_eq!(amqp_message_count(&e), 3);
    }
}
