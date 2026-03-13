// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Feature flag management system.

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum FlagValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct FeatureFlag {
    pub name: String,
    pub value: FlagValue,
    pub description: String,
    pub tags: Vec<String>,
    pub override_env: Option<String>,
}

#[allow(dead_code)]
pub struct FeatureFlagRegistry {
    pub flags: Vec<FeatureFlag>,
}

#[allow(dead_code)]
pub fn new_feature_registry() -> FeatureFlagRegistry {
    FeatureFlagRegistry { flags: Vec::new() }
}

#[allow(dead_code)]
pub fn register_flag(reg: &mut FeatureFlagRegistry, flag: FeatureFlag) {
    reg.flags.push(flag);
}

#[allow(dead_code)]
pub fn get_flag<'a>(reg: &'a FeatureFlagRegistry, name: &str) -> Option<&'a FeatureFlag> {
    reg.flags.iter().find(|f| f.name == name)
}

#[allow(dead_code)]
pub fn set_flag_value(reg: &mut FeatureFlagRegistry, name: &str, value: FlagValue) -> bool {
    if let Some(flag) = reg.flags.iter_mut().find(|f| f.name == name) {
        flag.value = value;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn is_enabled(reg: &FeatureFlagRegistry, name: &str) -> bool {
    match get_flag(reg, name) {
        Some(flag) => match &flag.value {
            FlagValue::Bool(b) => *b,
            _ => false,
        },
        None => false,
    }
}

#[allow(dead_code)]
pub fn flag_count(reg: &FeatureFlagRegistry) -> usize {
    reg.flags.len()
}

#[allow(dead_code)]
pub fn flags_with_tag<'a>(reg: &'a FeatureFlagRegistry, tag: &str) -> Vec<&'a FeatureFlag> {
    reg.flags
        .iter()
        .filter(|f| f.tags.iter().any(|t| t == tag))
        .collect()
}

#[allow(dead_code)]
pub fn all_enabled_flags(reg: &FeatureFlagRegistry) -> Vec<&FeatureFlag> {
    reg.flags
        .iter()
        .filter(|f| matches!(&f.value, FlagValue::Bool(true)))
        .collect()
}

#[allow(dead_code)]
pub fn default_bool_flag(name: &str, default: bool, description: &str) -> FeatureFlag {
    FeatureFlag {
        name: name.to_string(),
        value: FlagValue::Bool(default),
        description: description.to_string(),
        tags: Vec::new(),
        override_env: None,
    }
}

#[allow(dead_code)]
pub fn default_int_flag(name: &str, default: i64, description: &str) -> FeatureFlag {
    FeatureFlag {
        name: name.to_string(),
        value: FlagValue::Int(default),
        description: description.to_string(),
        tags: Vec::new(),
        override_env: None,
    }
}

#[allow(dead_code)]
pub fn get_flag_bool(reg: &FeatureFlagRegistry, name: &str) -> Option<bool> {
    match get_flag(reg, name) {
        Some(flag) => match &flag.value {
            FlagValue::Bool(b) => Some(*b),
            _ => None,
        },
        None => None,
    }
}

#[allow(dead_code)]
pub fn get_flag_int(reg: &FeatureFlagRegistry, name: &str) -> Option<i64> {
    match get_flag(reg, name) {
        Some(flag) => match &flag.value {
            FlagValue::Int(i) => Some(*i),
            _ => None,
        },
        None => None,
    }
}

#[allow(dead_code)]
pub fn feature_registry_to_json(reg: &FeatureFlagRegistry) -> String {
    let mut out = String::from("{\"flags\":[");
    for (i, flag) in reg.flags.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let val_str = match &flag.value {
            FlagValue::Bool(b) => format!("{}", b),
            FlagValue::Int(n) => format!("{}", n),
            FlagValue::Float(f) => format!("{}", f),
            FlagValue::Text(s) => format!("\"{}\"", s),
        };
        out.push_str(&format!(
            "{{\"name\":\"{}\",\"value\":{},\"description\":\"{}\"}}",
            flag.name, val_str, flag.description
        ));
    }
    out.push_str("]}");
    out
}

