#![allow(dead_code)]

/// Tag discriminant for tagged values.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    Int,
    Float,
    Text,
}

/// A tagged union that can hold an int, float, or string.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TaggedValue {
    Int(i64),
    Float(f64),
    Text(String),
}

/// Creates a tagged int.
#[allow(dead_code)]
pub fn new_tagged_int(value: i64) -> TaggedValue {
    TaggedValue::Int(value)
}

/// Creates a tagged float.
#[allow(dead_code)]
pub fn new_tagged_float(value: f64) -> TaggedValue {
    TaggedValue::Float(value)
}

/// Creates a tagged string.
#[allow(dead_code)]
pub fn new_tagged_string(value: &str) -> TaggedValue {
    TaggedValue::Text(value.to_string())
}

/// Returns the tag of the value.
#[allow(dead_code)]
pub fn tag_of(value: &TaggedValue) -> Tag {
    match value {
        TaggedValue::Int(_) => Tag::Int,
        TaggedValue::Float(_) => Tag::Float,
        TaggedValue::Text(_) => Tag::Text,
    }
}

/// Extracts as int.
#[allow(dead_code)]
pub fn as_int(value: &TaggedValue) -> Option<i64> {
    match value {
        TaggedValue::Int(v) => Some(*v),
        _ => None,
    }
}

/// Extracts as float.
#[allow(dead_code)]
pub fn as_float(value: &TaggedValue) -> Option<f64> {
    match value {
        TaggedValue::Float(v) => Some(*v),
        _ => None,
    }
}

/// Extracts as string.
#[allow(dead_code)]
pub fn as_string(value: &TaggedValue) -> Option<&str> {
    match value {
        TaggedValue::Text(s) => Some(s),
        _ => None,
    }
}

/// Converts to JSON representation.
#[allow(dead_code)]
pub fn tagged_to_json(value: &TaggedValue) -> String {
    match value {
        TaggedValue::Int(v) => format!("{{\"tag\":\"int\",\"value\":{v}}}"),
        TaggedValue::Float(v) => format!("{{\"tag\":\"float\",\"value\":{v}}}"),
        TaggedValue::Text(s) => format!("{{\"tag\":\"string\",\"value\":\"{s}\"}}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tagged_int() {
        let v = new_tagged_int(42);
        assert_eq!(as_int(&v), Some(42));
    }

    #[test]
    fn test_new_tagged_float() {
        let v = new_tagged_float(2.78);
        assert_eq!(as_float(&v), Some(2.78));
    }

    #[test]
    fn test_new_tagged_string() {
        let v = new_tagged_string("hello");
        assert_eq!(as_string(&v), Some("hello"));
    }

    #[test]
    fn test_tag_of() {
        assert_eq!(tag_of(&new_tagged_int(0)), Tag::Int);
        assert_eq!(tag_of(&new_tagged_float(0.0)), Tag::Float);
        assert_eq!(tag_of(&new_tagged_string("")), Tag::Text);
    }

    #[test]
    fn test_as_int_wrong_type() {
        let v = new_tagged_float(1.0);
        assert_eq!(as_int(&v), None);
    }

    #[test]
    fn test_as_float_wrong_type() {
        let v = new_tagged_int(1);
        assert_eq!(as_float(&v), None);
    }

    #[test]
    fn test_as_string_wrong_type() {
        let v = new_tagged_int(1);
        assert_eq!(as_string(&v), None);
    }

    #[test]
    fn test_tagged_to_json_int() {
        let v = new_tagged_int(42);
        let json = tagged_to_json(&v);
        assert!(json.contains("\"tag\":\"int\""));
        assert!(json.contains("\"value\":42"));
    }

    #[test]
    fn test_tagged_to_json_string() {
        let v = new_tagged_string("hi");
        let json = tagged_to_json(&v);
        assert!(json.contains("\"tag\":\"string\""));
    }

    #[test]
    fn test_equality() {
        let a = new_tagged_int(10);
        let b = new_tagged_int(10);
        assert_eq!(a, b);
    }
}
