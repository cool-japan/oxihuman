// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Typed config schema with validation.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum SchemaType {
    Bool,
    Int {
        min: i64,
        max: i64,
    },
    Float {
        min: f64,
        max: f64,
    },
    String {
        max_len: usize,
    },
    Enum {
        variants: Vec<std::string::String>,
    },
    Array {
        item_type: Box<SchemaType>,
        max_items: usize,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SchemaField {
    pub name: std::string::String,
    pub schema_type: SchemaType,
    pub required: bool,
    pub default_value: Option<std::string::String>,
    pub description: std::string::String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConfigSchema {
    pub name: std::string::String,
    pub version: std::string::String,
    pub fields: Vec<SchemaField>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConfigValue {
    pub fields: std::collections::HashMap<std::string::String, std::string::String>,
}

/// Create a new empty config schema.
#[allow(dead_code)]
pub fn new_config_schema(name: &str, version: &str) -> ConfigSchema {
    ConfigSchema {
        name: name.to_string(),
        version: version.to_string(),
        fields: Vec::new(),
    }
}

/// Add a field definition to the schema.
#[allow(dead_code)]
pub fn add_field(schema: &mut ConfigSchema, field: SchemaField) {
    schema.fields.push(field);
}

/// Validate a config value against the schema. Returns a list of error messages.
#[allow(dead_code)]
pub fn validate_value(schema: &ConfigSchema, value: &ConfigValue) -> Vec<std::string::String> {
    let mut errors = Vec::new();

    for field in &schema.fields {
        match value.fields.get(&field.name) {
            Some(v) => {
                if let Some(err) = validate_field_value(field, v) {
                    errors.push(err);
                }
            }
            None => {
                if field.required && field.default_value.is_none() {
                    errors.push(format!("required field '{}' is missing", field.name));
                }
            }
        }
    }

    errors
}

/// Validate a single field value string against its schema. Returns an error message or None.
#[allow(dead_code)]
pub fn validate_field_value(field: &SchemaField, value: &str) -> Option<std::string::String> {
    match &field.schema_type {
        SchemaType::Bool => {
            if value != "true" && value != "false" {
                return Some(format!(
                    "field '{}': expected bool (true/false), got '{value}'",
                    field.name
                ));
            }
        }
        SchemaType::Int { min, max } => match value.parse::<i64>() {
            Ok(n) => {
                if n < *min || n > *max {
                    return Some(format!(
                        "field '{}': int {n} out of range [{min}, {max}]",
                        field.name
                    ));
                }
            }
            Err(_) => {
                return Some(format!(
                    "field '{}': cannot parse '{value}' as int",
                    field.name
                ));
            }
        },
        SchemaType::Float { min, max } => match value.parse::<f64>() {
            Ok(f) => {
                if f < *min || f > *max {
                    return Some(format!(
                        "field '{}': float {f} out of range [{min}, {max}]",
                        field.name
                    ));
                }
            }
            Err(_) => {
                return Some(format!(
                    "field '{}': cannot parse '{value}' as float",
                    field.name
                ));
            }
        },
        SchemaType::String { max_len } => {
            // Strip surrounding quotes if JSON-encoded
            let s = strip_json_string(value);
            if s.len() > *max_len {
                return Some(format!(
                    "field '{}': string length {} exceeds max {max_len}",
                    field.name,
                    s.len()
                ));
            }
        }
        SchemaType::Enum { variants } => {
            let s = strip_json_string(value);
            if !variants.iter().any(|v| v == &s) {
                return Some(format!(
                    "field '{}': '{}' is not a valid variant (expected one of: {})",
                    field.name,
                    s,
                    variants.join(", ")
                ));
            }
        }
        SchemaType::Array {
            item_type: _,
            max_items,
        } => {
            // Simple heuristic: count commas + 1 as item estimate
            let trimmed = value.trim();
            if trimmed == "[]" {
                // empty array is fine
            } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let inner = &trimmed[1..trimmed.len() - 1];
                let count = if inner.trim().is_empty() {
                    0
                } else {
                    inner.split(',').count()
                };
                if count > *max_items {
                    return Some(format!(
                        "field '{}': array has {count} items, max is {max_items}",
                        field.name
                    ));
                }
            } else {
                return Some(format!(
                    "field '{}': expected JSON array, got '{value}'",
                    field.name
                ));
            }
        }
    }
    None
}

/// Fill in missing fields with their default values.
#[allow(dead_code)]
pub fn apply_defaults(schema: &ConfigSchema, value: &mut ConfigValue) {
    for field in &schema.fields {
        if !value.fields.contains_key(&field.name) {
            if let Some(default) = &field.default_value {
                value.fields.insert(field.name.clone(), default.clone());
            }
        }
    }
}

/// Get a bool value from a ConfigValue.
#[allow(dead_code)]
pub fn config_value_get_bool(value: &ConfigValue, key: &str) -> Option<bool> {
    value.fields.get(key).and_then(|v| match v.as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    })
}

/// Get an integer value from a ConfigValue.
#[allow(dead_code)]
pub fn config_value_get_int(value: &ConfigValue, key: &str) -> Option<i64> {
    value.fields.get(key)?.parse::<i64>().ok()
}

/// Get a float value from a ConfigValue.
#[allow(dead_code)]
pub fn config_value_get_float(value: &ConfigValue, key: &str) -> Option<f64> {
    value.fields.get(key)?.parse::<f64>().ok()
}

/// Get a string value from a ConfigValue.
#[allow(dead_code)]
pub fn config_value_get_str<'a>(value: &'a ConfigValue, key: &str) -> Option<&'a str> {
    value.fields.get(key).map(|s| s.as_str())
}

