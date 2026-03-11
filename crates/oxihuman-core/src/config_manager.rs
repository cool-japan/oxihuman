//! Configuration manager with profiles and layered overrides.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A typed configuration value.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum ConfigValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

/// A named configuration profile containing a map of key→value entries.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ConfigProfile {
    /// Name of this profile.
    pub name: String,
    /// Key → value store.
    pub values: HashMap<String, ConfigValue>,
    /// Whether this profile has unsaved changes.
    pub dirty: bool,
}

/// Type alias for looking up a value across profiles.
#[allow(dead_code)]
pub type ConfigLookup<'a> = Option<&'a ConfigValue>;

/// Top-level configuration manager holding multiple named profiles.
#[allow(dead_code)]
pub struct ConfigManager {
    /// Ordered list of profiles.
    pub profiles: Vec<ConfigProfile>,
    /// Name of the currently active profile.
    pub active: String,
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Create a new configuration manager with one default profile named "default".
#[allow(dead_code)]
pub fn new_config_manager() -> ConfigManager {
    let default_profile = ConfigProfile {
        name: "default".to_string(),
        values: HashMap::new(),
        dirty: false,
    };
    ConfigManager {
        profiles: vec![default_profile],
        active: "default".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Profile management
// ---------------------------------------------------------------------------

/// Create a new empty profile with the given name.
/// Returns `false` if a profile with that name already exists.
#[allow(dead_code)]
pub fn create_profile(mgr: &mut ConfigManager, name: &str) -> bool {
    if mgr.profiles.iter().any(|p| p.name == name) {
        return false;
    }
    mgr.profiles.push(ConfigProfile {
        name: name.to_string(),
        values: HashMap::new(),
        dirty: false,
    });
    true
}

/// Delete a profile by name.
/// Returns `false` if not found or it is the active profile.
#[allow(dead_code)]
pub fn delete_profile(mgr: &mut ConfigManager, name: &str) -> bool {
    if mgr.active == name {
        return false;
    }
    if let Some(pos) = mgr.profiles.iter().position(|p| p.name == name) {
        mgr.profiles.remove(pos);
        return true;
    }
    false
}

/// Return the name of the currently active profile.
#[allow(dead_code)]
pub fn active_profile(mgr: &ConfigManager) -> &str {
    &mgr.active
}

/// Switch the active profile to `name`.
/// Returns `false` if no such profile exists.
#[allow(dead_code)]
pub fn switch_profile(mgr: &mut ConfigManager, name: &str) -> bool {
    if mgr.profiles.iter().any(|p| p.name == name) {
        mgr.active = name.to_string();
        return true;
    }
    false
}

/// Return the total number of profiles.
#[allow(dead_code)]
pub fn profile_count(mgr: &ConfigManager) -> usize {
    mgr.profiles.len()
}

/// Return a list of all profile names.
#[allow(dead_code)]
pub fn list_profiles(mgr: &ConfigManager) -> Vec<&str> {
    mgr.profiles.iter().map(|p| p.name.as_str()).collect()
}

// ---------------------------------------------------------------------------
// Value access
// ---------------------------------------------------------------------------

/// Set a key-value pair in the named profile.
/// Returns `false` if the profile does not exist.
#[allow(dead_code)]
pub fn set_profile_value(
    mgr: &mut ConfigManager,
    profile: &str,
    key: &str,
    val: ConfigValue,
) -> bool {
    if let Some(p) = mgr.profiles.iter_mut().find(|p| p.name == profile) {
        p.values.insert(key.to_string(), val);
        p.dirty = true;
        return true;
    }
    false
}

/// Get a value from the named profile by key.
#[allow(dead_code)]
pub fn get_profile_value<'a>(mgr: &'a ConfigManager, profile: &str, key: &str) -> ConfigLookup<'a> {
    mgr.profiles
        .iter()
        .find(|p| p.name == profile)
        .and_then(|p| p.values.get(key))
}

/// Get a value from the active profile, falling back to the "default" profile
/// if the key is not present in the active profile.
#[allow(dead_code)]
pub fn get_value_with_fallback<'a>(mgr: &'a ConfigManager, key: &str) -> ConfigLookup<'a> {
    let active = mgr.active.clone();
    if let Some(v) = get_profile_value(mgr, &active, key) {
        return Some(v);
    }
    get_profile_value(mgr, "default", key)
}

// ---------------------------------------------------------------------------
// Merge and reset
// ---------------------------------------------------------------------------

