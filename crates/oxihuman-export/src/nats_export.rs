// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! NATS message export stub — packages mesh data as NATS publish payloads.

/// A NATS message stub.
#[derive(Debug, Clone)]
pub struct NatsMessage {
    pub subject: String,
    pub reply_to: Option<String>,
    pub payload: Vec<u8>,
    pub headers: Vec<(String, String)>,
}

/// A NATS export session.
#[derive(Debug, Default)]
pub struct NatsExport {
    pub messages: Vec<NatsMessage>,
    pub server_url: String,
}

/// Create a new NATS export session.
pub fn new_nats_export(server_url: &str) -> NatsExport {
    NatsExport {
        messages: Vec::new(),
        server_url: server_url.to_owned(),
    }
}

/// Publish a text message to a subject.
pub fn nats_publish_text(export: &mut NatsExport, subject: &str, text: &str) {
    export.messages.push(NatsMessage {
        subject: subject.to_owned(),
        reply_to: None,
        payload: text.as_bytes().to_vec(),
        headers: Vec::new(),
    });
}

/// Publish a binary message with an optional reply subject.
pub fn nats_publish_binary(
    export: &mut NatsExport,
    subject: &str,
    data: Vec<u8>,
    reply_to: Option<&str>,
) {
    export.messages.push(NatsMessage {
        subject: subject.to_owned(),
        reply_to: reply_to.map(str::to_owned),
        payload: data,
        headers: Vec::new(),
    });
}

/// Add a header to the last published message.
pub fn nats_add_header(export: &mut NatsExport, key: &str, value: &str) {
    if let Some(msg) = export.messages.last_mut() {
        msg.headers.push((key.to_owned(), value.to_owned()));
    }
}

/// Number of messages.
pub fn nats_message_count(export: &NatsExport) -> usize {
    export.messages.len()
}

/// Find the first message for a given subject.
pub fn find_nats_message<'a>(export: &'a NatsExport, subject: &str) -> Option<&'a NatsMessage> {
    export.messages.iter().find(|m| m.subject == subject)
}

/// Total payload bytes.
pub fn total_nats_bytes(export: &NatsExport) -> usize {
    export.messages.iter().map(|m| m.payload.len()).sum()
}

/// Count messages that have a reply-to subject set.
pub fn request_reply_count(export: &NatsExport) -> usize {
    export
        .messages
        .iter()
        .filter(|m| m.reply_to.is_some())
        .count()
}

/// Serialize metadata to JSON-style string.
pub fn nats_export_to_json(export: &NatsExport) -> String {
    format!(
        r#"{{"server":"{}", "message_count":{}, "total_bytes":{}}}"#,
        export.server_url,
        nats_message_count(export),
        total_nats_bytes(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_is_empty() {
        /* fresh export has no messages */
        let e = new_nats_export("nats://localhost:4222");
        assert_eq!(nats_message_count(&e), 0);
    }

    #[test]
    fn publish_text_increments_count() {
        /* publishing increases message count */
        let mut e = new_nats_export("nats://localhost:4222");
        nats_publish_text(&mut e, "mesh.update", "hello");
        assert_eq!(nats_message_count(&e), 1);
    }

    #[test]
    fn total_bytes_counted() {
        /* payload bytes should be tracked */
        let mut e = new_nats_export("nats://localhost:4222");
        nats_publish_text(&mut e, "s", "world");
        assert_eq!(total_nats_bytes(&e), 5);
    }

    #[test]
    fn find_by_subject_succeeds() {
        /* find returns the correct message */
        let mut e = new_nats_export("nats://localhost");
        nats_publish_text(&mut e, "mesh.pos", "data");
        assert!(find_nats_message(&e, "mesh.pos").is_some());
    }

    #[test]
    fn find_missing_subject_none() {
        /* missing subject returns None */
        let e = new_nats_export("nats://localhost");
        assert!(find_nats_message(&e, "gone").is_none());
    }

    #[test]
    fn request_reply_counted() {
        /* reply_to messages are tracked */
        let mut e = new_nats_export("nats://localhost");
        nats_publish_binary(&mut e, "req", vec![], Some("reply.inbox"));
        nats_publish_text(&mut e, "normal", "x");
        assert_eq!(request_reply_count(&e), 1);
    }

    #[test]
    fn header_added_to_last_message() {
        /* header should be stored on the last message */
        let mut e = new_nats_export("nats://localhost");
        nats_publish_text(&mut e, "s", "v");
        nats_add_header(&mut e, "X-OxiMesh", "1.0");
        assert_eq!(e.messages[0].headers.len(), 1);
    }

    #[test]
    fn json_contains_server() {
        /* JSON includes server URL */
        let e = new_nats_export("nats://nats.example.com");
        assert!(nats_export_to_json(&e).contains("nats.example.com"));
    }

    #[test]
    fn multiple_messages_counted() {
        /* two messages yield count 2 */
        let mut e = new_nats_export("nats://localhost");
        nats_publish_text(&mut e, "a", "x");
        nats_publish_text(&mut e, "b", "y");
        assert_eq!(nats_message_count(&e), 2);
    }
}
