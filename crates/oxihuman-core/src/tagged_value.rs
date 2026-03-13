// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A dynamically-typed tagged value, useful for config entries and property bags.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Default)]
pub enum TaggedValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    List(Vec<TaggedValue>),
    #[default]
    None,
}

#[allow(dead_code)]
impl TaggedValue {
    pub fn is_int(&self) -> bool { matches!(self, Self::Int(_)) }
    pub fn is_float(&self) -> bool { matches!(self, Self::Float(_)) }
    pub fn is_bool(&self) -> bool { matches!(self, Self::Bool(_)) }
    pub fn is_str(&self) -> bool { matches!(self, Self::Str(_)) }
    pub fn is_list(&self) -> bool { matches!(self, Self::List(_)) }
    pub fn is_none(&self) -> bool { matches!(self, Self::None) }

    pub fn as_int(&self) -> Option<i64> {
        if let Self::Int(v) = self { Some(*v) } else { None }
    }

    pub fn as_float(&self) -> Option<f64> {
        if let Self::Float(v) = self { Some(*v) } else { None }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(v) = self { Some(*v) } else { None }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let Self::Str(v) = self { Some(v) } else { None }
    }

    pub fn as_list(&self) -> Option<&[TaggedValue]> {
        if let Self::List(v) = self { Some(v) } else { None }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Int(_) => "int",
            Self::Float(_) => "float",
            Self::Bool(_) => "bool",
            Self::Str(_) => "str",
            Self::List(_) => "list",
            Self::None => "none",
        }
    }

    pub fn to_json_string(&self) -> String {
        match self {
            Self::Int(v) => format!("{v}"),
            Self::Float(v) => format!("{v}"),
            Self::Bool(v) => format!("{v}"),
            Self::Str(v) => format!("\"{v}\""),
            Self::List(items) => {
                let inner: Vec<String> = items.iter().map(|i| i.to_json_string()).collect();
                format!("[{}]", inner.join(","))
            }
            Self::None => "null".to_string(),
        }
    }
}

// Default is derived via #[default] on None variant

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int() {
        let v = TaggedValue::Int(42);
        assert!(v.is_int());
        assert_eq!(v.as_int(), Some(42));
    }

    #[test]
    fn test_float() {
        let v = TaggedValue::Float(1.234);
        assert!(v.is_float());
        assert!((v.as_float().expect("should succeed") - 1.234).abs() < 0.001);
    }

    #[test]
    fn test_bool() {
        let v = TaggedValue::Bool(true);
        assert!(v.is_bool());
        assert_eq!(v.as_bool(), Some(true));
    }

    #[test]
    fn test_str() {
        let v = TaggedValue::Str("hello".to_string());
        assert!(v.is_str());
        assert_eq!(v.as_str(), Some("hello"));
    }

    #[test]
    fn test_list() {
        let v = TaggedValue::List(vec![TaggedValue::Int(1), TaggedValue::Int(2)]);
        assert!(v.is_list());
        assert_eq!(v.as_list().expect("should succeed").len(), 2);
    }

    #[test]
    fn test_none() {
        let v = TaggedValue::None;
        assert!(v.is_none());
    }

    #[test]
    fn test_type_name() {
        assert_eq!(TaggedValue::Int(0).type_name(), "int");
        assert_eq!(TaggedValue::None.type_name(), "none");
    }

    #[test]
    fn test_json_string() {
        assert_eq!(TaggedValue::Int(5).to_json_string(), "5");
        assert_eq!(TaggedValue::Bool(true).to_json_string(), "true");
        assert_eq!(TaggedValue::None.to_json_string(), "null");
    }

    #[test]
    fn test_wrong_type_returns_none() {
        let v = TaggedValue::Int(1);
        assert_eq!(v.as_str(), None);
        assert_eq!(v.as_bool(), None);
    }

    #[test]
    fn test_default() {
        let v = TaggedValue::default();
        assert!(v.is_none());
    }
}