/// Serialize the schema to JSON.
#[allow(dead_code)]
pub fn schema_to_json(schema: &ConfigSchema) -> std::string::String {
    let fields_json: Vec<std::string::String> = schema
        .fields
        .iter()
        .map(|f| {
            let type_str = schema_type_to_json(&f.schema_type);
            let required = if f.required { "true" } else { "false" };
            let default = match &f.default_value {
                Some(d) => format!("{d:?}"),
                None => "null".to_string(),
            };
            format!(
                r#"{{"name":{:?},"type":{},"required":{},"default":{},"description":{:?}}}"#,
                f.name, type_str, required, default, f.description
            )
        })
        .collect();

    format!(
        r#"{{"name":{:?},"version":{:?},"fields":[{}]}}"#,
        schema.name,
        schema.version,
        fields_json.join(",")
    )
}

/// Create the default render config schema.
#[allow(dead_code)]
pub fn default_render_schema() -> ConfigSchema {
    let mut schema = new_config_schema("render", "1.0");

    add_field(
        &mut schema,
        SchemaField {
            name: "width".to_string(),
            schema_type: SchemaType::Int { min: 1, max: 16384 },
            required: true,
            default_value: Some("1920".to_string()),
            description: "Output image width in pixels".to_string(),
        },
    );
    add_field(
        &mut schema,
        SchemaField {
            name: "height".to_string(),
            schema_type: SchemaType::Int { min: 1, max: 16384 },
            required: true,
            default_value: Some("1080".to_string()),
            description: "Output image height in pixels".to_string(),
        },
    );
    add_field(
        &mut schema,
        SchemaField {
            name: "quality".to_string(),
            schema_type: SchemaType::Float { min: 0.0, max: 1.0 },
            required: false,
            default_value: Some("0.9".to_string()),
            description: "Render quality 0..1".to_string(),
        },
    );
    add_field(
        &mut schema,
        SchemaField {
            name: "format".to_string(),
            schema_type: SchemaType::Enum {
                variants: vec!["png".to_string(), "jpg".to_string(), "webp".to_string()],
            },
            required: false,
            default_value: Some("png".to_string()),
            description: "Output image format".to_string(),
        },
    );
    add_field(
        &mut schema,
        SchemaField {
            name: "antialiasing".to_string(),
            schema_type: SchemaType::Bool,
            required: false,
            default_value: Some("true".to_string()),
            description: "Enable antialiasing".to_string(),
        },
    );
    add_field(
        &mut schema,
        SchemaField {
            name: "output_path".to_string(),
            schema_type: SchemaType::String { max_len: 512 },
            required: false,
            default_value: Some("\"output.png\"".to_string()),
            description: "Output file path".to_string(),
        },
    );

    schema
}

