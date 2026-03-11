// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! JSON Schema generation for parameter/config export validation.
//!
//! Provides a lightweight, dependency-free JSON Schema builder that can
//! serialise schema definitions to JSON strings for downstream validation.

// ── Types ─────────────────────────────────────────────────────────────────────

/// JSON Schema primitive and composite types.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaType {
    /// An object with named properties.
    Object,
    /// An ordered list of values.
    Array,
    /// A UTF-8 string value.
    String,
    /// A numeric value (integer or float).
    Number,
    /// A boolean true/false value.
    Boolean,
    /// The JSON null value.
    Null,
}

/// A single property definition within a JSON Schema object.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SchemaProperty {
    /// Property key name.
    pub name: String,
    /// The JSON type of this property.
    pub schema_type: SchemaType,
    /// Optional human-readable description.
    pub description: Option<String>,
    /// Optional default value as a JSON-encoded string.
    pub default_value: Option<String>,
    /// Whether this property is required.
    pub required: bool,
}

/// A complete JSON Schema definition.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct JsonSchema {
    /// Schema title (maps to `"title"` key).
    pub title: String,
    /// Schema description.
    pub description: String,
    /// Root type of the schema.
    pub root_type: Option<SchemaType>,
    /// Properties (valid when root_type is Object).
    pub properties: Vec<SchemaProperty>,
    /// Names of required properties.
    pub required_fields: Vec<String>,
    /// Optional enum values as JSON-encoded strings.
    pub enum_values: Vec<String>,
}

/// Configuration for schema export behaviour.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SchemaExportConfig {
    /// Include `$schema` declaration in output.
    pub include_schema_declaration: bool,
    /// Indent level for pretty-printing (0 = compact).
    pub indent: usize,
    /// Schema draft version string.
    pub draft_version: String,
}

// ── Constructors ──────────────────────────────────────────────────────────────

/// Returns a default [`SchemaExportConfig`].
#[allow(dead_code)]
pub fn default_schema_config() -> SchemaExportConfig {
    SchemaExportConfig {
        include_schema_declaration: true,
        indent: 2,
        draft_version: "https://json-schema.org/draft/2020-12/schema".to_string(),
    }
}

/// Creates a new empty [`JsonSchema`] with the given title.
#[allow(dead_code)]
pub fn new_json_schema(title: &str) -> JsonSchema {
    JsonSchema {
        title: title.to_string(),
        root_type: Some(SchemaType::Object),
        ..Default::default()
    }
}

// ── Mutation helpers ───────────────────────────────────────────────────────────

/// Adds a property to the schema.
#[allow(dead_code)]
pub fn add_property(schema: &mut JsonSchema, prop: SchemaProperty) {
    if prop.required && !schema.required_fields.contains(&prop.name) {
        schema.required_fields.push(prop.name.clone());
    }
    // Replace if name already exists
    if let Some(pos) = schema.properties.iter().position(|p| p.name == prop.name) {
        schema.properties[pos] = prop;
    } else {
        schema.properties.push(prop);
    }
}

/// Marks a field name as required (adds to required list if not already present).
#[allow(dead_code)]
pub fn add_required_field(schema: &mut JsonSchema, field_name: &str) {
    if !schema.required_fields.contains(&field_name.to_string()) {
        schema.required_fields.push(field_name.to_string());
    }
}

/// Sets the schema title.
#[allow(dead_code)]
pub fn set_schema_title(schema: &mut JsonSchema, title: &str) {
    schema.title = title.to_string();
}

/// Returns the schema title.
#[allow(dead_code)]
pub fn schema_title(schema: &JsonSchema) -> &str {
    &schema.title
}

// ── Counting helpers ───────────────────────────────────────────────────────────

/// Returns the number of properties defined on the schema.
#[allow(dead_code)]
pub fn schema_property_count(schema: &JsonSchema) -> usize {
    schema.properties.len()
}

/// Returns the number of required fields.
#[allow(dead_code)]
pub fn required_field_count(schema: &JsonSchema) -> usize {
    schema.required_fields.len()
}

// ── Serialisation ─────────────────────────────────────────────────────────────

fn type_name(t: &SchemaType) -> &'static str {
    match t {
        SchemaType::Object => "object",
        SchemaType::Array => "array",
        SchemaType::String => "string",
        SchemaType::Number => "number",
        SchemaType::Boolean => "boolean",
        SchemaType::Null => "null",
    }
}

