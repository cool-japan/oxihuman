// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! YAML scene description export.

#[derive(Debug, Clone)]
pub enum YamlValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<YamlValue>),
    Map(Vec<(String, YamlValue)>),
}

pub fn yaml_str(s: &str) -> YamlValue {
    YamlValue::Str(s.to_string())
}
pub fn yaml_int(i: i64) -> YamlValue {
    YamlValue::Int(i)
}
pub fn yaml_float(f: f64) -> YamlValue {
    YamlValue::Float(f)
}
pub fn yaml_bool(b: bool) -> YamlValue {
    YamlValue::Bool(b)
}
pub fn yaml_null() -> YamlValue {
    YamlValue::Null
}
pub fn yaml_list(items: Vec<YamlValue>) -> YamlValue {
    YamlValue::List(items)
}
pub fn yaml_map(entries: Vec<(String, YamlValue)>) -> YamlValue {
    YamlValue::Map(entries)
}

fn render_yaml_value(v: &YamlValue, indent: usize) -> String {
    let pad = " ".repeat(indent);
    match v {
        YamlValue::Null => "null".to_string(),
        YamlValue::Bool(b) => b.to_string(),
        YamlValue::Int(i) => i.to_string(),
        YamlValue::Float(f) => format!("{}", f),
        YamlValue::Str(s) => format!("\"{}\"", s.replace('"', "\\\"")),
        YamlValue::List(items) => {
            if items.is_empty() {
                return "[]".to_string();
            }
            items
                .iter()
                .map(|i| format!("\n{}- {}", pad, render_yaml_value(i, indent + 2)))
                .collect()
        }
        YamlValue::Map(entries) => {
            if entries.is_empty() {
                return "{}".to_string();
            }
            entries
                .iter()
                .map(|(k, v)| {
                    let val = render_yaml_value(v, indent + 2);
                    format!("\n{}{}: {}", pad, k, val)
                })
                .collect()
        }
    }
}

pub fn render_yaml(root: &YamlValue) -> String {
    format!("---{}\n", render_yaml_value(root, 0))
}

pub fn export_yaml(root: &YamlValue) -> Vec<u8> {
    render_yaml(root).into_bytes()
}
pub fn yaml_size_bytes(root: &YamlValue) -> usize {
    render_yaml(root).len()
}

pub fn scene_to_yaml(name: &str, mesh_count: usize, texture_count: usize) -> YamlValue {
    yaml_map(vec![(
        "scene".to_string(),
        yaml_map(vec![
            ("name".to_string(), yaml_str(name)),
            ("mesh_count".to_string(), yaml_int(mesh_count as i64)),
            ("texture_count".to_string(), yaml_int(texture_count as i64)),
        ]),
    )])
}

pub fn yaml_map_get<'a>(map: &'a YamlValue, key: &str) -> Option<&'a YamlValue> {
    if let YamlValue::Map(entries) = map {
        entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    } else {
        None
    }
}

pub fn yaml_list_len(list: &YamlValue) -> usize {
    if let YamlValue::List(items) = list {
        items.len()
    } else {
        0
    }
}

pub fn validate_yaml_value(v: &YamlValue) -> bool {
    match v {
        YamlValue::Float(f) => f.is_finite(),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_str() {
        let v = yaml_str("hello");
        let s = render_yaml_value(&v, 0);
        assert!(s.contains("hello"));
    }

    #[test]
    fn test_yaml_int() {
        let v = yaml_int(42);
        let s = render_yaml_value(&v, 0);
        assert!(s.contains("42"));
    }

    #[test]
    fn test_render_yaml_contains_separator() {
        let v = yaml_null();
        let s = render_yaml(&v);
        assert!(s.contains("---"));
    }

    #[test]
    fn test_export_yaml_nonempty() {
        let v = yaml_bool(true);
        assert!(!export_yaml(&v).is_empty());
    }

    #[test]
    fn test_scene_to_yaml() {
        let v = scene_to_yaml("myScene", 3, 2);
        if let YamlValue::Map(e) = &v {
            assert!(!e.is_empty());
        }
    }

    #[test]
    fn test_yaml_list_len() {
        let v = yaml_list(vec![yaml_int(1), yaml_int(2)]);
        assert_eq!(yaml_list_len(&v), 2);
    }

    #[test]
    fn test_validate_yaml_value_ok() {
        assert!(validate_yaml_value(&yaml_float(1.0)));
    }

    #[test]
    fn test_yaml_map_get() {
        let m = yaml_map(vec![("key".to_string(), yaml_str("val"))]);
        let v = yaml_map_get(&m, "key");
        assert!(v.is_some());
    }
}
