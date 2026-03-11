// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A typed configuration value that can hold different data types.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigVal {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<ConfigVal>),
}

#[allow(dead_code)]
impl ConfigVal {
    pub fn as_bool(&self) -> Option<bool> {
        if let ConfigVal::Bool(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        if let ConfigVal::Int(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        if let ConfigVal::Float(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let ConfigVal::Str(v) = self {
            Some(v.as_str())
        } else {
            None
        }
    }

    pub fn as_list(&self) -> Option<&[ConfigVal]> {
        if let ConfigVal::List(v) = self {
            Some(v.as_slice())
        } else {
            None
        }
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, ConfigVal::Bool(_))
    }

    pub fn is_int(&self) -> bool {
        matches!(self, ConfigVal::Int(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, ConfigVal::Float(_))
    }

    pub fn is_str(&self) -> bool {
        matches!(self, ConfigVal::Str(_))
    }

    pub fn is_list(&self) -> bool {
        matches!(self, ConfigVal::List(_))
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            ConfigVal::Bool(_) => "bool",
            ConfigVal::Int(_) => "int",
            ConfigVal::Float(_) => "float",
            ConfigVal::Str(_) => "str",
            ConfigVal::List(_) => "list",
        }
    }

    pub fn to_f64(&self) -> Option<f64> {
        match self {
            ConfigVal::Float(v) => Some(*v),
            ConfigVal::Int(v) => Some(*v as f64),
            _ => None,
        }
    }
}

/// A configuration store mapping string keys to typed values.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConfigStore {
    entries: Vec<(String, ConfigVal)>,
}

#[allow(dead_code)]
impl ConfigStore {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn set(&mut self, key: &str, val: ConfigVal) {
        if let Some(entry) = self.entries.iter_mut().find(|(k, _)| k == key) {
            entry.1 = val;
        } else {
            self.entries.push((key.to_string(), val));
        }
    }

    pub fn get(&self, key: &str) -> Option<&ConfigVal> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn remove(&mut self, key: &str) -> bool {
        let len = self.entries.len();
        self.entries.retain(|(k, _)| k != key);
        self.entries.len() < len
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn keys(&self) -> Vec<&str> {
        self.entries.iter().map(|(k, _)| k.as_str()).collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for ConfigStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_val_bool() {
        let v = ConfigVal::Bool(true);
        assert_eq!(v.as_bool(), Some(true));
        assert!(v.is_bool());
        assert_eq!(v.type_name(), "bool");
    }

    #[test]
    fn test_config_val_int() {
        let v = ConfigVal::Int(42);
        assert_eq!(v.as_int(), Some(42));
        assert!(v.is_int());
    }

    #[test]
    fn test_config_val_float() {
        let v = ConfigVal::Float(2.78);
        assert_eq!(v.as_float(), Some(2.78));
        assert!(v.is_float());
    }

    #[test]
    fn test_config_val_str() {
        let v = ConfigVal::Str("hello".to_string());
        assert_eq!(v.as_str(), Some("hello"));
        assert!(v.is_str());
    }

    #[test]
    fn test_config_val_list() {
        let v = ConfigVal::List(vec![ConfigVal::Int(1), ConfigVal::Int(2)]);
        assert!(v.is_list());
        assert_eq!(v.as_list().unwrap().len(), 2);
    }

    #[test]
    fn test_to_f64() {
        assert_eq!(ConfigVal::Float(1.5).to_f64(), Some(1.5));
        assert_eq!(ConfigVal::Int(10).to_f64(), Some(10.0));
        assert_eq!(ConfigVal::Bool(true).to_f64(), None);
    }

    #[test]
    fn test_store_set_get() {
        let mut s = ConfigStore::new();
        s.set("key", ConfigVal::Int(99));
        assert_eq!(s.get("key").and_then(|v| v.as_int()), Some(99));
    }

    #[test]
    fn test_store_overwrite() {
        let mut s = ConfigStore::new();
        s.set("k", ConfigVal::Int(1));
        s.set("k", ConfigVal::Int(2));
        assert_eq!(s.get("k").and_then(|v| v.as_int()), Some(2));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn test_store_remove() {
        let mut s = ConfigStore::new();
        s.set("k", ConfigVal::Bool(true));
        assert!(s.remove("k"));
        assert!(s.is_empty());
    }

    #[test]
    fn test_store_keys() {
        let mut s = ConfigStore::new();
        s.set("a", ConfigVal::Int(1));
        s.set("b", ConfigVal::Int(2));
        let keys = s.keys();
        assert!(keys.contains(&"a"));
        assert!(keys.contains(&"b"));
    }
}
