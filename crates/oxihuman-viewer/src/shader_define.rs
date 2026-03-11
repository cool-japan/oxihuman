// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A single shader preprocessor define.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShaderDefine {
    pub name: String,
    pub value: Option<String>,
}

/// A set of shader defines.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DefineSet {
    pub defines: Vec<ShaderDefine>,
}

/// Create a new empty define set.
#[allow(dead_code)]
pub fn new_define_set() -> DefineSet {
    DefineSet {
        defines: Vec::new(),
    }
}

/// Add a define to the set.
#[allow(dead_code)]
pub fn add_define(set: &mut DefineSet, name: &str, value: Option<&str>) {
    if !has_define(set, name) {
        set.defines.push(ShaderDefine {
            name: name.to_string(),
            value: value.map(|v| v.to_string()),
        });
    }
}

/// Remove a define from the set.
#[allow(dead_code)]
pub fn remove_define(set: &mut DefineSet, name: &str) {
    set.defines.retain(|d| d.name != name);
}

/// Check if a define exists in the set.
#[allow(dead_code)]
pub fn has_define(set: &DefineSet, name: &str) -> bool {
    set.defines.iter().any(|d| d.name == name)
}

/// Return the number of defines.
#[allow(dead_code)]
pub fn define_count(set: &DefineSet) -> usize {
    set.defines.len()
}

/// Convert defines to a preprocessor string.
#[allow(dead_code)]
pub fn defines_to_string(set: &DefineSet) -> String {
    set.defines
        .iter()
        .map(|d| match &d.value {
            Some(v) => format!("#define {} {}", d.name, v),
            None => format!("#define {}", d.name),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn define_set_to_json(set: &DefineSet) -> String {
    let entries: Vec<String> = set
        .defines
        .iter()
        .map(|d| match &d.value {
            Some(v) => format!("{{\"name\":\"{}\",\"value\":\"{}\"}}", d.name, v),
            None => format!("{{\"name\":\"{}\"}}", d.name),
        })
        .collect();
    format!("[{}]", entries.join(","))
}

/// Clear all defines.
#[allow(dead_code)]
pub fn clear_defines(set: &mut DefineSet) {
    set.defines.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_set_empty() {
        let s = new_define_set();
        assert_eq!(define_count(&s), 0);
    }

    #[test]
    fn add_define_works() {
        let mut s = new_define_set();
        add_define(&mut s, "USE_NORMAL_MAP", None);
        assert_eq!(define_count(&s), 1);
    }

    #[test]
    fn has_define_check() {
        let mut s = new_define_set();
        add_define(&mut s, "SHADOWS", None);
        assert!(has_define(&s, "SHADOWS"));
        assert!(!has_define(&s, "FOG"));
    }

    #[test]
    fn remove_define_works() {
        let mut s = new_define_set();
        add_define(&mut s, "A", None);
        remove_define(&mut s, "A");
        assert!(!has_define(&s, "A"));
    }

    #[test]
    fn no_duplicates() {
        let mut s = new_define_set();
        add_define(&mut s, "A", None);
        add_define(&mut s, "A", None);
        assert_eq!(define_count(&s), 1);
    }

    #[test]
    fn to_preprocessor_string() {
        let mut s = new_define_set();
        add_define(&mut s, "MAX_LIGHTS", Some("8"));
        let text = defines_to_string(&s);
        assert!(text.contains("#define MAX_LIGHTS 8"));
    }

    #[test]
    fn to_json() {
        let mut s = new_define_set();
        add_define(&mut s, "TEST", None);
        let j = define_set_to_json(&s);
        assert!(j.contains("TEST"));
    }

    #[test]
    fn clear_works() {
        let mut s = new_define_set();
        add_define(&mut s, "A", None);
        clear_defines(&mut s);
        assert_eq!(define_count(&s), 0);
    }

    #[test]
    fn define_with_value() {
        let mut s = new_define_set();
        add_define(&mut s, "VERSION", Some("300"));
        let text = defines_to_string(&s);
        assert!(text.contains("300"));
    }

    #[test]
    fn define_without_value() {
        let mut s = new_define_set();
        add_define(&mut s, "FLAG", None);
        let text = defines_to_string(&s);
        assert_eq!(text, "#define FLAG");
    }
}
