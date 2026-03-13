//! Named character configuration profile storing a set of morph parameter values.
//!
//! Profiles allow saving and restoring complete sets of morph parameters
//! and comparing differences between configurations.

#![allow(dead_code)]

use std::collections::HashMap;

/// Configuration for a `CharacterProfile`.
#[derive(Debug, Clone)]
pub struct CharacterProfileConfig {
    /// Maximum number of parameters stored in a profile.
    pub max_params: usize,
}

/// A single morph parameter entry inside a profile.
#[derive(Debug, Clone)]
pub struct ProfileEntry {
    /// Parameter name.
    pub name: String,
    /// Parameter value.
    pub value: f32,
}

/// A named character morph configuration profile.
#[derive(Debug, Clone)]
pub struct CharacterProfile {
    config: CharacterProfileConfig,
    /// Profile name.
    pub name: String,
    params: HashMap<String, f32>,
}

/// Build a default `CharacterProfileConfig`.
#[allow(dead_code)]
pub fn default_character_profile_config() -> CharacterProfileConfig {
    CharacterProfileConfig { max_params: 512 }
}

/// Create a new `CharacterProfile` with the given name.
#[allow(dead_code)]
pub fn new_character_profile(name: &str, config: CharacterProfileConfig) -> CharacterProfile {
    CharacterProfile {
        config,
        name: name.to_string(),
        params: HashMap::new(),
    }
}

/// Set a named parameter in the profile.
#[allow(dead_code)]
pub fn profile_set_param(profile: &mut CharacterProfile, name: &str, value: f32) {
    if profile.params.len() >= profile.config.max_params && !profile.params.contains_key(name) {
        return;
    }
    profile.params.insert(name.to_string(), value);
}

/// Get a named parameter from the profile. Returns `None` if not found.
#[allow(dead_code)]
pub fn profile_get_param(profile: &CharacterProfile, name: &str) -> Option<f32> {
    profile.params.get(name).copied()
}

/// Return the number of parameters stored.
#[allow(dead_code)]
pub fn profile_param_count(profile: &CharacterProfile) -> usize {
    profile.params.len()
}

/// Apply this profile's parameters to a target profile (overwrite matching keys).
#[allow(dead_code)]
pub fn profile_apply_to(source: &CharacterProfile, target: &mut CharacterProfile) {
    for (k, &v) in &source.params {
        if target.params.len() < target.config.max_params || target.params.contains_key(k) {
            target.params.insert(k.clone(), v);
        }
    }
}

/// Return a list of parameters that differ between two profiles.
#[allow(dead_code)]
pub fn profile_diff(a: &CharacterProfile, b: &CharacterProfile) -> Vec<ProfileEntry> {
    let mut diffs = Vec::new();
    // Parameters in a that differ or are absent in b
    for (k, &va) in &a.params {
        match b.params.get(k) {
            Some(&vb) if (va - vb).abs() > f32::EPSILON => {
                diffs.push(ProfileEntry { name: k.clone(), value: va - vb });
            }
            None => {
                diffs.push(ProfileEntry { name: k.clone(), value: va });
            }
            _ => {}
        }
    }
    // Parameters only in b
    for (k, &vb) in &b.params {
        if !a.params.contains_key(k) {
            diffs.push(ProfileEntry { name: k.clone(), value: -vb });
        }
    }
    diffs
}

/// Serialize the profile to a JSON string.
#[allow(dead_code)]
pub fn profile_to_json(profile: &CharacterProfile) -> String {
    let mut entries: Vec<String> = profile
        .params
        .iter()
        .map(|(k, v)| format!("\"{}\":{}", k, v))
        .collect();
    entries.sort();
    format!(
        "{{\"name\":\"{}\",\"param_count\":{},\"params\":{{{}}}}}",
        profile.name,
        profile.params.len(),
        entries.join(",")
    )
}

/// Remove all parameters from the profile.
#[allow(dead_code)]
pub fn profile_clear(profile: &mut CharacterProfile) {
    profile.params.clear();
}

/// Return true if the profile contains the named parameter.
#[allow(dead_code)]
pub fn profile_has_param(profile: &CharacterProfile, name: &str) -> bool {
    profile.params.contains_key(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_profile(name: &str) -> CharacterProfile {
        new_character_profile(name, default_character_profile_config())
    }

    #[test]
    fn test_set_and_get_param() {
        let mut p = make_profile("char1");
        profile_set_param(&mut p, "jaw_open", 0.5);
        assert!((profile_get_param(&p, "jaw_open").expect("should succeed") - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_get_missing_param() {
        let p = make_profile("empty");
        assert!(profile_get_param(&p, "nonexistent").is_none());
    }

    #[test]
    fn test_param_count() {
        let mut p = make_profile("c");
        assert_eq!(profile_param_count(&p), 0);
        profile_set_param(&mut p, "a", 1.0);
        profile_set_param(&mut p, "b", 2.0);
        assert_eq!(profile_param_count(&p), 2);
    }

    #[test]
    fn test_has_param() {
        let mut p = make_profile("c");
        profile_set_param(&mut p, "smile", 0.8);
        assert!(profile_has_param(&p, "smile"));
        assert!(!profile_has_param(&p, "frown"));
    }

    #[test]
    fn test_apply_to() {
        let mut src = make_profile("src");
        profile_set_param(&mut src, "smile", 1.0);
        let mut dst = make_profile("dst");
        profile_set_param(&mut dst, "smile", 0.0);
        profile_apply_to(&src, &mut dst);
        assert!((profile_get_param(&dst, "smile").expect("should succeed") - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_diff_returns_differences() {
        let mut a = make_profile("a");
        let mut b = make_profile("b");
        profile_set_param(&mut a, "x", 1.0);
        profile_set_param(&mut b, "x", 0.5);
        let diffs = profile_diff(&a, &b);
        assert!(!diffs.is_empty());
    }

    #[test]
    fn test_diff_identical_profiles_empty() {
        let mut a = make_profile("a");
        let mut b = make_profile("b");
        profile_set_param(&mut a, "x", 1.0);
        profile_set_param(&mut b, "x", 1.0);
        let diffs = profile_diff(&a, &b);
        assert!(diffs.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut p = make_profile("c");
        profile_set_param(&mut p, "a", 1.0);
        profile_set_param(&mut p, "b", 2.0);
        profile_clear(&mut p);
        assert_eq!(profile_param_count(&p), 0);
    }

    #[test]
    fn test_to_json_contains_name() {
        let p = make_profile("hero");
        let json = profile_to_json(&p);
        assert!(json.contains("\"name\":\"hero\""));
    }

    #[test]
    fn test_max_params_enforced() {
        let cfg = CharacterProfileConfig { max_params: 2 };
        let mut p = new_character_profile("limited", cfg);
        profile_set_param(&mut p, "a", 1.0);
        profile_set_param(&mut p, "b", 2.0);
        profile_set_param(&mut p, "c", 3.0); // should be ignored
        assert_eq!(profile_param_count(&p), 2);
    }
}
