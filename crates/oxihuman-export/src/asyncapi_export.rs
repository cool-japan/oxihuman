// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! AsyncAPI spec export stub.

use std::collections::BTreeMap;

/// AsyncAPI channel binding protocol.
#[derive(Debug, Clone, PartialEq)]
pub enum AsyncProtocol {
    Amqp,
    Mqtt,
    WebSocket,
    Kafka,
    Http,
}

impl AsyncProtocol {
    /// Protocol name string.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Amqp => "amqp",
            Self::Mqtt => "mqtt",
            Self::WebSocket => "ws",
            Self::Kafka => "kafka",
            Self::Http => "http",
        }
    }
}

/// An AsyncAPI message.
#[derive(Debug, Clone)]
pub struct AsyncMessage {
    pub name: String,
    pub summary: String,
    pub payload_schema: String,
}

/// An AsyncAPI channel (publish/subscribe topic).
#[derive(Debug, Clone)]
pub struct AsyncChannel {
    pub name: String,
    pub description: Option<String>,
    pub subscribe: Option<AsyncMessage>,
    pub publish: Option<AsyncMessage>,
}

impl AsyncChannel {
    /// Create a new channel.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            subscribe: None,
            publish: None,
        }
    }
}

/// AsyncAPI document.
#[derive(Debug, Clone, Default)]
pub struct AsyncApiSpec {
    pub title: String,
    pub version: String,
    pub protocol: Option<AsyncProtocol>,
    pub channels: BTreeMap<String, AsyncChannel>,
    pub servers: Vec<String>,
}

impl AsyncApiSpec {
    /// Add a channel.
    pub fn add_channel(&mut self, channel: AsyncChannel) {
        self.channels.insert(channel.name.clone(), channel);
    }

    /// Number of channels.
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    /// Find channel by name.
    pub fn find_channel(&self, name: &str) -> Option<&AsyncChannel> {
        self.channels.get(name)
    }
}

/// Render AsyncAPI spec as minimal JSON.
pub fn render_asyncapi_json(spec: &AsyncApiSpec) -> String {
    let protocol_str = spec.protocol.as_ref().map(|p| p.name()).unwrap_or("ws");
    let channels: Vec<String> = spec
        .channels
        .iter()
        .map(|(name, ch)| {
            format!(
                r#""{name}":{{"description":"{}"}}"#,
                ch.description.as_deref().unwrap_or("")
            )
        })
        .collect();
    format!(
        r#"{{"asyncapi":"2.6.0","info":{{"title":"{}","version":"{}"}},"defaultContentType":"application/json","channels":{{{}}},"protocol":"{}"}}"#,
        spec.title,
        spec.version,
        channels.join(","),
        protocol_str
    )
}

/// Validate spec (title, version, at least one channel).
pub fn validate_spec(spec: &AsyncApiSpec) -> bool {
    !spec.title.is_empty() && !spec.version.is_empty() && !spec.channels.is_empty()
}

/// Count channels that have a subscribe operation.
pub fn subscribe_channel_count(spec: &AsyncApiSpec) -> usize {
    spec.channels
        .values()
        .filter(|c| c.subscribe.is_some())
        .count()
}

/// Count channels that have a publish operation.
pub fn publish_channel_count(spec: &AsyncApiSpec) -> usize {
    spec.channels
        .values()
        .filter(|c| c.publish.is_some())
        .count()
}

/// Add a server URL to the spec.
pub fn add_server(spec: &mut AsyncApiSpec, url: impl Into<String>) {
    spec.servers.push(url.into());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_spec() -> AsyncApiSpec {
        let mut spec = AsyncApiSpec {
            title: "Test Async".into(),
            version: "1.0.0".into(),
            protocol: Some(AsyncProtocol::WebSocket),
            ..Default::default()
        };
        let mut ch = AsyncChannel::new("user/events");
        ch.description = Some("User event stream".into());
        ch.subscribe = Some(AsyncMessage {
            name: "UserCreated".into(),
            summary: "A user was created".into(),
            payload_schema: "{}".into(),
        });
        spec.add_channel(ch);
        spec
    }

    #[test]
    fn channel_count() {
        assert_eq!(sample_spec().channel_count(), 1);
    }

    #[test]
    fn find_channel_found() {
        assert!(sample_spec().find_channel("user/events").is_some());
    }

    #[test]
    fn find_channel_missing() {
        assert!(sample_spec().find_channel("nope").is_none());
    }

    #[test]
    fn render_contains_asyncapi_version() {
        assert!(render_asyncapi_json(&sample_spec()).contains("2.6.0"));
    }

    #[test]
    fn render_contains_title() {
        assert!(render_asyncapi_json(&sample_spec()).contains("Test Async"));
    }

    #[test]
    fn validate_ok() {
        assert!(validate_spec(&sample_spec()));
    }

    #[test]
    fn validate_no_channels() {
        let spec = AsyncApiSpec {
            title: "T".into(),
            version: "1.0".into(),
            ..Default::default()
        };
        assert!(!validate_spec(&spec));
    }

    #[test]
    fn subscribe_count() {
        /* one channel has subscribe */
        assert_eq!(subscribe_channel_count(&sample_spec()), 1);
    }

    #[test]
    fn publish_count_zero() {
        /* no publish operations */
        assert_eq!(publish_channel_count(&sample_spec()), 0);
    }

    #[test]
    fn add_server_increments() {
        let mut spec = sample_spec();
        add_server(&mut spec, "ws://localhost:4000");
        assert_eq!(spec.servers.len(), 1);
    }
}
