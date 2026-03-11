// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! OSC (Open Sound Control) bundle stub export.

/// An OSC argument value.
#[derive(Debug, Clone)]
pub enum OscArg {
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
}

impl OscArg {
    pub fn type_tag(&self) -> char {
        match self {
            OscArg::Int(_) => 'i',
            OscArg::Float(_) => 'f',
            OscArg::String(_) => 's',
            OscArg::Bool(true) => 'T',
            OscArg::Bool(false) => 'F',
        }
    }
}

/// A single OSC message: address pattern + arguments.
#[derive(Debug, Clone)]
pub struct OscMessage {
    pub address: String,
    pub args: Vec<OscArg>,
}

impl OscMessage {
    pub fn new(address: impl Into<String>, args: Vec<OscArg>) -> Self {
        Self {
            address: address.into(),
            args,
        }
    }
}

/// An OSC bundle containing multiple messages with a timestamp.
#[derive(Debug, Clone, Default)]
pub struct OscBundle {
    pub timestamp_secs: f64,
    pub messages: Vec<OscMessage>,
}

impl OscBundle {
    pub fn new(timestamp_secs: f64) -> Self {
        Self {
            timestamp_secs,
            messages: Vec::new(),
        }
    }

    pub fn add_message(&mut self, msg: OscMessage) {
        self.messages.push(msg);
    }
}

/// Pad a byte buffer to the next 4-byte boundary with zeros.
pub fn pad_to_4(buf: &mut Vec<u8>) {
    while !buf.len().is_multiple_of(4) {
        buf.push(0);
    }
}

/// Encode an OSC string (null-terminated, padded to 4 bytes).
pub fn encode_osc_string(s: &str) -> Vec<u8> {
    let mut buf: Vec<u8> = s.as_bytes().to_vec();
    buf.push(0); /* null terminator */
    pad_to_4(&mut buf);
    buf
}

/// Encode a single OSC message to bytes.
pub fn encode_osc_message(msg: &OscMessage) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&encode_osc_string(&msg.address));
    /* Build type tag string */
    let tags: String = std::iter::once(',')
        .chain(msg.args.iter().map(|a| a.type_tag()))
        .collect();
    buf.extend_from_slice(&encode_osc_string(&tags));
    /* Encode arguments */
    for arg in &msg.args {
        match arg {
            OscArg::Int(v) => buf.extend_from_slice(&v.to_be_bytes()),
            OscArg::Float(v) => buf.extend_from_slice(&v.to_be_bytes()),
            OscArg::String(s) => buf.extend_from_slice(&encode_osc_string(s)),
            OscArg::Bool(_) => { /* T/F have no data bytes */ }
        }
    }
    buf
}

/// Encode an OSC bundle to bytes.
pub fn encode_osc_bundle(bundle: &OscBundle) -> Vec<u8> {
    /* Bundle starts with "#bundle\0" + 8-byte NTP timestamp stub */
    let mut buf = encode_osc_string("#bundle");
    /* NTP timestamp: seconds since 1900, fractional seconds */
    let secs_since_1900 = (bundle.timestamp_secs + 2_208_988_800.0) as u32;
    buf.extend_from_slice(&secs_since_1900.to_be_bytes());
    buf.extend_from_slice(&0u32.to_be_bytes()); /* fractional seconds stub */
    /* Each message is prefixed with its 4-byte length */
    for msg in &bundle.messages {
        let encoded = encode_osc_message(msg);
        buf.extend_from_slice(&(encoded.len() as u32).to_be_bytes());
        buf.extend_from_slice(&encoded);
    }
    buf
}

/// Validate that a byte buffer starts with the OSC bundle magic.
pub fn is_osc_bundle(data: &[u8]) -> bool {
    data.len() >= 8 && &data[0..7] == b"#bundle"
}

/// Count OSC messages in a bundle.
pub fn count_osc_messages(bundle: &OscBundle) -> usize {
    bundle.messages.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osc_arg_type_tags() {
        assert_eq!(OscArg::Int(0).type_tag(), 'i' /* int tag */);
        assert_eq!(OscArg::Float(0.0).type_tag(), 'f' /* float tag */);
        assert_eq!(
            OscArg::String("x".into()).type_tag(),
            's' /* string tag */
        );
        assert_eq!(OscArg::Bool(true).type_tag(), 'T' /* true tag */);
        assert_eq!(OscArg::Bool(false).type_tag(), 'F' /* false tag */);
    }

    #[test]
    fn test_pad_to_4() {
        let mut buf = vec![1u8, 2, 3];
        pad_to_4(&mut buf);
        assert_eq!(buf.len() % 4, 0 /* padded to multiple of 4 */);
    }

    #[test]
    fn test_encode_osc_string_multiple_of_4() {
        let s = encode_osc_string("/test");
        assert_eq!(s.len() % 4, 0 /* OSC strings are 4-byte aligned */);
    }

    #[test]
    fn test_encode_osc_string_null_terminated() {
        let s = encode_osc_string("hi");
        assert!(s.contains(&0u8) /* contains null terminator */);
    }

    #[test]
    fn test_encode_osc_message_non_empty() {
        let msg = OscMessage::new("/test", vec![OscArg::Int(42)]);
        let bytes = encode_osc_message(&msg);
        assert!(!bytes.is_empty() /* non-empty message */);
    }

    #[test]
    fn test_encode_osc_bundle_magic() {
        let bundle = OscBundle::new(0.0);
        let bytes = encode_osc_bundle(&bundle);
        assert!(is_osc_bundle(&bytes) /* bundle magic present */);
    }

    #[test]
    fn test_count_messages() {
        let mut bundle = OscBundle::new(0.0);
        bundle.add_message(OscMessage::new("/a", vec![]));
        bundle.add_message(OscMessage::new("/b", vec![]));
        assert_eq!(count_osc_messages(&bundle), 2 /* two messages */);
    }

    #[test]
    fn test_bundle_with_float_arg() {
        let mut bundle = OscBundle::new(1.0);
        bundle.add_message(OscMessage::new("/freq", vec![OscArg::Float(440.0)]));
        let bytes = encode_osc_bundle(&bundle);
        assert!(bytes.len() > 16 /* has message data */);
    }

    #[test]
    fn test_is_osc_bundle_rejects_random() {
        assert!(!is_osc_bundle(b"randomdata") /* not a bundle */);
    }
}