#[allow(dead_code)]
pub fn remove_flag(reg: &mut FeatureFlagRegistry, name: &str) -> bool {
    let before = reg.flags.len();
    reg.flags.retain(|f| f.name != name);
    reg.flags.len() < before
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feature_registry() {
        let reg = new_feature_registry();
        assert_eq!(flag_count(&reg), 0);
    }

    #[test]
    fn test_register_flag() {
        let mut reg = new_feature_registry();
        let flag = default_bool_flag("debug", true, "Enable debug mode");
        register_flag(&mut reg, flag);
        assert_eq!(flag_count(&reg), 1);
    }

    #[test]
    fn test_get_flag() {
        let mut reg = new_feature_registry();
        let flag = default_bool_flag("feature_x", false, "Feature X");
        register_flag(&mut reg, flag);
        assert!(get_flag(&reg, "feature_x").is_some());
        assert!(get_flag(&reg, "missing").is_none());
    }

    #[test]
    fn test_set_flag_value() {
        let mut reg = new_feature_registry();
        register_flag(&mut reg, default_bool_flag("beta", false, "Beta"));
        let ok = set_flag_value(&mut reg, "beta", FlagValue::Bool(true));
        assert!(ok);
        assert!(is_enabled(&reg, "beta"));
        let not_ok = set_flag_value(&mut reg, "nonexistent", FlagValue::Bool(true));
        assert!(!not_ok);
    }

    #[test]
    fn test_is_enabled() {
        let mut reg = new_feature_registry();
        register_flag(&mut reg, default_bool_flag("on", true, "On"));
        register_flag(&mut reg, default_bool_flag("off", false, "Off"));
        assert!(is_enabled(&reg, "on"));
        assert!(!is_enabled(&reg, "off"));
        assert!(!is_enabled(&reg, "missing"));
    }

    #[test]
    fn test_flag_count() {
        let mut reg = new_feature_registry();
        assert_eq!(flag_count(&reg), 0);
        register_flag(&mut reg, default_bool_flag("a", true, "A"));
        register_flag(&mut reg, default_bool_flag("b", false, "B"));
        assert_eq!(flag_count(&reg), 2);
    }

    #[test]
    fn test_flags_with_tag() {
        let mut reg = new_feature_registry();
        let mut flag = default_bool_flag("experimental", true, "Experimental feature");
        flag.tags.push("experimental".to_string());
        register_flag(&mut reg, flag);
        register_flag(
            &mut reg,
            default_bool_flag("stable", true, "Stable feature"),
        );
        let experimental = flags_with_tag(&reg, "experimental");
        assert_eq!(experimental.len(), 1);
        assert_eq!(experimental[0].name, "experimental");
    }

    #[test]
    fn test_all_enabled_flags() {
        let mut reg = new_feature_registry();
        register_flag(&mut reg, default_bool_flag("enabled1", true, "E1"));
        register_flag(&mut reg, default_bool_flag("disabled1", false, "D1"));
        register_flag(&mut reg, default_bool_flag("enabled2", true, "E2"));
        let enabled = all_enabled_flags(&reg);
        assert_eq!(enabled.len(), 2);
    }

    #[test]
    fn test_get_flag_bool() {
        let mut reg = new_feature_registry();
        register_flag(&mut reg, default_bool_flag("flag", true, "F"));
        assert_eq!(get_flag_bool(&reg, "flag"), Some(true));
        assert_eq!(get_flag_bool(&reg, "missing"), None);
    }

    #[test]
    fn test_get_flag_int() {
        let mut reg = new_feature_registry();
        register_flag(&mut reg, default_int_flag("max_count", 42, "Max count"));
        assert_eq!(get_flag_int(&reg, "max_count"), Some(42));
        assert_eq!(get_flag_int(&reg, "missing"), None);
    }

    #[test]
    fn test_get_flag_int_wrong_type() {
        let mut reg = new_feature_registry();
        register_flag(&mut reg, default_bool_flag("flag", true, "F"));
        assert_eq!(get_flag_int(&reg, "flag"), None);
    }

    #[test]
    fn test_feature_registry_to_json() {
        let mut reg = new_feature_registry();
        register_flag(&mut reg, default_bool_flag("a", true, "A flag"));
        let json = feature_registry_to_json(&reg);
        assert!(!json.is_empty());
        assert!(json.contains("flags"));
        assert!(json.contains("\"a\""));
    }

    #[test]
    fn test_remove_flag() {
        let mut reg = new_feature_registry();
        register_flag(&mut reg, default_bool_flag("temp", true, "Temporary"));
        assert_eq!(flag_count(&reg), 1);
        let removed = remove_flag(&mut reg, "temp");
        assert!(removed);
        assert_eq!(flag_count(&reg), 0);
        let not_removed = remove_flag(&mut reg, "temp");
        assert!(!not_removed);
    }

    #[test]
    fn test_flag_value_variants() {
        let mut reg = new_feature_registry();
        register_flag(
            &mut reg,
            FeatureFlag {
                name: "text_flag".to_string(),
                value: FlagValue::Text("hello".to_string()),
                description: "Text".to_string(),
                tags: Vec::new(),
                override_env: None,
            },
        );
        register_flag(
            &mut reg,
            FeatureFlag {
                name: "float_flag".to_string(),
                value: FlagValue::Float(2.78),
                description: "Float".to_string(),
                tags: Vec::new(),
                override_env: Some("MY_FLOAT_FLAG".to_string()),
            },
        );
        let json = feature_registry_to_json(&reg);
        assert!(json.contains("text_flag"));
        assert!(json.contains("float_flag"));
    }
}