/// Merge all key-value pairs from `src` profile into `dst` profile.
/// Existing keys in `dst` are overwritten.
/// Returns `false` if either profile does not exist.
#[allow(dead_code)]
pub fn merge_profiles(mgr: &mut ConfigManager, src: &str, dst: &str) -> bool {
    // Collect src values first to avoid borrow issues.
    let src_values: Option<Vec<(String, ConfigValue)>> =
        mgr.profiles.iter().find(|p| p.name == src).map(|p| {
            p.values
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        });

    if let Some(pairs) = src_values {
        if let Some(dst_profile) = mgr.profiles.iter_mut().find(|p| p.name == dst) {
            for (k, v) in pairs {
                dst_profile.values.insert(k, v);
            }
            dst_profile.dirty = true;
            return true;
        }
    }
    false
}

/// Reset a profile to empty (remove all key-value pairs).
/// Returns `false` if the profile does not exist.
#[allow(dead_code)]
pub fn reset_profile_to_defaults(mgr: &mut ConfigManager, profile: &str) -> bool {
    if let Some(p) = mgr.profiles.iter_mut().find(|p| p.name == profile) {
        p.values.clear();
        p.dirty = true;
        return true;
    }
    false
}

// ---------------------------------------------------------------------------
// Bulk import
// ---------------------------------------------------------------------------

/// Populate the named profile from a slice of `(key, value)` string pairs.
/// Values starting with `true`/`false` become `Bool`, digits become `Int`,
/// digits with `.` become `Float`, otherwise `Str`.
/// Returns the number of pairs imported, or `None` if the profile was not found.
#[allow(dead_code)]
pub fn config_from_pairs(
    mgr: &mut ConfigManager,
    profile: &str,
    pairs: &[(&str, &str)],
) -> Option<usize> {
    let profile_exists = mgr.profiles.iter().any(|p| p.name == profile);
    if !profile_exists {
        return None;
    }
    let mut count = 0;
    for (k, v) in pairs {
        let val = parse_config_value(v);
        set_profile_value(mgr, profile, k, val);
        count += 1;
    }
    Some(count)
}

fn parse_config_value(s: &str) -> ConfigValue {
    if s == "true" {
        return ConfigValue::Bool(true);
    }
    if s == "false" {
        return ConfigValue::Bool(false);
    }
    if s.contains('.') {
        if let Ok(f) = s.parse::<f64>() {
            return ConfigValue::Float(f);
        }
    }
    if let Ok(i) = s.parse::<i64>() {
        return ConfigValue::Int(i);
    }
    ConfigValue::Str(s.to_string())
}

// ---------------------------------------------------------------------------
// JSON serialisation (minimal, no external deps)
// ---------------------------------------------------------------------------

