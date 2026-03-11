// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Simple JSON schema validator stub.

/// Supported JSON schema types.
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaType {
    Null,
    Boolean,
    Integer,
    Number,
    String,
    Array,
    Object,
}

/// A minimal JSON Schema node.
#[derive(Debug, Clone)]
pub struct SchemaNode {
    pub schema_type: Option<SchemaType>,
    pub required: Vec<String>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub description: Option<String>,
}

#[allow(clippy::derivable_impls)]
impl Default for SchemaNode {
    fn default() -> Self {
        SchemaNode {
            schema_type: None,
            required: vec![],
            min_length: None,
            max_length: None,
            minimum: None,
            maximum: None,
            description: None,
        }
    }
}

impl SchemaNode {
    /// Create a new empty schema node.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the type constraint.
    pub fn with_type(mut self, t: SchemaType) -> Self {
        self.schema_type = Some(t);
        self
    }

    /// Add a required property name.
    pub fn require(mut self, field: &str) -> Self {
        self.required.push(field.to_string());
        self
    }

    /// Set the minimum numeric value.
    pub fn with_minimum(mut self, v: f64) -> Self {
        self.minimum = Some(v);
        self
    }

    /// Set the maximum numeric value.
    pub fn with_maximum(mut self, v: f64) -> Self {
        self.maximum = Some(v);
        self
    }
}

/// Validation error.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

/// Validate a numeric value against a schema node.
pub fn validate_number(schema: &SchemaNode, value: f64, path: &str) -> Vec<ValidationError> {
    let mut errs = vec![];
    if let Some(min) = schema.minimum {
        if value < min {
            errs.push(ValidationError {
                path: path.to_string(),
                message: format!("{value} < minimum {min}"),
            });
        }
    }
    if let Some(max) = schema.maximum {
        if value > max {
            errs.push(ValidationError {
                path: path.to_string(),
                message: format!("{value} > maximum {max}"),
            });
        }
    }
    errs
}

/// Validate a string value against a schema node.
pub fn validate_string(schema: &SchemaNode, value: &str, path: &str) -> Vec<ValidationError> {
    let mut errs = vec![];
    let len = value.len();
    if let Some(min) = schema.min_length {
        if len < min {
            errs.push(ValidationError {
                path: path.to_string(),
                message: format!("string length {len} < minLength {min}"),
            });
        }
    }
    if let Some(max) = schema.max_length {
        if len > max {
            errs.push(ValidationError {
                path: path.to_string(),
                message: format!("string length {len} > maxLength {max}"),
            });
        }
    }
    errs
}

/// Return `true` if a field name is in the required list.
pub fn is_required(schema: &SchemaNode, field: &str) -> bool {
    schema.required.iter().any(|r| r == field)
}

/// Count required fields.
pub fn required_count(schema: &SchemaNode) -> usize {
    schema.required.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_default() {
        /* default schema has no type */
        let s = SchemaNode::new();
        assert!(s.schema_type.is_none());
    }

    #[test]
    fn test_with_type() {
        /* with_type sets schema_type */
        let s = SchemaNode::new().with_type(SchemaType::String);
        assert_eq!(s.schema_type, Some(SchemaType::String));
    }

    #[test]
    fn test_require_adds_field() {
        /* require appends to required list */
        let s = SchemaNode::new().require("name").require("age");
        assert_eq!(s.required.len(), 2);
    }

    #[test]
    fn test_is_required_true() {
        /* listed field is required */
        let s = SchemaNode::new().require("email");
        assert!(is_required(&s, "email"));
    }

    #[test]
    fn test_is_required_false() {
        /* unlisted field is not required */
        let s = SchemaNode::new().require("email");
        assert!(!is_required(&s, "phone"));
    }

    #[test]
    fn test_validate_number_ok() {
        /* value in range produces no errors */
        let s = SchemaNode::new().with_minimum(0.0).with_maximum(100.0);
        assert!(validate_number(&s, 50.0, "/x").is_empty());
    }

    #[test]
    fn test_validate_number_below_min() {
        /* value below minimum produces error */
        let s = SchemaNode::new().with_minimum(10.0);
        assert!(!validate_number(&s, 5.0, "/x").is_empty());
    }

    #[test]
    fn test_validate_string_ok() {
        /* string in length range is valid */
        let mut s = SchemaNode::new();
        s.min_length = Some(2);
        s.max_length = Some(10);
        assert!(validate_string(&s, "hello", "/s").is_empty());
    }

    #[test]
    fn test_validate_string_too_short() {
        /* string shorter than minLength fails */
        let mut s = SchemaNode::new();
        s.min_length = Some(5);
        assert!(!validate_string(&s, "hi", "/s").is_empty());
    }

    #[test]
    fn test_required_count() {
        /* required_count returns correct count */
        let s = SchemaNode::new().require("a").require("b").require("c");
        assert_eq!(required_count(&s), 3);
    }
}
