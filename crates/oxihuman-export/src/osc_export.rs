// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! OSC (Open Sound Control) message serialization.

/// OSC argument types.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum OscArg {
    Int(i32),
    Float(f32),
    String(String),
    Blob(Vec<u8>),
}

/// An OSC message.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OscMessage {
    pub address: String,
    pub args: Vec<OscArg>,
}

/// An OSC bundle.
#[allow(dead_code)]
pub struct OscBundle {
    pub time_tag: u64,
    pub messages: Vec<OscMessage>,
}

/// Pad bytes to 4-byte alignment.
fn pad4(n: usize) -> usize {
    (4 - n % 4) % 4
}

/// Encode a string with null terminator, padded to 4 bytes.
fn encode_osc_string(s: &str) -> Vec<u8> {
    let mut out = s.as_bytes().to_vec();
    out.push(0);
    let pad = pad4(out.len());
    out.extend(std::iter::repeat_n(0, pad));
    out
}

/// Serialize an OSC message to bytes.
#[allow(dead_code)]
pub fn serialize_osc_message(msg: &OscMessage) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&encode_osc_string(&msg.address));
    let type_tag = {
        let mut t = String::from(",");
        for arg in &msg.args {
            t.push(match arg {
                OscArg::Int(_) => 'i',
                OscArg::Float(_) => 'f',
                OscArg::String(_) => 's',
                OscArg::Blob(_) => 'b',
            });
        }
        t
    };
    out.extend_from_slice(&encode_osc_string(&type_tag));
    for arg in &msg.args {
        match arg {
            OscArg::Int(i) => out.extend_from_slice(&i.to_be_bytes()),
            OscArg::Float(f) => out.extend_from_slice(&f.to_bits().to_be_bytes()),
            OscArg::String(s) => out.extend_from_slice(&encode_osc_string(s)),
            OscArg::Blob(b) => {
                out.extend_from_slice(&(b.len() as u32).to_be_bytes());
                out.extend_from_slice(b);
                let pad = pad4(b.len());
                out.extend(std::iter::repeat_n(0, pad));
            }
        }
    }
    out
}

/// Serialize an OSC bundle.
#[allow(dead_code)]
pub fn serialize_osc_bundle(bundle: &OscBundle) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&encode_osc_string("#bundle"));
    out.extend_from_slice(&bundle.time_tag.to_be_bytes());
    for msg in &bundle.messages {
        let msg_bytes = serialize_osc_message(msg);
        out.extend_from_slice(&(msg_bytes.len() as u32).to_be_bytes());
        out.extend_from_slice(&msg_bytes);
    }
    out
}

/// Create an OSC message with no arguments.
#[allow(dead_code)]
pub fn new_osc_message(address: &str) -> OscMessage {
    OscMessage {
        address: address.to_string(),
        args: Vec::new(),
    }
}

/// Add an integer argument.
#[allow(dead_code)]
pub fn osc_add_int(msg: &mut OscMessage, val: i32) {
    msg.args.push(OscArg::Int(val));
}

/// Add a float argument.
#[allow(dead_code)]
pub fn osc_add_float(msg: &mut OscMessage, val: f32) {
    msg.args.push(OscArg::Float(val));
}

/// Add a string argument.
#[allow(dead_code)]
pub fn osc_add_string(msg: &mut OscMessage, val: &str) {
    msg.args.push(OscArg::String(val.to_string()));
}

/// Number of arguments in a message.
#[allow(dead_code)]
pub fn osc_arg_count(msg: &OscMessage) -> usize {
    msg.args.len()
}

/// Byte length of the serialized message.
#[allow(dead_code)]
pub fn osc_message_size(msg: &OscMessage) -> usize {
    serialize_osc_message(msg).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_simple_address_aligned() {
        let msg = new_osc_message("/foo");
        let bytes = serialize_osc_message(&msg);
        assert_eq!(bytes.len() % 4, 0);
    }

    #[test]
    fn serialize_int_arg() {
        let mut msg = new_osc_message("/test");
        osc_add_int(&mut msg, 42);
        let bytes = serialize_osc_message(&msg);
        assert!(bytes.len() >= 8);
        assert_eq!(bytes.len() % 4, 0);
    }

    #[test]
    fn serialize_float_arg() {
        let mut msg = new_osc_message("/level");
        osc_add_float(&mut msg, 0.5);
        let bytes = serialize_osc_message(&msg);
        assert!(bytes.len() >= 8);
    }

    #[test]
    fn serialize_string_arg() {
        let mut msg = new_osc_message("/name");
        osc_add_string(&mut msg, "hello");
        let bytes = serialize_osc_message(&msg);
        assert_eq!(bytes.len() % 4, 0);
    }

    #[test]
    fn osc_arg_count_correct() {
        let mut msg = new_osc_message("/multi");
        osc_add_int(&mut msg, 1);
        osc_add_float(&mut msg, 2.0);
        assert_eq!(osc_arg_count(&msg), 2);
    }

    #[test]
    fn osc_message_size_multiple_of_4() {
        let mut msg = new_osc_message("/data");
        osc_add_float(&mut msg, 1.0);
        osc_add_int(&mut msg, 2);
        let size = osc_message_size(&msg);
        assert_eq!(size % 4, 0);
    }

    #[test]
    fn bundle_starts_with_hash_bundle() {
        let bundle = OscBundle {
            time_tag: 1,
            messages: vec![],
        };
        let bytes = serialize_osc_bundle(&bundle);
        assert_eq!(&bytes[0..7], b"#bundle");
    }

    #[test]
    fn bundle_contains_message() {
        let msg = new_osc_message("/ping");
        let bundle = OscBundle {
            time_tag: 0,
            messages: vec![msg],
        };
        let bytes = serialize_osc_bundle(&bundle);
        assert!(bytes.len() > 16);
    }

    #[test]
    fn pad4_values() {
        assert_eq!(pad4(0), 0);
        assert_eq!(pad4(1), 3);
        assert_eq!(pad4(4), 0);
        assert_eq!(pad4(5), 3);
    }

    #[test]
    fn encode_osc_string_padded() {
        let s = encode_osc_string("hi");
        assert_eq!(s.len() % 4, 0);
    }
}
