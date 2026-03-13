// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Minimal YAML parser stub (scalar key-value pairs only).

/// A YAML scalar value.
#[derive(Debug, Clone, PartialEq)]
pub enum YamlScalar {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

/// A YAML parse error.
#[derive(Debug, Clone, PartialEq)]
pub struct YamlError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for YamlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "YAML error at line {}: {}", self.line, self.message)
    }
}

/// A flat YAML document (key → scalar).
#[derive(Debug, Clone, Default)]
pub struct YamlDocument {
    entries: Vec<(String, YamlScalar)>,
}

impl YamlDocument {
    /// Create an empty document.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a key-value pair.
    pub fn insert(&mut self, key: impl Into<String>, val: YamlScalar) {
        self.entries.push((key.into(), val));
    }

    /// Look up a scalar by key.
    pub fn get(&self, key: &str) -> Option<&YamlScalar> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Return the number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Return `true` if empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Parse a single YAML scalar line (`key: value`).
pub fn parse_scalar_line(
    line: &str,
    lineno: usize,
) -> Result<Option<(String, YamlScalar)>, YamlError> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    let mut parts = line.splitn(2, ':');
    let key = parts
        .next()
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let raw = parts.next().map(|s| s.trim()).unwrap_or("");
    if key.is_empty() {
        return Err(YamlError {
            line: lineno,
            message: "empty key".to_string(),
        });
    }
    let scalar = parse_scalar(raw);
    Ok(Some((key, scalar)))
}

/// Parse a raw scalar string to a `YamlScalar`.
pub fn parse_scalar(raw: &str) -> YamlScalar {
    if raw == "~" || raw == "null" || raw.is_empty() {
        return YamlScalar::Null;
    }
    if raw == "true" || raw == "yes" {
        return YamlScalar::Bool(true);
    }
    if raw == "false" || raw == "no" {
        return YamlScalar::Bool(false);
    }
    if let Ok(i) = raw.parse::<i64>() {
        return YamlScalar::Int(i);
    }
    if let Ok(f) = raw.parse::<f64>() {
        return YamlScalar::Float(f);
    }
    /* strip optional quotes */
    let s = if (raw.starts_with('"') && raw.ends_with('"'))
        || (raw.starts_with('\'') && raw.ends_with('\''))
    {
        raw[1..raw.len().saturating_sub(1)].to_string()
    } else {
        raw.to_string()
    };
    YamlScalar::Str(s)
}

/// Parse a multiline YAML string.
pub fn parse_yaml(input: &str) -> Result<YamlDocument, YamlError> {
    let mut doc = YamlDocument::new();
    for (i, line) in input.lines().enumerate() {
        if let Some((k, v)) = parse_scalar_line(line, i + 1)? {
            doc.insert(k, v);
        }
    }
    Ok(doc)
}

/// Retrieve an integer value.
pub fn get_int(doc: &YamlDocument, key: &str) -> Option<i64> {
    doc.get(key).and_then(|v| {
        if let YamlScalar::Int(i) = v {
            Some(*i)
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
        /* new doc is empty */
        assert!(YamlDocument::new().is_empty());
    }

    #[test]
    fn test_parse_int() {
        /* integer scalar */
        let doc = parse_yaml("port: 9090\n").expect("should succeed");
        assert_eq!(get_int(&doc, "port"), Some(9090));
    }

    #[test]
    fn test_parse_bool_true() {
        /* true keyword */
        let doc = parse_yaml("enabled: true\n").expect("should succeed");
        assert_eq!(doc.get("enabled"), Some(&YamlScalar::Bool(true)));
    }

    #[test]
    fn test_parse_bool_false() {
        /* false keyword */
        let doc = parse_yaml("enabled: false\n").expect("should succeed");
        assert_eq!(doc.get("enabled"), Some(&YamlScalar::Bool(false)));
    }

    #[test]
    fn test_parse_null() {
        /* null value */
        let doc = parse_yaml("x: null\n").expect("should succeed");
        assert_eq!(doc.get("x"), Some(&YamlScalar::Null));
    }

    #[test]
    fn test_parse_string() {
        /* string value */
        let doc = parse_yaml("name: oxihuman\n").expect("should succeed");
        assert_eq!(
            doc.get("name"),
            Some(&YamlScalar::Str("oxihuman".to_string()))
        );
    }

    #[test]
    fn test_comment_skipped() {
        /* comment lines are ignored */
        let doc = parse_yaml("# comment\nkey: 1\n").expect("should succeed");
        assert_eq!(doc.len(), 1);
    }

    #[test]
    fn test_parse_float() {
        /* float scalar */
        let doc = parse_yaml("ratio: 3.14\n").expect("should succeed");
        assert!(matches!(doc.get("ratio"), Some(YamlScalar::Float(_))));
    }

    #[test]
    fn test_insert_get() {
        /* insert and get */
        let mut doc = YamlDocument::new();
        doc.insert("k", YamlScalar::Int(7));
        assert_eq!(doc.get("k"), Some(&YamlScalar::Int(7)));
    }

    #[test]
    fn test_yes_no_boolean() {
        /* yes/no are booleans */
        assert_eq!(parse_scalar("yes"), YamlScalar::Bool(true));
        assert_eq!(parse_scalar("no"), YamlScalar::Bool(false));
    }
}
