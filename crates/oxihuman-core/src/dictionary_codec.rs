// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.parameter>

//! Dictionary encoding for repeated string values.

#![allow(dead_code)]

use std::collections::HashMap;

/// Dictionary codec that maps strings to integer IDs.
#[allow(dead_code)]
pub struct DictionaryCodec {
    dict: HashMap<String, u32>,
    reverse: Vec<String>,
}

impl DictionaryCodec {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            dict: HashMap::new(),
            reverse: Vec::new(),
        }
    }

    /// Build codec from a slice of strings.
    #[allow(dead_code)]
    pub fn from_data(data: &[&str]) -> Self {
        let mut codec = Self::new();
        for &s in data {
            codec.get_or_insert(s);
        }
        codec
    }

    /// Get ID for string, inserting if new.
    #[allow(dead_code)]
    pub fn get_or_insert(&mut self, s: &str) -> u32 {
        if let Some(&id) = self.dict.get(s) {
            return id;
        }
        let id = self.reverse.len() as u32;
        self.reverse.push(s.to_string());
        self.dict.insert(s.to_string(), id);
        id
    }

    /// Encode slice of strings to IDs.
    #[allow(dead_code)]
    pub fn encode(&mut self, data: &[&str]) -> Vec<u32> {
        data.iter().map(|&s| self.get_or_insert(s)).collect()
    }

    /// Decode IDs back to strings.
    #[allow(dead_code)]
    pub fn decode(&self, ids: &[u32]) -> Vec<String> {
        ids.iter()
            .map(|&id| self.reverse.get(id as usize).cloned().unwrap_or_default())
            .collect()
    }

    /// Look up existing ID without inserting.
    #[allow(dead_code)]
    pub fn lookup(&self, s: &str) -> Option<u32> {
        self.dict.get(s).copied()
    }

    /// Look up string by ID.
    #[allow(dead_code)]
    pub fn lookup_id(&self, id: u32) -> Option<&str> {
        self.reverse.get(id as usize).map(|s| s.as_str())
    }

    #[allow(dead_code)]
    pub fn dict_size(&self) -> usize {
        self.reverse.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.reverse.is_empty()
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.dict.clear();
        self.reverse.clear();
    }

    /// Encode to bytes: 4 bytes per ID (little-endian).
    #[allow(dead_code)]
    pub fn encode_bytes(&mut self, data: &[&str]) -> Vec<u8> {
        let ids = self.encode(data);
        ids.iter().flat_map(|&id| id.to_le_bytes()).collect()
    }

    /// Compression ratio: unique / total.
    #[allow(dead_code)]
    pub fn compression_ratio(total: usize, unique: usize) -> f64 {
        if total == 0 {
            return 1.0;
        }
        unique as f64 / total as f64
    }
}

impl Default for DictionaryCodec {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let mut codec = DictionaryCodec::new();
        let data = ["apple", "banana", "apple", "cherry", "banana"];
        let ids = codec.encode(&data);
        let decoded = codec.decode(&ids);
        let expected: Vec<String> = data.iter().map(|s| s.to_string()).collect();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_repeated_values_same_id() {
        let mut codec = DictionaryCodec::new();
        let id1 = codec.get_or_insert("hello");
        let id2 = codec.get_or_insert("hello");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_unique_ids() {
        let mut codec = DictionaryCodec::new();
        let id_a = codec.get_or_insert("a");
        let id_b = codec.get_or_insert("b");
        assert_ne!(id_a, id_b);
    }

    #[test]
    fn test_lookup() {
        let mut codec = DictionaryCodec::new();
        codec.get_or_insert("x");
        assert!(codec.lookup("x").is_some());
        assert!(codec.lookup("y").is_none());
    }

    #[test]
    fn test_lookup_id() {
        let mut codec = DictionaryCodec::new();
        let id = codec.get_or_insert("hello");
        assert_eq!(codec.lookup_id(id), Some("hello"));
    }

    #[test]
    fn test_dict_size() {
        let mut codec = DictionaryCodec::from_data(&["a", "b", "a", "c"]);
        assert_eq!(codec.dict_size(), 3);
        codec.get_or_insert("d");
        assert_eq!(codec.dict_size(), 4);
    }

    #[test]
    fn test_clear() {
        let mut codec = DictionaryCodec::from_data(&["x", "y"]);
        codec.clear();
        assert!(codec.is_empty());
    }

    #[test]
    fn test_compression_ratio() {
        let r = DictionaryCodec::compression_ratio(100, 10);
        assert!((r - 0.1).abs() < 1e-9);
    }

    #[test]
    fn test_encode_bytes_length() {
        let mut codec = DictionaryCodec::new();
        let data = ["a", "b", "c"];
        let bytes = codec.encode_bytes(&data);
        assert_eq!(bytes.len(), 12);
    }

    #[test]
    fn test_from_data() {
        let codec = DictionaryCodec::from_data(&["foo", "bar", "foo"]);
        assert_eq!(codec.dict_size(), 2);
    }
}
