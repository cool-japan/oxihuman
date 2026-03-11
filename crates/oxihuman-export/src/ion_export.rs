// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Amazon Ion text format encoding stub.

/// Ion value types.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum IonValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Symbol(String),
    List(Vec<IonValue>),
    Struct(Vec<(String, IonValue)>),
}

impl IonValue {
    /// Serialize to Ion text format.
    #[allow(dead_code)]
    pub fn to_ion_text(&self) -> String {
        match self {
            IonValue::Null => "null".to_string(),
            IonValue::Bool(b) => b.to_string(),
            IonValue::Int(i) => i.to_string(),
            IonValue::Float(f) => format!("{}e0", f),
            IonValue::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            IonValue::Symbol(s) => s.clone(),
            IonValue::List(items) => {
                let inner: Vec<String> = items.iter().map(|v| v.to_ion_text()).collect();
                format!("[{}]", inner.join(", "))
            }
            IonValue::Struct(fields) => {
                let inner: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_ion_text()))
                    .collect();
                format!("{{{}}}", inner.join(", "))
            }
        }
    }
}

/// An Ion document (sequence of top-level values).
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct IonDocument {
    pub values: Vec<IonValue>,
}

impl IonDocument {
    #[allow(dead_code)]
    pub fn new() -> Self {
        IonDocument::default()
    }

    #[allow(dead_code)]
    pub fn push(&mut self, val: IonValue) {
        self.values.push(val);
    }
}

/// Export an Ion document to text.
#[allow(dead_code)]
pub fn export_ion(doc: &IonDocument) -> String {
    doc.values
        .iter()
        .map(|v| v.to_ion_text())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Value count in document.
#[allow(dead_code)]
pub fn ion_value_count(doc: &IonDocument) -> usize {
    doc.values.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_text() {
        assert_eq!(IonValue::Null.to_ion_text(), "null");
    }

    #[test]
    fn bool_true_text() {
        assert_eq!(IonValue::Bool(true).to_ion_text(), "true");
    }

    #[test]
    fn bool_false_text() {
        assert_eq!(IonValue::Bool(false).to_ion_text(), "false");
    }

    #[test]
    fn int_text() {
        assert_eq!(IonValue::Int(42).to_ion_text(), "42");
    }

    #[test]
    fn float_text_has_e() {
        let s = IonValue::Float(std::f64::consts::PI).to_ion_text();
        assert!(s.contains('e'));
    }

    #[test]
    fn string_quoted() {
        let s = IonValue::String("hello".to_string()).to_ion_text();
        assert!(s.starts_with('"') && s.ends_with('"'));
    }

    #[test]
    fn list_brackets() {
        let v = IonValue::List(vec![IonValue::Int(1), IonValue::Int(2)]);
        let s = v.to_ion_text();
        assert!(s.starts_with('[') && s.ends_with(']'));
    }

    #[test]
    fn struct_braces() {
        let v = IonValue::Struct(vec![("x".to_string(), IonValue::Int(1))]);
        let s = v.to_ion_text();
        assert!(s.starts_with('{') && s.ends_with('}'));
    }

    #[test]
    fn document_export_empty() {
        let doc = IonDocument::new();
        assert_eq!(export_ion(&doc), "");
    }

    #[test]
    fn document_count() {
        let mut doc = IonDocument::new();
        doc.push(IonValue::Null);
        doc.push(IonValue::Bool(true));
        assert_eq!(ion_value_count(&doc), 2);
    }
}
