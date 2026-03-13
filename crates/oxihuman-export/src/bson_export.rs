// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! BSON (Binary JSON) document encoding stub.

/// BSON element type codes.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BsonType {
    Double = 0x01,
    String = 0x02,
    Document = 0x03,
    Array = 0x04,
    Bool = 0x08,
    Null = 0x0A,
    Int32 = 0x10,
    Int64 = 0x12,
}

/// A BSON key-value element.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BsonElement {
    pub key: String,
    pub data: Vec<u8>,
    pub btype: BsonType,
}

/// A BSON document.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct BsonDocument {
    pub elements: Vec<BsonElement>,
}

impl BsonDocument {
    #[allow(dead_code)]
    pub fn new() -> Self {
        BsonDocument::default()
    }

    #[allow(dead_code)]
    pub fn insert_int32(&mut self, key: &str, val: i32) {
        self.elements.push(BsonElement {
            key: key.to_string(),
            data: val.to_le_bytes().to_vec(),
            btype: BsonType::Int32,
        });
    }

    #[allow(dead_code)]
    pub fn insert_int64(&mut self, key: &str, val: i64) {
        self.elements.push(BsonElement {
            key: key.to_string(),
            data: val.to_le_bytes().to_vec(),
            btype: BsonType::Int64,
        });
    }

    #[allow(dead_code)]
    pub fn insert_double(&mut self, key: &str, val: f64) {
        self.elements.push(BsonElement {
            key: key.to_string(),
            data: val.to_bits().to_le_bytes().to_vec(),
            btype: BsonType::Double,
        });
    }

    #[allow(dead_code)]
    pub fn insert_string(&mut self, key: &str, val: &str) {
        let len = (val.len() as i32 + 1).to_le_bytes();
        let mut data = len.to_vec();
        data.extend_from_slice(val.as_bytes());
        data.push(0x00);
        self.elements.push(BsonElement {
            key: key.to_string(),
            data,
            btype: BsonType::String,
        });
    }

    #[allow(dead_code)]
    pub fn insert_bool(&mut self, key: &str, val: bool) {
        self.elements.push(BsonElement {
            key: key.to_string(),
            data: vec![if val { 0x01 } else { 0x00 }],
            btype: BsonType::Bool,
        });
    }
}

/// Serialize a BSON document to bytes.
#[allow(dead_code)]
pub fn serialize_bson(doc: &BsonDocument) -> Vec<u8> {
    let mut body: Vec<u8> = Vec::new();
    for elem in &doc.elements {
        body.push(elem.btype as u8);
        body.extend_from_slice(elem.key.as_bytes());
        body.push(0x00);
        body.extend_from_slice(&elem.data);
    }
    body.push(0x00); // terminator
    let total_len = (body.len() + 4) as i32;
    let mut out = total_len.to_le_bytes().to_vec();
    out.extend(body);
    out
}

/// Element count.
#[allow(dead_code)]
pub fn element_count(doc: &BsonDocument) -> usize {
    doc.elements.len()
}

/// Byte length of serialized BSON.
#[allow(dead_code)]
pub fn bson_byte_len(doc: &BsonDocument) -> usize {
    serialize_bson(doc).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_doc_min_size() {
        let doc = BsonDocument::new();
        let b = serialize_bson(&doc);
        assert_eq!(b.len(), 5);
    }

    #[test]
    fn int32_element_count() {
        let mut doc = BsonDocument::new();
        doc.insert_int32("x", 42);
        assert_eq!(element_count(&doc), 1);
    }

    #[test]
    fn serialize_starts_with_length() {
        let doc = BsonDocument::new();
        let b = serialize_bson(&doc);
        let len = i32::from_le_bytes([b[0], b[1], b[2], b[3]]);
        assert_eq!(len as usize, b.len());
    }

    #[test]
    fn bool_true_value() {
        let mut doc = BsonDocument::new();
        doc.insert_bool("flag", true);
        let elem = &doc.elements[0];
        assert_eq!(elem.data[0], 0x01);
    }

    #[test]
    fn bool_false_value() {
        let mut doc = BsonDocument::new();
        doc.insert_bool("flag", false);
        assert_eq!(doc.elements[0].data[0], 0x00);
    }

    #[test]
    fn string_insert() {
        let mut doc = BsonDocument::new();
        doc.insert_string("name", "hello");
        assert!(!doc.elements.is_empty());
    }

    #[test]
    fn double_insert() {
        let mut doc = BsonDocument::new();
        doc.insert_double("pi", std::f64::consts::PI);
        assert_eq!(doc.elements[0].btype, BsonType::Double);
    }

    #[test]
    fn int64_insert() {
        let mut doc = BsonDocument::new();
        doc.insert_int64("big", 999_999_999_999);
        assert_eq!(doc.elements[0].btype, BsonType::Int64);
    }

    #[test]
    fn multiple_elements() {
        let mut doc = BsonDocument::new();
        doc.insert_int32("a", 1);
        doc.insert_int32("b", 2);
        assert_eq!(element_count(&doc), 2);
    }

    #[test]
    fn byte_len_increases_with_content() {
        let doc0 = BsonDocument::new();
        let mut doc1 = BsonDocument::new();
        doc1.insert_int32("x", 1);
        assert!(bson_byte_len(&doc1) > bson_byte_len(&doc0));
    }
}