fn config_value_to_json(v: &ConfigValue) -> String {
    match v {
        ConfigValue::Bool(b) => b.to_string(),
        ConfigValue::Int(i) => i.to_string(),
        ConfigValue::Float(f) => format!("{f}"),
        ConfigValue::Str(s) => format!(r#""{s}""#),
    }
}

/// Serialise the entire configuration manager state to a compact JSON string.
#[allow(dead_code)]
pub fn config_to_json(mgr: &ConfigManager) -> String {
    let profiles_json: Vec<String> = mgr
        .profiles
        .iter()
        .map(|p| {
            let entries: Vec<String> = p
                .values
                .iter()
                .map(|(k, v)| format!(r#""{k}":{}"#, config_value_to_json(v)))
                .collect();
            format!(
                r#"{{"name":"{}","dirty":{},"values":{{{}}}}}"#,
                p.name,
                p.dirty,
                entries.join(",")
            )
        })
        .collect();

    format!(
        r#"{{"active":"{}","profiles":[{}]}}"#,
        mgr.active,
        profiles_json.join(",")
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_config_manager_has_default_profile() {
        let mgr = new_config_manager();
        assert_eq!(profile_count(&mgr), 1);
        assert_eq!(active_profile(&mgr), "default");
    }

    #[test]
    fn test_create_profile_succeeds() {
        let mut mgr = new_config_manager();
        assert!(create_profile(&mut mgr, "production"));
        assert_eq!(profile_count(&mgr), 2);
    }

    #[test]
    fn test_create_profile_duplicate_fails() {
        let mut mgr = new_config_manager();
        create_profile(&mut mgr, "dev");
        assert!(!create_profile(&mut mgr, "dev"));
        assert_eq!(profile_count(&mgr), 2);
    }

    #[test]
    fn test_delete_profile_removes_it() {
        let mut mgr = new_config_manager();
        create_profile(&mut mgr, "temp");
        assert!(delete_profile(&mut mgr, "temp"));
        assert_eq!(profile_count(&mgr), 1);
    }

    #[test]
    fn test_delete_active_profile_fails() {
        let mut mgr = new_config_manager();
        assert!(!delete_profile(&mut mgr, "default"));
    }

    #[test]
    fn test_switch_profile() {
        let mut mgr = new_config_manager();
        create_profile(&mut mgr, "alt");
        assert!(switch_profile(&mut mgr, "alt"));
        assert_eq!(active_profile(&mgr), "alt");
    }

    #[test]
    fn test_switch_profile_unknown_fails() {
        let mut mgr = new_config_manager();
        assert!(!switch_profile(&mut mgr, "ghost"));
    }

    #[test]
    fn test_set_and_get_profile_value_bool() {
        let mut mgr = new_config_manager();
        set_profile_value(&mut mgr, "default", "show_grid", ConfigValue::Bool(true));
        let v = get_profile_value(&mgr, "default", "show_grid");
        assert_eq!(v, Some(&ConfigValue::Bool(true)));
    }

    #[test]
    fn test_set_and_get_profile_value_int() {
        let mut mgr = new_config_manager();
        set_profile_value(&mut mgr, "default", "max_fps", ConfigValue::Int(60));
        let v = get_profile_value(&mgr, "default", "max_fps");
        assert_eq!(v, Some(&ConfigValue::Int(60)));
    }

    #[test]
    fn test_set_and_get_profile_value_float() {
        let mut mgr = new_config_manager();
        set_profile_value(&mut mgr, "default", "gamma", ConfigValue::Float(2.2));
        let v = get_profile_value(&mgr, "default", "gamma");
        assert_eq!(v, Some(&ConfigValue::Float(2.2)));
    }

    #[test]
    fn test_set_and_get_profile_value_str() {
        let mut mgr = new_config_manager();
        set_profile_value(
            &mut mgr,
            "default",
            "lang",
            ConfigValue::Str("en".to_string()),
        );
        let v = get_profile_value(&mgr, "default", "lang");
        assert_eq!(v, Some(&ConfigValue::Str("en".to_string())));
    }

    #[test]
    fn test_get_value_with_fallback_uses_active() {
        let mut mgr = new_config_manager();
        create_profile(&mut mgr, "custom");
        switch_profile(&mut mgr, "custom");
        set_profile_value(&mut mgr, "custom", "key", ConfigValue::Int(99));
        let v = get_value_with_fallback(&mgr, "key");
        assert_eq!(v, Some(&ConfigValue::Int(99)));
    }

    #[test]
    fn test_get_value_with_fallback_falls_to_default() {
        let mut mgr = new_config_manager();
        set_profile_value(
            &mut mgr,
            "default",
            "fallback_key",
            ConfigValue::Bool(false),
        );
        create_profile(&mut mgr, "p2");
        switch_profile(&mut mgr, "p2");
        let v = get_value_with_fallback(&mgr, "fallback_key");
        assert_eq!(v, Some(&ConfigValue::Bool(false)));
    }

    #[test]
    fn test_merge_profiles() {
        let mut mgr = new_config_manager();
        create_profile(&mut mgr, "src");
        create_profile(&mut mgr, "dst");
        set_profile_value(&mut mgr, "src", "x", ConfigValue::Int(1));
        assert!(merge_profiles(&mut mgr, "src", "dst"));
        assert_eq!(
            get_profile_value(&mgr, "dst", "x"),
            Some(&ConfigValue::Int(1))
        );
    }

    #[test]
    fn test_reset_profile_to_defaults() {
        let mut mgr = new_config_manager();
        set_profile_value(&mut mgr, "default", "k", ConfigValue::Int(5));
        assert!(reset_profile_to_defaults(&mut mgr, "default"));
        assert!(get_profile_value(&mgr, "default", "k").is_none());
    }

    #[test]
    fn test_config_from_pairs_parses_types() {
        let mut mgr = new_config_manager();
        let pairs = vec![
            ("flag", "true"),
            ("count", "10"),
            ("ratio", "0.5"),
            ("name", "hello"),
        ];
        let n = config_from_pairs(&mut mgr, "default", &pairs);
        assert_eq!(n, Some(4));
        assert_eq!(
            get_profile_value(&mgr, "default", "flag"),
            Some(&ConfigValue::Bool(true))
        );
        assert_eq!(
            get_profile_value(&mgr, "default", "count"),
            Some(&ConfigValue::Int(10))
        );
    }

    #[test]
    fn test_list_profiles() {
        let mut mgr = new_config_manager();
        create_profile(&mut mgr, "p1");
        create_profile(&mut mgr, "p2");
        let names = list_profiles(&mgr);
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"default"));
        assert!(names.contains(&"p1"));
    }

    #[test]
    fn test_config_to_json_contains_active() {
        let mgr = new_config_manager();
        let json = config_to_json(&mgr);
        assert!(json.contains("\"active\":\"default\""));
    }

    #[test]
    fn test_config_from_pairs_unknown_profile_returns_none() {
        let mut mgr = new_config_manager();
        let result = config_from_pairs(&mut mgr, "nonexistent", &[("k", "v")]);
        assert!(result.is_none());
    }
}
