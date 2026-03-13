// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! SMILE (Streaming JSON) binary encoding stub.

/// SMILE magic header bytes: `:)\n`.
pub const SMILE_MAGIC: &[u8; 4] = b":)\n\0";

/// SMILE header version byte.
pub const SMILE_VERSION: u8 = 0x00;

/// SMILE token types.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmileToken {
    StartObject = 0xFA,
    EndObject = 0xFB,
    StartArray = 0xF8,
    EndArray = 0xF9,
    Null = 0x21,
    True = 0x22,
    False = 0x23,
}

/// A SMILE document (stub builder).
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct SmileDocument {
    pub bytes: Vec<u8>,
}

impl SmileDocument {
    /// Create a new SMILE document with magic header.
    #[allow(dead_code)]
    pub fn new() -> Self {
        let mut doc = SmileDocument { bytes: Vec::new() };
        doc.bytes.extend_from_slice(SMILE_MAGIC);
        doc.bytes.push(SMILE_VERSION);
        doc
    }

    /// Emit a fixed token.
    #[allow(dead_code)]
    pub fn push_token(&mut self, token: SmileToken) {
        self.bytes.push(token as u8);
    }

    /// Emit a short ASCII key (length 1–63).
    #[allow(dead_code)]
    pub fn push_key(&mut self, key: &str) {
        let len = key.len().min(63);
        self.bytes.push(0x80 | len as u8);
        self.bytes.extend_from_slice(&key.as_bytes()[..len]);
    }

    /// Emit a short ASCII string value (length 0–63).
    #[allow(dead_code)]
    pub fn push_string_value(&mut self, val: &str) {
        let len = val.len().min(63);
        self.bytes.push(0x40 | len as u8);
        self.bytes.extend_from_slice(&val.as_bytes()[..len]);
    }

    /// Emit a 32-bit integer (variable-length zigzag encoding stub).
    #[allow(dead_code)]
    pub fn push_int32(&mut self, val: i32) {
        let zz = ((val << 1) ^ (val >> 31)) as u32;
        self.bytes.push(0x24);
        self.bytes.extend_from_slice(&zz.to_be_bytes());
    }

    /// Byte length.
    #[allow(dead_code)]
    pub fn byte_len(&self) -> usize {
        self.bytes.len()
    }
}

/// Serialize a SMILE document to bytes.
#[allow(dead_code)]
pub fn export_smile(doc: &SmileDocument) -> &[u8] {
    &doc.bytes
}

/// Check whether bytes start with the SMILE magic.
#[allow(dead_code)]
pub fn is_smile_magic(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && &bytes[..4] == SMILE_MAGIC
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_with_magic() {
        let doc = SmileDocument::new();
        assert!(is_smile_magic(&doc.bytes));
    }

    #[test]
    fn new_has_version_byte() {
        let doc = SmileDocument::new();
        assert_eq!(doc.bytes[4], SMILE_VERSION);
    }

    #[test]
    fn push_token_null() {
        let mut doc = SmileDocument::new();
        doc.push_token(SmileToken::Null);
        assert_eq!(*doc.bytes.last().expect("should succeed"), SmileToken::Null as u8);
    }

    #[test]
    fn push_token_true() {
        let mut doc = SmileDocument::new();
        doc.push_token(SmileToken::True);
        assert_eq!(*doc.bytes.last().expect("should succeed"), 0x22);
    }

    #[test]
    fn push_key_encodes_length() {
        let mut doc = SmileDocument::new();
        doc.push_key("hi");
        assert_eq!(doc.bytes[5], 0x80 | 2);
    }

    #[test]
    fn push_string_value_encodes_length() {
        let mut doc = SmileDocument::new();
        doc.push_string_value("ok");
        assert_eq!(doc.bytes[5], 0x40 | 2);
    }

    #[test]
    fn push_int32_five_bytes() {
        let mut doc = SmileDocument::new();
        let prev = doc.bytes.len();
        doc.push_int32(42);
        assert_eq!(doc.bytes.len() - prev, 5);
    }

    #[test]
    fn byte_len_increases() {
        let mut doc = SmileDocument::new();
        let l0 = doc.byte_len();
        doc.push_token(SmileToken::Null);
        assert!(doc.byte_len() > l0);
    }

    #[test]
    fn is_smile_magic_false_for_empty() {
        assert!(!is_smile_magic(&[]));
    }

    #[test]
    fn start_object_token() {
        let mut doc = SmileDocument::new();
        doc.push_token(SmileToken::StartObject);
        assert_eq!(*doc.bytes.last().expect("should succeed"), 0xFA);
    }
}
