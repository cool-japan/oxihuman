// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export data as Protocol Buffers text format.

#![allow(dead_code)]

/// A proto text field.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProtoField {
    pub name: String,
    pub value: String,
}

/// A proto text message.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ProtoMessage {
    pub name: String,
    pub fields: Vec<ProtoField>,
    pub nested: Vec<ProtoMessage>,
}

/// Create a new proto message.
#[allow(dead_code)]
pub fn new_proto_message(name: &str) -> ProtoMessage {
    ProtoMessage {
        name: name.to_string(),
        fields: Vec::new(),
        nested: Vec::new(),
    }
}

/// Add a string field.
#[allow(dead_code)]
pub fn add_proto_string(msg: &mut ProtoMessage, name: &str, value: &str) {
    msg.fields.push(ProtoField {
        name: name.to_string(),
        value: format!("\"{}\"", value),
    });
}

/// Add an integer field.
#[allow(dead_code)]
pub fn add_proto_int(msg: &mut ProtoMessage, name: &str, value: i64) {
    msg.fields.push(ProtoField {
        name: name.to_string(),
        value: value.to_string(),
    });
}

/// Add a float field.
#[allow(dead_code)]
pub fn add_proto_float(msg: &mut ProtoMessage, name: &str, value: f64) {
    msg.fields.push(ProtoField {
        name: name.to_string(),
        value: format!("{:.6}", value),
    });
}

/// Add a boolean field.
#[allow(dead_code)]
pub fn add_proto_bool(msg: &mut ProtoMessage, name: &str, value: bool) {
    msg.fields.push(ProtoField {
        name: name.to_string(),
        value: if value {
            "true".to_string()
        } else {
            "false".to_string()
        },
    });
}

/// Add a nested message.
#[allow(dead_code)]
pub fn add_proto_nested(msg: &mut ProtoMessage, nested: ProtoMessage) {
    msg.nested.push(nested);
}

/// Return the field count.
#[allow(dead_code)]
pub fn proto_field_count(msg: &ProtoMessage) -> usize {
    msg.fields.len()
}

/// Find a field by name.
#[allow(dead_code)]
pub fn find_proto_field<'a>(msg: &'a ProtoMessage, name: &str) -> Option<&'a str> {
    msg.fields
        .iter()
        .find(|f| f.name == name)
        .map(|f| f.value.as_str())
}

/// Serialise as proto text format.
#[allow(dead_code)]
pub fn to_proto_text(msg: &ProtoMessage, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    let mut out = if msg.name.is_empty() {
        String::new()
    } else {
        format!("{}{} {{\n", pad, msg.name)
    };
    for f in &msg.fields {
        out.push_str(&format!("{}  {}: {}\n", pad, f.name, f.value));
    }
    for n in &msg.nested {
        out.push_str(&to_proto_text(n, indent + 1));
    }
    if !msg.name.is_empty() {
        out.push_str(&format!("{}}}\n", pad));
    }
    out
}

/// Export mesh stats as proto text.
#[allow(dead_code)]
pub fn export_mesh_stats_proto(vertex_count: usize, index_count: usize, name: &str) -> String {
    let mut msg = new_proto_message("Mesh");
    add_proto_string(&mut msg, "name", name);
    add_proto_int(&mut msg, "vertex_count", vertex_count as i64);
    add_proto_int(&mut msg, "index_count", index_count as i64);
    to_proto_text(&msg, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_proto_message_empty() {
        let msg = new_proto_message("Foo");
        assert_eq!(proto_field_count(&msg), 0);
    }

    #[test]
    fn test_add_string() {
        let mut msg = new_proto_message("M");
        add_proto_string(&mut msg, "name", "test");
        assert_eq!(proto_field_count(&msg), 1);
    }

    #[test]
    fn test_add_int() {
        let mut msg = new_proto_message("M");
        add_proto_int(&mut msg, "count", 42);
        assert_eq!(find_proto_field(&msg, "count"), Some("42"));
    }

    #[test]
    fn test_add_float() {
        let mut msg = new_proto_message("M");
        add_proto_float(&mut msg, "scale", 1.0);
        assert!(find_proto_field(&msg, "scale").is_some());
    }

    #[test]
    fn test_add_bool() {
        let mut msg = new_proto_message("M");
        add_proto_bool(&mut msg, "enabled", true);
        assert_eq!(find_proto_field(&msg, "enabled"), Some("true"));
    }

    #[test]
    fn test_to_proto_text_contains_name() {
        let msg = new_proto_message("Mesh");
        let s = to_proto_text(&msg, 0);
        assert!(s.contains("Mesh"));
    }

    #[test]
    fn test_to_proto_text_contains_field() {
        let mut msg = new_proto_message("M");
        add_proto_int(&mut msg, "vertices", 100);
        let s = to_proto_text(&msg, 0);
        assert!(s.contains("vertices: 100"));
    }

    #[test]
    fn test_nested_message() {
        let mut outer = new_proto_message("Outer");
        let mut inner = new_proto_message("inner");
        add_proto_string(&mut inner, "k", "v");
        add_proto_nested(&mut outer, inner);
        let s = to_proto_text(&outer, 0);
        assert!(s.contains("inner"));
    }

    #[test]
    fn test_export_mesh_stats_proto() {
        let s = export_mesh_stats_proto(100, 300, "body");
        assert!(s.contains("vertex_count"));
        assert!(s.contains("body"));
    }

    #[test]
    fn test_find_missing_field() {
        let msg = new_proto_message("M");
        assert!(find_proto_field(&msg, "ghost").is_none());
    }
}