/// Serialises the schema to a JSON string.
///
/// Uses a simple hand-rolled formatter to avoid external dependencies.
#[allow(dead_code)]
pub fn schema_to_json(schema: &JsonSchema, config: &SchemaExportConfig) -> String {
    let indent = config.indent;
    let i1 = " ".repeat(indent);
    let i2 = " ".repeat(indent * 2);
    let i3 = " ".repeat(indent * 3);

    let mut parts: Vec<String> = Vec::new();

    if config.include_schema_declaration {
        parts.push(format!("{}\"$schema\": \"{}\"", i1, config.draft_version));
    }

    if !schema.title.is_empty() {
        parts.push(format!("{}\"title\": \"{}\"", i1, schema.title));
    }
    if !schema.description.is_empty() {
        parts.push(format!("{}\"description\": \"{}\"", i1, schema.description));
    }

    if let Some(rt) = &schema.root_type {
        parts.push(format!("{}\"type\": \"{}\"", i1, type_name(rt)));
    }

    if !schema.enum_values.is_empty() {
        let vals = schema.enum_values.join(", ");
        parts.push(format!("{}\"enum\": [{}]", i1, vals));
    }

    if !schema.properties.is_empty() {
        let mut prop_parts: Vec<String> = Vec::new();
        for prop in &schema.properties {
            let mut lines: Vec<String> = Vec::new();
            lines.push(format!(
                "{}\"type\": \"{}\"",
                i3,
                type_name(&prop.schema_type)
            ));
            if let Some(desc) = &prop.description {
                lines.push(format!("{}\"description\": \"{}\"", i3, desc));
            }
            if let Some(dv) = &prop.default_value {
                lines.push(format!("{}\"default\": {}", i3, dv));
            }
            prop_parts.push(format!(
                "{}\"{}\": {{\n{}\n{}}}",
                i2,
                prop.name,
                lines.join(",\n"),
                i2
            ));
        }
        parts.push(format!(
            "{}\"properties\": {{\n{}\n{}}}",
            i1,
            prop_parts.join(",\n"),
            i1
        ));
    }

    if !schema.required_fields.is_empty() {
        let req: Vec<String> = schema
            .required_fields
            .iter()
            .map(|r| format!("\"{}\"", r))
            .collect();
        parts.push(format!("{}\"required\": [{}]", i1, req.join(", ")));
    }

    format!("{{\n{}\n}}", parts.join(",\n"))
}

// ── Validation ─────────────────────────────────────────────────────────────────

/// Performs basic type-checking of a value string against a [`SchemaType`].
///
/// Accepts JSON-encoded value strings (e.g. `"42"`, `"true"`, `"\"hello\""`)
/// and returns `true` when the value matches the expected type.
#[allow(dead_code)]
pub fn validate_against_schema(schema_type: &SchemaType, value: &str) -> bool {
    let v = value.trim();
    match schema_type {
        SchemaType::Object => v.starts_with('{') && v.ends_with('}'),
        SchemaType::Array => v.starts_with('[') && v.ends_with(']'),
        SchemaType::String => v.starts_with('"') && v.ends_with('"'),
        SchemaType::Number => v.parse::<f64>().is_ok(),
        SchemaType::Boolean => v == "true" || v == "false",
        SchemaType::Null => v == "null",
    }
}

// ── Schema merging ─────────────────────────────────────────────────────────────

/// Merges `other` into `base`, adding properties and required fields.
///
/// Properties with duplicate names from `other` overwrite those in `base`.
#[allow(dead_code)]
pub fn merge_schemas(base: &mut JsonSchema, other: &JsonSchema) {
    for prop in &other.properties {
        add_property(base, prop.clone());
    }
    for req in &other.required_fields {
        add_required_field(base, req);
    }
}

// ── Convenience builders ───────────────────────────────────────────────────────

/// Builds a [`JsonSchema`] from an iterator of `(name, SchemaType)` pairs.
#[allow(dead_code)]
pub fn schema_from_pairs(title: &str, pairs: &[(&str, SchemaType)]) -> JsonSchema {
    let mut schema = new_json_schema(title);
    for (name, t) in pairs {
        add_property(
            &mut schema,
            SchemaProperty {
                name: name.to_string(),
                schema_type: t.clone(),
                description: None,
                default_value: None,
                required: false,
            },
        );
    }
    schema
}

