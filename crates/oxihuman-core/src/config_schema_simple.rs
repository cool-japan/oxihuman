// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Simple JSON-like schema validation for configuration objects.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleSchemaType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
    Null,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SimpleSchemaField {
    pub name: std::string::String,
    pub field_type: SimpleSchemaType,
    pub required: bool,
    pub default_value: Option<std::string::String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SimpleSchema {
    pub name: std::string::String,
    pub fields: Vec<SimpleSchemaField>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: std::string::String,
    pub message: std::string::String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

#[allow(dead_code)]
pub fn new_schema(name: &str) -> SimpleSchema {
    SimpleSchema {
        name: name.to_string(),
        fields: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_field(schema: &mut SimpleSchema, field: SimpleSchemaField) {
    schema.fields.push(field);
}

#[allow(dead_code)]
pub fn new_schema_field(
    name: &str,
    field_type: SimpleSchemaType,
    required: bool,
) -> SimpleSchemaField {
    SimpleSchemaField {
        name: name.to_string(),
        field_type,
        required,
        default_value: None,
    }
}

#[allow(dead_code)]
pub fn validate_config(
    schema: &SimpleSchema,
    data: &[(std::string::String, std::string::String)],
) -> ValidationResult {
    let mut errors = Vec::new();

    for field in &schema.fields {
        let found = data.iter().find(|(k, _)| k == &field.name);
        match found {
            Some((_, value)) => {
                if let Some(err) = validate_field_type(&field.field_type, &field.name, value) {
                    errors.push(err);
                }
            }
            None => {
                if field.required && field.default_value.is_none() {
                    errors.push(ValidationError {
                        field: field.name.clone(),
                        message: format!("required field '{}' is missing", field.name),
                    });
                }
            }
        }
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
    }
}

fn validate_field_type(
    field_type: &SimpleSchemaType,
    field_name: &str,
    value: &str,
) -> Option<ValidationError> {
    let bad = |msg: &str| {
        Some(ValidationError {
            field: field_name.to_string(),
            message: msg.to_string(),
        })
    };

    match field_type {
        SimpleSchemaType::Integer => {
            if value.parse::<i64>().is_err() {
                return bad(&format!(
                    "field '{}': expected integer, got '{}'",
                    field_name, value
                ));
            }
        }
        SimpleSchemaType::Float => {
            if value.parse::<f64>().is_err() {
                return bad(&format!(
                    "field '{}': expected float, got '{}'",
                    field_name, value
                ));
            }
        }
        SimpleSchemaType::Boolean => {
            if value != "true" && value != "false" {
                return bad(&format!(
                    "field '{}': expected boolean (true/false), got '{}'",
                    field_name, value
                ));
            }
        }
        SimpleSchemaType::Array => {
            let t = value.trim();
            if !t.starts_with('[') || !t.ends_with(']') {
                return bad(&format!(
                    "field '{}': expected array, got '{}'",
                    field_name, value
                ));
            }
        }
        SimpleSchemaType::Object => {
            let t = value.trim();
            if !t.starts_with('{') || !t.ends_with('}') {
                return bad(&format!(
                    "field '{}': expected object, got '{}'",
                    field_name, value
                ));
            }
        }
        SimpleSchemaType::String | SimpleSchemaType::Null => {}
    }
    None
}

#[allow(dead_code)]
pub fn schema_field_count(schema: &SimpleSchema) -> usize {
    schema.fields.len()
}

#[allow(dead_code)]
pub fn required_field_count(schema: &SimpleSchema) -> usize {
    schema.fields.iter().filter(|f| f.required).count()
}

#[allow(dead_code)]
pub fn schema_type_name(t: &SimpleSchemaType) -> &'static str {
    match t {
        SimpleSchemaType::String => "string",
        SimpleSchemaType::Integer => "integer",
        SimpleSchemaType::Float => "float",
        SimpleSchemaType::Boolean => "boolean",
        SimpleSchemaType::Array => "array",
        SimpleSchemaType::Object => "object",
        SimpleSchemaType::Null => "null",
    }
}

#[allow(dead_code)]
pub fn schema_to_json(schema: &SimpleSchema) -> std::string::String {
    let fields_json: Vec<std::string::String> = schema
        .fields
        .iter()
        .map(|f| {
            let type_name = schema_type_name(&f.field_type);
            let required = if f.required { "true" } else { "false" };
            let default = match &f.default_value {
                Some(d) => format!("{:?}", d),
                None => "null".to_string(),
            };
            format!(
                r#"{{"name":{:?},"type":{:?},"required":{},"default":{}}}"#,
                f.name, type_name, required, default
            )
        })
        .collect();
    format!(
        r#"{{"name":{:?},"fields":[{}]}}"#,
        schema.name,
        fields_json.join(",")
    )
}

#[allow(dead_code)]
pub fn validation_result_to_json(r: &ValidationResult) -> std::string::String {
    let errors_json: Vec<std::string::String> = r
        .errors
        .iter()
        .map(|e| format!(r#"{{"field":{:?},"message":{:?}}}"#, e.field, e.message))
        .collect();
    format!(
        r#"{{"valid":{},"errors":[{}]}}"#,
        r.valid,
        errors_json.join(",")
    )
}

#[allow(dead_code)]
pub fn schema_has_field(schema: &SimpleSchema, name: &str) -> bool {
    schema.fields.iter().any(|f| f.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_data(pairs: &[(&str, &str)]) -> Vec<(std::string::String, std::string::String)> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn test_new_schema() {
        let s = new_schema("test");
        assert_eq!(s.name, "test");
        assert!(s.fields.is_empty());
    }

    #[test]
    fn test_add_field_and_count() {
        let mut s = new_schema("s");
        add_field(
            &mut s,
            new_schema_field("x", SimpleSchemaType::Integer, true),
        );
        assert_eq!(schema_field_count(&s), 1);
        assert_eq!(required_field_count(&s), 1);
    }

    #[test]
    fn test_validate_config_valid() {
        let mut s = new_schema("s");
        add_field(
            &mut s,
            new_schema_field("count", SimpleSchemaType::Integer, true),
        );
        let data = make_data(&[("count", "42")]);
        let r = validate_config(&s, &data);
        assert!(r.valid);
        assert!(r.errors.is_empty());
    }

    #[test]
    fn test_validate_config_missing_required() {
        let mut s = new_schema("s");
        add_field(
            &mut s,
            new_schema_field("count", SimpleSchemaType::Integer, true),
        );
        let data = make_data(&[]);
        let r = validate_config(&s, &data);
        assert!(!r.valid);
        assert!(!r.errors.is_empty());
    }

    #[test]
    fn test_validate_config_type_mismatch() {
        let mut s = new_schema("s");
        add_field(
            &mut s,
            new_schema_field("flag", SimpleSchemaType::Boolean, true),
        );
        let data = make_data(&[("flag", "yes")]);
        let r = validate_config(&s, &data);
        assert!(!r.valid);
        assert!(r.errors[0].message.contains("boolean"));
    }

    #[test]
    fn test_schema_type_name() {
        assert_eq!(schema_type_name(&SimpleSchemaType::Integer), "integer");
        assert_eq!(schema_type_name(&SimpleSchemaType::Float), "float");
        assert_eq!(schema_type_name(&SimpleSchemaType::Boolean), "boolean");
        assert_eq!(schema_type_name(&SimpleSchemaType::String), "string");
        assert_eq!(schema_type_name(&SimpleSchemaType::Array), "array");
        assert_eq!(schema_type_name(&SimpleSchemaType::Object), "object");
        assert_eq!(schema_type_name(&SimpleSchemaType::Null), "null");
    }

    #[test]
    fn test_schema_to_json() {
        let mut s = new_schema("cfg");
        add_field(
            &mut s,
            new_schema_field("width", SimpleSchemaType::Integer, true),
        );
        let json = schema_to_json(&s);
        assert!(json.contains("cfg"));
        assert!(json.contains("width"));
        assert!(json.contains("integer"));
    }

    #[test]
    fn test_validation_result_to_json_valid() {
        let r = ValidationResult {
            valid: true,
            errors: vec![],
        };
        let json = validation_result_to_json(&r);
        assert!(json.contains("\"valid\":true"));
    }

    #[test]
    fn test_schema_has_field() {
        let mut s = new_schema("s");
        add_field(
            &mut s,
            new_schema_field("foo", SimpleSchemaType::String, false),
        );
        assert!(schema_has_field(&s, "foo"));
        assert!(!schema_has_field(&s, "bar"));
    }

    #[test]
    fn test_validate_array_type() {
        let mut s = new_schema("s");
        add_field(
            &mut s,
            new_schema_field("items", SimpleSchemaType::Array, true),
        );
        let good = make_data(&[("items", "[1,2,3]")]);
        assert!(validate_config(&s, &good).valid);
        let bad = make_data(&[("items", "notarray")]);
        assert!(!validate_config(&s, &bad).valid);
    }

    #[test]
    fn test_optional_field_missing_ok() {
        let mut s = new_schema("s");
        add_field(
            &mut s,
            new_schema_field("opt", SimpleSchemaType::String, false),
        );
        let data = make_data(&[]);
        let r = validate_config(&s, &data);
        assert!(r.valid);
    }
}
