// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! MQTT payload export stub — serialises mesh/animation data as MQTT message payloads.

/// MQTT QoS level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MqttQos {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

/// A single MQTT message stub.
#[derive(Debug, Clone)]
pub struct MqttMessage {
    pub topic: String,
    pub payload: Vec<u8>,
    pub qos: MqttQos,
    pub retain: bool,
}

/// An MQTT export session.
#[derive(Debug, Default)]
pub struct MqttExport {
    pub messages: Vec<MqttMessage>,
    pub broker_url: String,
}

/// Create a new MQTT export session.
pub fn new_mqtt_export(broker_url: &str) -> MqttExport {
    MqttExport {
        messages: Vec::new(),
        broker_url: broker_url.to_owned(),
    }
}

/// Add a text payload message.
pub fn add_mqtt_text(export: &mut MqttExport, topic: &str, text: &str, qos: MqttQos, retain: bool) {
    export.messages.push(MqttMessage {
        topic: topic.to_owned(),
        payload: text.as_bytes().to_vec(),
        qos,
        retain,
    });
}

/// Add a binary payload message.
pub fn add_mqtt_binary(export: &mut MqttExport, topic: &str, data: Vec<u8>, qos: MqttQos) {
    export.messages.push(MqttMessage {
        topic: topic.to_owned(),
        payload: data,
        qos,
        retain: false,
    });
}

/// Number of messages.
pub fn mqtt_message_count(export: &MqttExport) -> usize {
    export.messages.len()
}

/// Total payload bytes across all messages.
pub fn total_payload_bytes(export: &MqttExport) -> usize {
    export.messages.iter().map(|m| m.payload.len()).sum()
}

/// Find a message by topic.
pub fn find_mqtt_message<'a>(export: &'a MqttExport, topic: &str) -> Option<&'a MqttMessage> {
    export.messages.iter().find(|m| m.topic == topic)
}

/// Count messages with the retain flag set.
pub fn retained_message_count(export: &MqttExport) -> usize {
    export.messages.iter().filter(|m| m.retain).count()
}

/// QoS level name string.
pub fn qos_name(qos: MqttQos) -> &'static str {
    match qos {
        MqttQos::AtMostOnce => "at_most_once",
        MqttQos::AtLeastOnce => "at_least_once",
        MqttQos::ExactlyOnce => "exactly_once",
    }
}

/// Serialize export metadata to JSON-style string.
pub fn mqtt_export_to_json(export: &MqttExport) -> String {
    format!(
        r#"{{"broker":"{}", "message_count":{}, "total_bytes":{}}}"#,
        export.broker_url,
        mqtt_message_count(export),
        total_payload_bytes(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_has_no_messages() {
        /* fresh export has no messages */
        let e = new_mqtt_export("mqtt://localhost:1883");
        assert_eq!(mqtt_message_count(&e), 0);
    }

    #[test]
    fn add_text_increases_count() {
        /* adding a text message increments count */
        let mut e = new_mqtt_export("mqtt://localhost:1883");
        add_mqtt_text(&mut e, "mesh/pos", "hello", MqttQos::AtLeastOnce, false);
        assert_eq!(mqtt_message_count(&e), 1);
    }

    #[test]
    fn total_payload_bytes_correct() {
        /* 5-byte payload should be counted */
        let mut e = new_mqtt_export("mqtt://localhost");
        add_mqtt_text(&mut e, "t", "hello", MqttQos::AtMostOnce, false);
        assert_eq!(total_payload_bytes(&e), 5);
    }

    #[test]
    fn find_message_by_topic() {
        /* find returns the matching message */
        let mut e = new_mqtt_export("mqtt://localhost");
        add_mqtt_text(&mut e, "my/topic", "data", MqttQos::ExactlyOnce, false);
        assert!(find_mqtt_message(&e, "my/topic").is_some());
    }

    #[test]
    fn find_missing_topic_returns_none() {
        /* missing topic gives None */
        let e = new_mqtt_export("mqtt://localhost");
        assert!(find_mqtt_message(&e, "nope").is_none());
    }

    #[test]
    fn retained_count_correct() {
        /* only retained messages are counted */
        let mut e = new_mqtt_export("mqtt://localhost");
        add_mqtt_text(&mut e, "a", "x", MqttQos::AtMostOnce, true);
        add_mqtt_text(&mut e, "b", "y", MqttQos::AtMostOnce, false);
        assert_eq!(retained_message_count(&e), 1);
    }

    #[test]
    fn qos_name_at_most_once() {
        /* AtMostOnce maps to "at_most_once" */
        assert_eq!(qos_name(MqttQos::AtMostOnce), "at_most_once");
    }

    #[test]
    fn json_contains_broker() {
        /* JSON includes broker URL */
        let e = new_mqtt_export("mqtt://broker.example.com");
        assert!(mqtt_export_to_json(&e).contains("broker.example.com"));
    }

    #[test]
    fn add_binary_message_tracked() {
        /* binary message is tracked in total payload bytes */
        let mut e = new_mqtt_export("mqtt://localhost");
        add_mqtt_binary(&mut e, "data/raw", vec![0u8; 16], MqttQos::AtLeastOnce);
        assert_eq!(total_payload_bytes(&e), 16);
    }
}