/// Creates a schema that represents a closed enum of string values.
#[allow(dead_code)]
pub fn enum_schema(title: &str, values: &[&str]) -> JsonSchema {
    let mut schema = JsonSchema {
        title: title.to_string(),
        root_type: Some(SchemaType::String),
        enum_values: values.iter().map(|v| format!("\"{}\"", v)).collect(),
        ..Default::default()
    };
    schema.description = format!("Enum: {}", values.join(", "));
    schema
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_prop(name: &str, t: SchemaType) -> SchemaProperty {
        SchemaProperty {
            name: name.to_string(),
            schema_type: t,
            description: None,
            default_value: None,
            required: false,
        }
    }

    #[test]
    fn test_default_schema_config_has_declaration() {
        let cfg = default_schema_config();
        assert!(cfg.include_schema_declaration);
    }

    #[test]
    fn test_default_schema_config_indent() {
        let cfg = default_schema_config();
        assert_eq!(cfg.indent, 2);
    }

    #[test]
    fn test_new_json_schema_title() {
        let s = new_json_schema("MySchema");
        assert_eq!(schema_title(&s), "MySchema");
    }

    #[test]
    fn test_new_json_schema_empty_properties() {
        let s = new_json_schema("Empty");
        assert_eq!(schema_property_count(&s), 0);
    }

    #[test]
    fn test_add_property_increments_count() {
        let mut s = new_json_schema("Test");
        add_property(&mut s, make_prop("x", SchemaType::Number));
        assert_eq!(schema_property_count(&s), 1);
    }

    #[test]
    fn test_add_property_required_auto_registers() {
        let mut s = new_json_schema("Test");
        add_property(
            &mut s,
            SchemaProperty {
                required: true,
                ..make_prop("y", SchemaType::String)
            },
        );
        assert_eq!(required_field_count(&s), 1);
        assert!(s.required_fields.contains(&"y".to_string()));
    }

    #[test]
    fn test_add_property_replaces_duplicate() {
        let mut s = new_json_schema("Test");
        add_property(&mut s, make_prop("z", SchemaType::Boolean));
        add_property(&mut s, make_prop("z", SchemaType::Number));
        assert_eq!(schema_property_count(&s), 1);
        assert_eq!(s.properties[0].schema_type, SchemaType::Number);
    }

    #[test]
    fn test_add_required_field() {
        let mut s = new_json_schema("Test");
        add_required_field(&mut s, "name");
        assert_eq!(required_field_count(&s), 1);
    }

    #[test]
    fn test_add_required_field_no_duplicates() {
        let mut s = new_json_schema("Test");
        add_required_field(&mut s, "id");
        add_required_field(&mut s, "id");
        assert_eq!(required_field_count(&s), 1);
    }

    #[test]
    fn test_set_and_get_schema_title() {
        let mut s = new_json_schema("Old");
        set_schema_title(&mut s, "New");
        assert_eq!(schema_title(&s), "New");
    }

    #[test]
    fn test_schema_to_json_contains_title() {
        let s = new_json_schema("Config");
        let cfg = default_schema_config();
        let json = schema_to_json(&s, &cfg);
        assert!(json.contains("\"title\": \"Config\""));
    }

    #[test]
    fn test_schema_to_json_contains_type() {
        let s = new_json_schema("Config");
        let cfg = default_schema_config();
        let json = schema_to_json(&s, &cfg);
        assert!(json.contains("\"type\": \"object\""));
    }

    #[test]
    fn test_validate_against_schema_number_ok() {
        assert!(validate_against_schema(&SchemaType::Number, "3.14"));
    }

    #[test]
    fn test_validate_against_schema_number_fail() {
        assert!(!validate_against_schema(&SchemaType::Number, "\"hello\""));
    }

    #[test]
    fn test_validate_against_schema_boolean() {
        assert!(validate_against_schema(&SchemaType::Boolean, "true"));
        assert!(validate_against_schema(&SchemaType::Boolean, "false"));
        assert!(!validate_against_schema(&SchemaType::Boolean, "1"));
    }

    #[test]
    fn test_validate_against_schema_null() {
        assert!(validate_against_schema(&SchemaType::Null, "null"));
        assert!(!validate_against_schema(&SchemaType::Null, "false"));
    }

    #[test]
    fn test_merge_schemas() {
        let mut base = new_json_schema("Base");
        add_property(&mut base, make_prop("a", SchemaType::String));

        let mut other = new_json_schema("Other");
        add_property(&mut other, make_prop("b", SchemaType::Number));
        add_required_field(&mut other, "b");

        merge_schemas(&mut base, &other);
        assert_eq!(schema_property_count(&base), 2);
        assert!(base.required_fields.contains(&"b".to_string()));
    }

    #[test]
    fn test_schema_from_pairs() {
        let pairs = [("width", SchemaType::Number), ("label", SchemaType::String)];
        let s = schema_from_pairs("Cfg", &pairs);
        assert_eq!(schema_property_count(&s), 2);
    }

    #[test]
    fn test_enum_schema_values() {
        let s = enum_schema("Mode", &["fast", "slow", "medium"]);
        assert_eq!(s.enum_values.len(), 3);
        assert_eq!(s.root_type, Some(SchemaType::String));
    }
}