/// Merge two configs; override_ values take priority over base.
#[allow(dead_code)]
pub fn merge_configs(base: &ConfigValue, override_: &ConfigValue) -> ConfigValue {
    let mut merged = base.fields.clone();
    for (k, v) in &override_.fields {
        merged.insert(k.clone(), v.clone());
    }
    ConfigValue { fields: merged }
}

// Internal helper: strip JSON string quotes
fn strip_json_string(s: &str) -> std::string::String {
    let trimmed = s.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

fn schema_type_to_json(t: &SchemaType) -> std::string::String {
    match t {
        SchemaType::Bool => r#""bool""#.to_string(),
        SchemaType::Int { min, max } => format!(r#"{{"int":{{"min":{min},"max":{max}}}}}"#),
        SchemaType::Float { min, max } => format!(r#"{{"float":{{"min":{min},"max":{max}}}}}"#),
        SchemaType::String { max_len } => format!(r#"{{"string":{{"max_len":{max_len}}}}}"#),
        SchemaType::Enum { variants } => {
            let vs: Vec<std::string::String> = variants.iter().map(|v| format!("{v:?}")).collect();
            format!(r#"{{"enum":{{"variants":[{}]}}}}"#, vs.join(","))
        }
        SchemaType::Array {
            item_type,
            max_items,
        } => {
            format!(
                r#"{{"array":{{"item_type":{},"max_items":{max_items}}}}}"#,
                schema_type_to_json(item_type)
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_simple_schema() -> ConfigSchema {
        let mut schema = new_config_schema("test", "1.0");
        add_field(
            &mut schema,
            SchemaField {
                name: "count".to_string(),
                schema_type: SchemaType::Int { min: 0, max: 100 },
                required: true,
                default_value: None,
                description: "A count".to_string(),
            },
        );
        schema
    }

    fn make_value(pairs: &[(&str, &str)]) -> ConfigValue {
        ConfigValue {
            fields: pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    #[test]
    fn test_new_config_schema() {
        let schema = new_config_schema("test", "2.0");
        assert_eq!(schema.name, "test");
        assert_eq!(schema.version, "2.0");
        assert!(schema.fields.is_empty());
    }

    #[test]
    fn test_add_field() {
        let mut schema = new_config_schema("s", "1");
        add_field(
            &mut schema,
            SchemaField {
                name: "x".to_string(),
                schema_type: SchemaType::Bool,
                required: false,
                default_value: None,
                description: "".to_string(),
            },
        );
        assert_eq!(schema.fields.len(), 1);
    }

    #[test]
    fn test_validate_valid_int() {
        let schema = make_simple_schema();
        let val = make_value(&[("count", "42")]);
        let errs = validate_value(&schema, &val);
        assert!(errs.is_empty(), "expected no errors, got: {errs:?}");
    }

    #[test]
    fn test_validate_int_out_of_range() {
        let schema = make_simple_schema();
        let val = make_value(&[("count", "200")]);
        let errs = validate_value(&schema, &val);
        assert!(!errs.is_empty());
        assert!(errs[0].contains("out of range"));
    }

    #[test]
    fn test_validate_required_missing() {
        let schema = make_simple_schema();
        let val = ConfigValue {
            fields: std::collections::HashMap::new(),
        };
        let errs = validate_value(&schema, &val);
        assert!(!errs.is_empty());
        assert!(errs[0].contains("missing"));
    }

    #[test]
    fn test_validate_bool_field() {
        let mut schema = new_config_schema("s", "1");
        add_field(
            &mut schema,
            SchemaField {
                name: "flag".to_string(),
                schema_type: SchemaType::Bool,
                required: true,
                default_value: None,
                description: "".to_string(),
            },
        );
        let valid = make_value(&[("flag", "true")]);
        assert!(validate_value(&schema, &valid).is_empty());

        let invalid = make_value(&[("flag", "yes")]);
        assert!(!validate_value(&schema, &invalid).is_empty());
    }

    #[test]
    fn test_validate_float_field() {
        let mut schema = new_config_schema("s", "1");
        add_field(
            &mut schema,
            SchemaField {
                name: "ratio".to_string(),
                schema_type: SchemaType::Float { min: 0.0, max: 1.0 },
                required: true,
                default_value: None,
                description: "".to_string(),
            },
        );
        let valid = make_value(&[("ratio", "0.5")]);
        assert!(validate_value(&schema, &valid).is_empty());

        let invalid = make_value(&[("ratio", "2.0")]);
        assert!(!validate_value(&schema, &invalid).is_empty());
    }

    #[test]
    fn test_validate_enum_field() {
        let mut schema = new_config_schema("s", "1");
        add_field(
            &mut schema,
            SchemaField {
                name: "mode".to_string(),
                schema_type: SchemaType::Enum {
                    variants: vec!["a".to_string(), "b".to_string()],
                },
                required: true,
                default_value: None,
                description: "".to_string(),
            },
        );
        let valid = make_value(&[("mode", "a")]);
        assert!(validate_value(&schema, &valid).is_empty());

        let invalid = make_value(&[("mode", "c")]);
        assert!(!validate_value(&schema, &invalid).is_empty());
    }

    #[test]
    fn test_apply_defaults() {
        let mut schema = new_config_schema("s", "1");
        add_field(
            &mut schema,
            SchemaField {
                name: "x".to_string(),
                schema_type: SchemaType::Int { min: 0, max: 100 },
                required: false,
                default_value: Some("42".to_string()),
                description: "".to_string(),
            },
        );
        let mut val = ConfigValue {
            fields: std::collections::HashMap::new(),
        };
        apply_defaults(&schema, &mut val);
        assert_eq!(val.fields.get("x").map(|s| s.as_str()), Some("42"));
    }

    #[test]
    fn test_config_value_get_bool() {
        let val = make_value(&[("flag", "true")]);
        assert_eq!(config_value_get_bool(&val, "flag"), Some(true));
        assert_eq!(config_value_get_bool(&val, "missing"), None);
    }

    #[test]
    fn test_config_value_get_int() {
        let val = make_value(&[("n", "77")]);
        assert_eq!(config_value_get_int(&val, "n"), Some(77));
    }

    #[test]
    fn test_config_value_get_float() {
        let val = make_value(&[("f", "2.71")]);
        let result = config_value_get_float(&val, "f").expect("should succeed");
        assert!((result - 2.71).abs() < 1e-4);
    }

    #[test]
    fn test_config_value_get_str() {
        let val = make_value(&[("key", "hello")]);
        assert_eq!(config_value_get_str(&val, "key"), Some("hello"));
        assert_eq!(config_value_get_str(&val, "missing"), None);
    }

    #[test]
    fn test_schema_to_json_contains_name() {
        let schema = default_render_schema();
        let json = schema_to_json(&schema);
        assert!(json.contains("render"));
        assert!(json.contains("width"));
        assert!(json.contains("quality"));
    }

    #[test]
    fn test_default_render_schema_validates() {
        let schema = default_render_schema();
        let val = make_value(&[("width", "1920"), ("height", "1080")]);
        let errs = validate_value(&schema, &val);
        assert!(errs.is_empty(), "errors: {errs:?}");
    }

    #[test]
    fn test_merge_configs_override_wins() {
        let base = make_value(&[("a", "1"), ("b", "2")]);
        let over = make_value(&[("b", "99"), ("c", "3")]);
        let merged = merge_configs(&base, &over);
        assert_eq!(merged.fields.get("a").map(|s| s.as_str()), Some("1"));
        assert_eq!(merged.fields.get("b").map(|s| s.as_str()), Some("99"));
        assert_eq!(merged.fields.get("c").map(|s| s.as_str()), Some("3"));
    }
}
