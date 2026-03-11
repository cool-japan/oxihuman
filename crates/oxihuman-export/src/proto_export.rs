// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Protocol Buffers stub export — generate .proto schema stubs.

/// A Protocol Buffers field descriptor.
#[derive(Debug, Clone)]
pub struct ProtoFieldDef {
    pub name: String,
    pub proto_type: String,
    pub field_number: u32,
    pub repeated: bool,
}

/// A Protocol Buffers message descriptor.
#[derive(Debug, Clone, Default)]
pub struct ProtoMessageDef {
    pub name: String,
    pub fields: Vec<ProtoFieldDef>,
}

impl ProtoMessageDef {
    pub fn new(name: &str) -> Self {
        Self { name: name.into(), fields: vec![] }
    }
}

/// A .proto schema file stub.
#[derive(Debug, Clone, Default)]
pub struct ProtoSchemaDef {
    pub syntax: String,
    pub package: String,
    pub messages: Vec<ProtoMessageDef>,
}

/// Create a new proto schema.
pub fn new_proto_schema(package: &str) -> ProtoSchemaDef {
    ProtoSchemaDef { syntax: "proto3".into(), package: package.into(), messages: vec![] }
}

/// Add a message to the schema.
pub fn add_proto_message(schema: &mut ProtoSchemaDef, msg: ProtoMessageDef) {
    schema.messages.push(msg);
}

/// Add a field to a message.
pub fn add_field(msg: &mut ProtoMessageDef, name: &str, proto_type: &str, field_number: u32, repeated: bool) {
    msg.fields.push(ProtoFieldDef {
        name: name.into(),
        proto_type: proto_type.into(),
        field_number,
        repeated,
    });
}

/// Serialize the schema to a .proto string.
pub fn schema_to_proto_string(schema: &ProtoSchemaDef) -> String {
    let mut out = format!("syntax = \"{}\";\n\npackage {};\n\n", schema.syntax, schema.package);
    for msg in &schema.messages {
        out.push_str(&format!("message {} {{\n", msg.name));
        for f in &msg.fields {
            let rep = if f.repeated { "repeated " } else { "" };
            out.push_str(&format!("  {}{} {} = {};\n", rep, f.proto_type, f.name, f.field_number));
        }
        out.push_str("}\n\n");
    }
    out
}

/// Export mesh as proto schema with Vertex and Mesh messages.
pub fn export_mesh_proto_schema(positions: &[[f32; 3]]) -> ProtoSchemaDef {
    let mut schema = new_proto_schema("oxihuman");
    let mut vertex_msg = ProtoMessageDef::new("Vertex");
    add_field(&mut vertex_msg, "x", "float", 1, false);
    add_field(&mut vertex_msg, "y", "float", 2, false);
    add_field(&mut vertex_msg, "z", "float", 3, false);
    add_proto_message(&mut schema, vertex_msg);
    let mut mesh_msg = ProtoMessageDef::new("Mesh");
    add_field(&mut mesh_msg, "vertices", "Vertex", 1, true);
    add_field(&mut mesh_msg, "indices", "uint32", 2, true);
    add_field(&mut mesh_msg, "vertex_count", "uint32", 3, false);
    if let Some(last_field) = mesh_msg.fields.last_mut() {
        let _ = last_field;
    }
    /* store vertex_count hint */
    let _ = positions.len();
    add_proto_message(&mut schema, mesh_msg);
    schema
}

/// Number of messages in the schema.
pub fn proto_message_count(schema: &ProtoSchemaDef) -> usize {
    schema.messages.len()
}

/// Number of fields in a message.
pub fn proto_field_count(msg: &ProtoMessageDef) -> usize {
    msg.fields.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_proto_schema() {
        /* new schema has correct syntax and package */
        let s = new_proto_schema("test.pkg");
        assert_eq!(s.syntax, "proto3");
        assert_eq!(s.package, "test.pkg");
    }

    #[test]
    fn test_add_message() {
        /* adding a message increases count */
        let mut s = new_proto_schema("pkg");
        add_proto_message(&mut s, ProtoMessageDef::new("Foo"));
        assert_eq!(proto_message_count(&s), 1);
    }

    #[test]
    fn test_add_field_increases_count() {
        /* adding field increases field count */
        let mut msg = ProtoMessageDef::new("Bar");
        add_field(&mut msg, "id", "uint32", 1, false);
        assert_eq!(proto_field_count(&msg), 1);
    }

    #[test]
    fn test_schema_to_proto_contains_syntax() {
        /* output contains syntax declaration */
        let s = new_proto_schema("p");
        let out = schema_to_proto_string(&s);
        assert!(out.contains("syntax = \"proto3\""));
    }

    #[test]
    fn test_schema_to_proto_contains_package() {
        /* output contains package name */
        let s = new_proto_schema("mypkg");
        assert!(schema_to_proto_string(&s).contains("mypkg"));
    }

    #[test]
    fn test_schema_to_proto_contains_message() {
        /* message block appears in output */
        let mut s = new_proto_schema("p");
        add_proto_message(&mut s, ProtoMessageDef::new("MyMsg"));
        assert!(schema_to_proto_string(&s).contains("message MyMsg"));
    }

    #[test]
    fn test_schema_to_proto_repeated_field() {
        /* repeated keyword appears for repeated fields */
        let mut s = new_proto_schema("p");
        let mut msg = ProtoMessageDef::new("M");
        add_field(&mut msg, "items", "float", 1, true);
        add_proto_message(&mut s, msg);
        assert!(schema_to_proto_string(&s).contains("repeated"));
    }

    #[test]
    fn test_export_mesh_proto_schema() {
        /* mesh schema has two messages */
        let p = vec![[0.0f32,0.0,0.0],[1.0,0.0,0.0]];
        let s = export_mesh_proto_schema(&p);
        assert_eq!(proto_message_count(&s), 2);
    }

    #[test]
    fn test_export_mesh_proto_vertex_fields() {
        /* Vertex message has 3 fields */
        let s = export_mesh_proto_schema(&[]);
        assert_eq!(proto_field_count(&s.messages[0]), 3);
    }

    #[test]
    fn test_proto_field_number() {
        /* field numbers are stored correctly */
        let mut msg = ProtoMessageDef::new("T");
        add_field(&mut msg, "val", "int32", 42, false);
        assert_eq!(msg.fields[0].field_number, 42);
    }
}
