// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// Reads key=value configuration from text.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConfigReader {
    values: HashMap<String, String>,
    sections: Vec<String>,
}

impl Default for ConfigReader {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl ConfigReader {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            sections: Vec::new(),
        }
    }

    pub fn parse(text: &str) -> Self {
        let mut reader = Self::new();
        let mut current_section = String::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
                continue;
            }
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed[1..trimmed.len() - 1].to_string();
                if !reader.sections.contains(&current_section) {
                    reader.sections.push(current_section.clone());
                }
                continue;
            }
            if let Some((key, value)) = trimmed.split_once('=') {
                let full_key = if current_section.is_empty() {
                    key.trim().to_string()
                } else {
                    format!("{}.{}", current_section, key.trim())
                };
                reader.values.insert(full_key, value.trim().to_string());
            }
        }
        reader
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.values
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.values.get(key).and_then(|v| v.parse().ok())
    }

    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.values.get(key).and_then(|v| v.parse().ok())
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.values.get(key).and_then(|v| match v.as_str() {
            "true" | "1" | "yes" => Some(true),
            "false" | "0" | "no" => Some(false),
            _ => None,
        })
    }

    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }

    pub fn sections(&self) -> &[String] {
        &self.sections
    }

    pub fn keys_in_section(&self, section: &str) -> Vec<String> {
        let prefix = format!("{section}.");
        self.values
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect()
    }

    pub fn all_keys(&self) -> Vec<&str> {
        self.values.keys().map(|k| k.as_str()).collect()
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let r = ConfigReader::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_parse_simple() {
        let r = ConfigReader::parse("key=value\nfoo=bar");
        assert_eq!(r.get("key"), Some("value"));
        assert_eq!(r.get("foo"), Some("bar"));
    }

    #[test]
    fn test_parse_sections() {
        let r = ConfigReader::parse("[section]\nk=v");
        assert_eq!(r.get("section.k"), Some("v"));
        assert_eq!(r.sections(), &["section"]);
    }

    #[test]
    fn test_comments_ignored() {
        let r = ConfigReader::parse("# comment\n; another\nk=v");
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_get_int() {
        let r = ConfigReader::parse("n=42");
        assert_eq!(r.get_int("n"), Some(42));
    }

    #[test]
    fn test_get_float() {
        let r = ConfigReader::parse("f=2.75");
        let v = r.get_float("f").unwrap();
        assert!((v - 2.75).abs() < 1e-6);
    }

    #[test]
    fn test_get_bool() {
        let r = ConfigReader::parse("a=true\nb=false\nc=yes");
        assert_eq!(r.get_bool("a"), Some(true));
        assert_eq!(r.get_bool("b"), Some(false));
        assert_eq!(r.get_bool("c"), Some(true));
    }

    #[test]
    fn test_get_or_default() {
        let r = ConfigReader::new();
        assert_eq!(r.get_or("missing", "default"), "default");
    }

    #[test]
    fn test_contains() {
        let r = ConfigReader::parse("k=v");
        assert!(r.contains("k"));
        assert!(!r.contains("x"));
    }

    #[test]
    fn test_set() {
        let mut r = ConfigReader::new();
        r.set("k", "v");
        assert_eq!(r.get("k"), Some("v"));
    }
}
