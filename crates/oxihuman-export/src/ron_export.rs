// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! RON (Rusty Object Notation) export.

#[derive(Debug, Clone)]
pub enum RonValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Option(Option<Box<RonValue>>),
    List(Vec<RonValue>),
    Map(Vec<(String, RonValue)>),
    Struct {
        name: String,
        fields: Vec<(String, RonValue)>,
    },
}

pub fn ron_bool(b: bool) -> RonValue {
    RonValue::Bool(b)
}
pub fn ron_int(i: i64) -> RonValue {
    RonValue::Int(i)
}
pub fn ron_float(f: f64) -> RonValue {
    RonValue::Float(f)
}
pub fn ron_str(s: &str) -> RonValue {
    RonValue::Str(s.to_string())
}
pub fn ron_none() -> RonValue {
    RonValue::Option(None)
}
pub fn ron_some(v: RonValue) -> RonValue {
    RonValue::Option(Some(Box::new(v)))
}
pub fn ron_list(items: Vec<RonValue>) -> RonValue {
    RonValue::List(items)
}
pub fn ron_map(entries: Vec<(String, RonValue)>) -> RonValue {
    RonValue::Map(entries)
}
pub fn ron_struct(name: &str, fields: Vec<(String, RonValue)>) -> RonValue {
    RonValue::Struct {
        name: name.to_string(),
        fields,
    }
}

fn render_ron_value(v: &RonValue) -> String {
    match v {
        RonValue::Bool(b) => b.to_string(),
        RonValue::Int(i) => i.to_string(),
        RonValue::Float(f) => format!("{:?}", f),
        RonValue::Str(s) => format!("\"{}\"", s.replace('"', "\\\"")),
        RonValue::Option(None) => "None".to_string(),
        RonValue::Option(Some(inner)) => format!("Some({})", render_ron_value(inner)),
        RonValue::List(items) => {
            let inner: Vec<String> = items.iter().map(render_ron_value).collect();
            format!("[{}]", inner.join(", "))
        }
        RonValue::Map(entries) => {
            let inner: Vec<String> = entries
                .iter()
                .map(|(k, v)| format!("\"{}\": {}", k, render_ron_value(v)))
                .collect();
            format!("{{{}}}", inner.join(", "))
        }
        RonValue::Struct { name, fields } => {
            let inner: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k, render_ron_value(v)))
                .collect();
            format!("{}({})", name, inner.join(", "))
        }
    }
}

pub fn render_ron(root: &RonValue) -> String {
    format!("(\n    {}\n)\n", render_ron_value(root))
}

pub fn export_ron(root: &RonValue) -> Vec<u8> {
    render_ron(root).into_bytes()
}
pub fn ron_size_bytes(root: &RonValue) -> usize {
    render_ron(root).len()
}
pub fn validate_ron_value(v: &RonValue) -> bool {
    match v {
        RonValue::Float(f) => f.is_finite(),
        _ => true,
    }
}

pub fn scene_to_ron(name: &str, mesh_count: usize) -> RonValue {
    ron_struct(
        "Scene",
        vec![
            ("name".to_string(), ron_str(name)),
            ("mesh_count".to_string(), ron_int(mesh_count as i64)),
        ],
    )
}

pub fn ron_list_len(v: &RonValue) -> usize {
    if let RonValue::List(items) = v {
        items.len()
    } else {
        0
    }
}

pub fn ron_map_get<'a>(v: &'a RonValue, key: &str) -> Option<&'a RonValue> {
    if let RonValue::Map(entries) = v {
        entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ron_bool() {
        let s = render_ron_value(&ron_bool(true));
        assert_eq!(s, "true");
    }

    #[test]
    fn test_ron_int() {
        let s = render_ron_value(&ron_int(99));
        assert!(s.contains("99"));
    }

    #[test]
    fn test_ron_option_none() {
        let s = render_ron_value(&ron_none());
        assert_eq!(s, "None");
    }

    #[test]
    fn test_ron_option_some() {
        let s = render_ron_value(&ron_some(ron_int(1)));
        assert!(s.starts_with("Some("));
    }

    #[test]
    fn test_ron_struct() {
        let v = ron_struct("Foo", vec![("x".to_string(), ron_int(1))]);
        let s = render_ron_value(&v);
        assert!(s.contains("Foo"));
    }

    #[test]
    fn test_export_ron_nonempty() {
        let v = ron_bool(false);
        assert!(!export_ron(&v).is_empty());
    }

    #[test]
    fn test_validate_ron_value() {
        assert!(validate_ron_value(&ron_float(1.0)));
    }

    #[test]
    fn test_scene_to_ron() {
        let v = scene_to_ron("MyScene", 5);
        let s = render_ron_value(&v);
        assert!(s.contains("MyScene"));
    }
}
