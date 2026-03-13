// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Minimal TOML parser stub.

/// A TOML value.
#[derive(Debug, Clone, PartialEq)]
pub enum TomlValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<TomlValue>),
    Table(Vec<(String, TomlValue)>),
}

/// Parse error for TOML.
#[derive(Debug, Clone, PartialEq)]
pub struct TomlParseError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for TomlParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TOML parse error at line {}: {}",
            self.line, self.message
        )
    }
}

/// A parsed TOML document (flat key-value map stub).
#[derive(Debug, Clone, Default)]
pub struct TomlDocument {
    entries: Vec<(String, TomlValue)>,
}

impl TomlDocument {
    /// Create an empty TOML document.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a key-value pair.
    pub fn insert(&mut self, key: impl Into<String>, value: TomlValue) {
        self.entries.push((key.into(), value));
    }

    /// Look up a key.
    pub fn get(&self, key: &str) -> Option<&TomlValue> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Return the number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Return `true` if the document is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Return all keys.
    pub fn keys(&self) -> Vec<&str> {
        self.entries.iter().map(|(k, _)| k.as_str()).collect()
    }
}

/// Parse a bare TOML key = value line (stub: integers and strings only).
pub fn parse_line(
    line: &str,
    lineno: usize,
) -> Result<Option<(String, TomlValue)>, TomlParseError> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    let mut parts = line.splitn(2, '=');
    let key = parts
        .next()
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let raw_val = parts.next().map(|s| s.trim()).unwrap_or("");
    if key.is_empty() {
        return Err(TomlParseError {
            line: lineno,
            message: "missing key".to_string(),
        });
    }
    /* integer */
    if let Ok(i) = raw_val.parse::<i64>() {
        return Ok(Some((key, TomlValue::Integer(i))));
    }
    /* boolean */
    if raw_val == "true" {
        return Ok(Some((key, TomlValue::Boolean(true))));
    }
    if raw_val == "false" {
        return Ok(Some((key, TomlValue::Boolean(false))));
    }
    /* quoted string */
    if raw_val.starts_with('"') && raw_val.ends_with('"') && raw_val.len() >= 2 {
        let s = raw_val[1..raw_val.len().saturating_sub(1)].to_string();
        return Ok(Some((key, TomlValue::String(s))));
    }
    Err(TomlParseError {
        line: lineno,
        message: format!("unsupported value: {raw_val}"),
    })
}

/// Parse a multiline TOML string into a document.
pub fn parse_toml(input: &str) -> Result<TomlDocument, TomlParseError> {
    let mut doc = TomlDocument::new();
    for (i, line) in input.lines().enumerate() {
        if let Some((k, v)) = parse_line(line, i + 1)? {
            doc.insert(k, v);
        }
    }
    Ok(doc)
}

/// Return the integer value for a key, or `None`.
pub fn get_integer(doc: &TomlDocument, key: &str) -> Option<i64> {
    doc.get(key).and_then(|v| {
        if let TomlValue::Integer(i) = v {
            Some(*i)
        } else {
            None
        }
    })
}

/// Return the string value for a key, or `None`.
pub fn get_string<'a>(doc: &'a TomlDocument, key: &str) -> Option<&'a str> {
    doc.get(key).and_then(|v| {
        if let TomlValue::String(s) = v {
            Some(s.as_str())
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_doc() {
        /* empty document */
        let doc = TomlDocument::new();
        assert!(doc.is_empty());
    }

    #[test]
    fn test_insert_get() {
        /* insert then get */
        let mut doc = TomlDocument::new();
        doc.insert("x", TomlValue::Integer(42));
        assert!(doc.get("x").is_some());
    }

    #[test]
    fn test_parse_integer_line() {
        /* integer value parsed correctly */
        let r = parse_line("port = 8080", 1)
            .expect("should succeed")
            .expect("should succeed");
        assert_eq!(r.0, "port");
        assert_eq!(r.1, TomlValue::Integer(8080));
    }

    #[test]
    fn test_parse_string_line() {
        /* quoted string parsed correctly */
        let r = parse_line("name = \"hello\"", 1)
            .expect("should succeed")
            .expect("should succeed");
        assert_eq!(r.1, TomlValue::String("hello".to_string()));
    }

    #[test]
    fn test_parse_boolean() {
        /* boolean true/false */
        let r = parse_line("flag = true", 1)
            .expect("should succeed")
            .expect("should succeed");
        assert_eq!(r.1, TomlValue::Boolean(true));
    }

    #[test]
    fn test_comment_line_skipped() {
        /* comment lines return None */
        assert!(parse_line("# comment", 1)
            .expect("should succeed")
            .is_none());
    }

    #[test]
    fn test_get_integer() {
        /* get_integer helper */
        let doc = parse_toml("workers = 4\n").expect("should succeed");
        assert_eq!(get_integer(&doc, "workers"), Some(4));
    }

    #[test]
    fn test_get_string_helper() {
        /* get_string helper */
        let doc = parse_toml("app = \"oxihuman\"\n").expect("should succeed");
        assert_eq!(get_string(&doc, "app"), Some("oxihuman"));
    }

    #[test]
    fn test_keys() {
        /* keys returns all key names */
        let doc = parse_toml("a = 1\nb = 2\n").expect("should succeed");
        let keys = doc.keys();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_unsupported_value_error() {
        /* unsupported value type returns error */
        assert!(parse_line("x = [1,2,3]", 1).is_err());
    }
}
