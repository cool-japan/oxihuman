// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! TOML configuration export.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum TomlValue {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<TomlValue>),
}

#[derive(Debug, Clone, Default)]
pub struct TomlTable {
    pub name: String,
    pub entries: HashMap<String, TomlValue>,
    pub sub_tables: Vec<TomlTable>,
}

pub fn new_toml_table(name: &str) -> TomlTable {
    TomlTable {
        name: name.to_string(),
        entries: HashMap::new(),
        sub_tables: Vec::new(),
    }
}

pub fn toml_set_string(table: &mut TomlTable, key: &str, value: &str) {
    table
        .entries
        .insert(key.to_string(), TomlValue::String(value.to_string()));
}

pub fn toml_set_int(table: &mut TomlTable, key: &str, value: i64) {
    table
        .entries
        .insert(key.to_string(), TomlValue::Integer(value));
}

pub fn toml_set_float(table: &mut TomlTable, key: &str, value: f64) {
    table
        .entries
        .insert(key.to_string(), TomlValue::Float(value));
}

pub fn toml_set_bool(table: &mut TomlTable, key: &str, value: bool) {
    table
        .entries
        .insert(key.to_string(), TomlValue::Bool(value));
}

pub fn toml_add_sub_table(table: &mut TomlTable, sub: TomlTable) {
    table.sub_tables.push(sub);
}

fn render_toml_value(v: &TomlValue) -> String {
    match v {
        TomlValue::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
        TomlValue::Integer(i) => i.to_string(),
        TomlValue::Float(f) => format!("{}", f),
        TomlValue::Bool(b) => b.to_string(),
        TomlValue::Array(arr) => {
            let items: Vec<String> = arr.iter().map(render_toml_value).collect();
            format!("[{}]", items.join(", "))
        }
    }
}

pub fn render_toml_table(table: &TomlTable, prefix: &str) -> String {
    let mut s = String::new();
    let full_name = if prefix.is_empty() || table.name.is_empty() {
        table.name.clone()
    } else {
        format!("{}.{}", prefix, table.name)
    };
    if !full_name.is_empty() {
        s.push_str(&format!("[{}]\n", full_name));
    }
    let mut keys: Vec<&String> = table.entries.keys().collect();
    keys.sort();
    for key in keys {
        s.push_str(&format!(
            "{} = {}\n",
            key,
            render_toml_value(&table.entries[key])
        ));
    }
    for sub in &table.sub_tables {
        s.push('\n');
        s.push_str(&render_toml_table(sub, &full_name));
    }
    s
}

pub fn export_toml(table: &TomlTable) -> Vec<u8> {
    render_toml_table(table, "").into_bytes()
}

pub fn toml_entry_count(table: &TomlTable) -> usize {
    table.entries.len()
}
pub fn validate_toml_table(table: &TomlTable) -> bool {
    !table.entries.is_empty() || !table.sub_tables.is_empty() || !table.name.is_empty()
}
pub fn toml_size_bytes(table: &TomlTable) -> usize {
    render_toml_table(table, "").len()
}

pub fn default_export_config_toml() -> TomlTable {
    let mut t = new_toml_table("export");
    toml_set_string(&mut t, "format", "glb");
    toml_set_bool(&mut t, "embed_textures", true);
    toml_set_float(&mut t, "lod_distance", 5.0);
    toml_set_int(&mut t, "lod_levels", 3);
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_toml_table() {
        let t = new_toml_table("cfg");
        assert_eq!(t.name, "cfg");
    }

    #[test]
    fn test_toml_set_entries() {
        let mut t = new_toml_table("t");
        toml_set_string(&mut t, "k", "v");
        toml_set_int(&mut t, "n", 42);
        assert_eq!(toml_entry_count(&t), 2);
    }

    #[test]
    fn test_render_toml_contains_section() {
        let mut t = new_toml_table("cfg");
        toml_set_string(&mut t, "x", "y");
        let s = render_toml_table(&t, "");
        assert!(s.contains("[cfg]"));
    }

    #[test]
    fn test_export_toml_nonempty() {
        let t = new_toml_table("t");
        assert!(!export_toml(&t).is_empty() || t.name.is_empty());
        let mut t2 = new_toml_table("t2");
        toml_set_bool(&mut t2, "x", true);
        assert!(!export_toml(&t2).is_empty());
    }

    #[test]
    fn test_validate_toml_table() {
        let t = new_toml_table("t");
        assert!(validate_toml_table(&t));
    }

    #[test]
    fn test_default_export_config_toml() {
        let t = default_export_config_toml();
        assert!(toml_entry_count(&t) >= 3);
    }

    #[test]
    fn test_render_bool() {
        let mut t = new_toml_table("t");
        toml_set_bool(&mut t, "flag", true);
        let s = render_toml_table(&t, "");
        assert!(s.contains("true"));
    }

    #[test]
    fn test_toml_size_bytes() {
        let mut t = new_toml_table("t");
        toml_set_string(&mut t, "x", "y");
        assert!(toml_size_bytes(&t) > 0);
    }
}
