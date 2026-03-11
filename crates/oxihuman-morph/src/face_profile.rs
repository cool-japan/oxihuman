// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// A named face profile containing parameter key-value pairs.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceProfile {
    pub name: String,
    pub params: HashMap<String, f32>,
}

/// Create a new empty face profile.
#[allow(dead_code)]
pub fn new_face_profile(name: &str) -> FaceProfile {
    FaceProfile {
        name: name.to_string(),
        params: HashMap::new(),
    }
}

/// Set a parameter value on the profile.
#[allow(dead_code)]
pub fn profile_set_param(profile: &mut FaceProfile, key: &str, value: f32) {
    profile.params.insert(key.to_string(), value);
}

/// Get a parameter value from the profile.
#[allow(dead_code)]
pub fn profile_get_param(profile: &FaceProfile, key: &str) -> Option<f32> {
    profile.params.get(key).copied()
}

/// Return the number of parameters.
#[allow(dead_code)]
pub fn profile_param_count(profile: &FaceProfile) -> usize {
    profile.params.len()
}

/// Serialize the profile to a JSON string.
#[allow(dead_code)]
pub fn profile_to_json(profile: &FaceProfile) -> String {
    let mut entries: Vec<String> = profile
        .params
        .iter()
        .map(|(k, v)| format!("\"{}\":{:.4}", k, v))
        .collect();
    entries.sort();
    format!("{{\"name\":\"{}\",\"params\":{{{}}}}}", profile.name, entries.join(","))
}

/// Blend two profiles by a factor t (0 = a, 1 = b).
#[allow(dead_code)]
pub fn profile_blend(a: &FaceProfile, b: &FaceProfile, t: f32) -> FaceProfile {
    let t = t.clamp(0.0, 1.0);
    let mut result = new_face_profile(&a.name);
    for (k, va) in &a.params {
        let vb = b.params.get(k).copied().unwrap_or(0.0);
        result.params.insert(k.clone(), va + (vb - va) * t);
    }
    for (k, vb) in &b.params {
        if !a.params.contains_key(k) {
            result.params.insert(k.clone(), vb * t);
        }
    }
    result
}

/// Remove all parameters from the profile.
#[allow(dead_code)]
pub fn profile_clear(profile: &mut FaceProfile) {
    profile.params.clear();
}

/// Return the profile name.
#[allow(dead_code)]
pub fn profile_name_fp(profile: &FaceProfile) -> &str {
    &profile.name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_profile_empty() {
        let p = new_face_profile("base");
        assert_eq!(profile_param_count(&p), 0);
    }

    #[test]
    fn set_and_get_param() {
        let mut p = new_face_profile("p");
        profile_set_param(&mut p, "jaw_width", 0.5);
        assert!((profile_get_param(&p, "jaw_width").unwrap() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn get_missing_param() {
        let p = new_face_profile("p");
        assert!(profile_get_param(&p, "nope").is_none());
    }

    #[test]
    fn param_count() {
        let mut p = new_face_profile("p");
        profile_set_param(&mut p, "a", 1.0);
        profile_set_param(&mut p, "b", 2.0);
        assert_eq!(profile_param_count(&p), 2);
    }

    #[test]
    fn blend_half() {
        let mut a = new_face_profile("a");
        profile_set_param(&mut a, "x", 0.0);
        let mut b = new_face_profile("b");
        profile_set_param(&mut b, "x", 1.0);
        let c = profile_blend(&a, &b, 0.5);
        assert!((profile_get_param(&c, "x").unwrap() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn clear_params() {
        let mut p = new_face_profile("p");
        profile_set_param(&mut p, "x", 1.0);
        profile_clear(&mut p);
        assert_eq!(profile_param_count(&p), 0);
    }

    #[test]
    fn name_accessor() {
        let p = new_face_profile("myface");
        assert_eq!(profile_name_fp(&p), "myface");
    }

    #[test]
    fn to_json() {
        let mut p = new_face_profile("test");
        profile_set_param(&mut p, "x", 0.5);
        let j = profile_to_json(&p);
        assert!(j.contains("\"name\":\"test\""));
    }

    #[test]
    fn blend_at_zero() {
        let mut a = new_face_profile("a");
        profile_set_param(&mut a, "x", 0.2);
        let mut b = new_face_profile("b");
        profile_set_param(&mut b, "x", 0.8);
        let c = profile_blend(&a, &b, 0.0);
        assert!((profile_get_param(&c, "x").unwrap() - 0.2).abs() < 1e-6);
    }

    #[test]
    fn blend_at_one() {
        let mut a = new_face_profile("a");
        profile_set_param(&mut a, "x", 0.2);
        let mut b = new_face_profile("b");
        profile_set_param(&mut b, "x", 0.8);
        let c = profile_blend(&a, &b, 1.0);
        assert!((profile_get_param(&c, "x").unwrap() - 0.8).abs() < 1e-6);
    }
}
