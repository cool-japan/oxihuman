// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use crate::params::ParamState;
use std::collections::HashMap;

/// A single entry in a blend profile: target name + weight.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlendEntry {
    pub target_name: String,
    pub weight: f32, // [0.0, 1.0]
}

impl BlendEntry {
    pub fn new(target_name: impl Into<String>, weight: f32) -> Self {
        Self {
            target_name: target_name.into(),
            weight,
        }
    }
}

/// A named collection of morph target weights.
/// Multiple profiles can be blended together.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlendProfile {
    pub name: String,
    pub description: String,
    pub entries: Vec<BlendEntry>,
    /// Optional associated ParamState for this profile.
    pub params: Option<ParamState>,
}

impl BlendProfile {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            entries: Vec::new(),
            params: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_params(mut self, p: ParamState) -> Self {
        self.params = Some(p);
        self
    }

    pub fn add_entry(&mut self, target_name: impl Into<String>, weight: f32) {
        self.entries.push(BlendEntry::new(target_name, weight));
    }

    /// Remove the first entry with the given target name. Returns true if found.
    pub fn remove_entry(&mut self, target_name: &str) -> bool {
        if let Some(pos) = self
            .entries
            .iter()
            .position(|e| e.target_name == target_name)
        {
            self.entries.remove(pos);
            true
        } else {
            false
        }
    }

    /// Returns 0.0 if the target is not present.
    pub fn get_weight(&self, target_name: &str) -> f32 {
        self.entries
            .iter()
            .find(|e| e.target_name == target_name)
            .map(|e| e.weight)
            .unwrap_or(0.0)
    }

    pub fn set_weight(&mut self, target_name: &str, weight: f32) {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|e| e.target_name == target_name)
        {
            entry.weight = weight;
        }
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Normalize all weights so they sum to 1.0. No-op if sum is zero.
    pub fn normalize(&mut self) {
        let sum: f32 = self.entries.iter().map(|e| e.weight).sum();
        if sum > 0.0 {
            for entry in &mut self.entries {
                entry.weight /= sum;
            }
        }
    }

    /// Scale all weights by a factor.
    pub fn scale(&mut self, factor: f32) {
        for entry in &mut self.entries {
            entry.weight *= factor;
        }
    }

    /// Clamp all weights to [0.0, 1.0].
    pub fn clamp_weights(&mut self) {
        for entry in &mut self.entries {
            entry.weight = entry.weight.clamp(0.0, 1.0);
        }
    }

    /// Merge another profile into this one (sum weights, clamp to 1.0).
    pub fn merge(&mut self, other: &BlendProfile) {
        for other_entry in &other.entries {
            if let Some(entry) = self
                .entries
                .iter_mut()
                .find(|e| e.target_name == other_entry.target_name)
            {
                entry.weight = (entry.weight + other_entry.weight).min(1.0);
            } else {
                self.entries.push(other_entry.clone());
            }
        }
    }

    /// Return a new profile that is the linear blend of self and other at t.
    /// At t=0.0 the result equals self; at t=1.0 it equals other.
    pub fn lerp(&self, other: &BlendProfile, t: f32) -> BlendProfile {
        let t = t.clamp(0.0, 1.0);
        let mut result = BlendProfile::new(format!("{}_lerp_{}", self.name, other.name));

        // Collect all target names from both profiles.
        let mut targets: Vec<&str> = self.affected_targets();
        for name in other.affected_targets() {
            if !targets.contains(&name) {
                targets.push(name);
            }
        }

        for target in targets {
            let w_self = self.get_weight(target);
            let w_other = other.get_weight(target);
            let w = w_self * (1.0 - t) + w_other * t;
            result.add_entry(target, w);
        }

        result
    }

    /// Collect unique target names affected by this profile.
    pub fn affected_targets(&self) -> Vec<&str> {
        let mut seen: Vec<&str> = Vec::new();
        for entry in &self.entries {
            if !seen.contains(&entry.target_name.as_str()) {
                seen.push(&entry.target_name);
            }
        }
        seen
    }

    /// Convert to a HashMap<String, f32> for easy lookup.
    pub fn to_weight_map(&self) -> HashMap<String, f32> {
        self.entries
            .iter()
            .map(|e| (e.target_name.clone(), e.weight))
            .collect()
    }
}

/// A library of named blend profiles.
#[allow(dead_code)]
pub struct BlendProfileLibrary {
    profiles: HashMap<String, BlendProfile>,
}

impl BlendProfileLibrary {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
        }
    }

    pub fn add(&mut self, profile: BlendProfile) {
        self.profiles.insert(profile.name.clone(), profile);
    }

    pub fn get(&self, name: &str) -> Option<&BlendProfile> {
        self.profiles.get(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<BlendProfile> {
        self.profiles.remove(name)
    }

    pub fn names(&self) -> Vec<&str> {
        self.profiles.keys().map(|k| k.as_str()).collect()
    }

    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }

    /// Find profiles that affect a given target name.
    pub fn profiles_for_target(&self, target_name: &str) -> Vec<&BlendProfile> {
        self.profiles
            .values()
            .filter(|p| p.entries.iter().any(|e| e.target_name == target_name))
            .collect()
    }
}

impl Default for BlendProfileLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blend_entry_new_fields() {
        let e = BlendEntry::new("smile", 0.8);
        assert_eq!(e.target_name, "smile");
        assert!((e.weight - 0.8).abs() < 1e-6);
    }

    #[test]
    fn blend_profile_new_empty() {
        let p = BlendProfile::new("base");
        assert_eq!(p.name, "base");
        assert_eq!(p.description, "");
        assert!(p.entries.is_empty());
        assert!(p.params.is_none());
        assert!(p.is_empty());
        assert_eq!(p.entry_count(), 0);
    }

    #[test]
    fn add_and_get_weight() {
        let mut p = BlendProfile::new("test");
        p.add_entry("brow_raise", 0.6);
        assert!((p.get_weight("brow_raise") - 0.6).abs() < 1e-6);
        assert!((p.get_weight("nonexistent") - 0.0).abs() < 1e-6);
    }

    #[test]
    fn remove_entry_returns_true() {
        let mut p = BlendProfile::new("test");
        p.add_entry("smile", 0.5);
        let removed = p.remove_entry("smile");
        assert!(removed);
        assert!(p.is_empty());
    }

    #[test]
    fn remove_nonexistent_returns_false() {
        let mut p = BlendProfile::new("test");
        let removed = p.remove_entry("ghost");
        assert!(!removed);
    }

    #[test]
    fn set_weight_updates() {
        let mut p = BlendProfile::new("test");
        p.add_entry("eye_blink", 0.2);
        p.set_weight("eye_blink", 0.9);
        assert!((p.get_weight("eye_blink") - 0.9).abs() < 1e-6);
    }

    #[test]
    fn normalize_sums_to_one() {
        let mut p = BlendProfile::new("test");
        p.add_entry("a", 1.0);
        p.add_entry("b", 3.0);
        p.normalize();
        let sum: f32 = p.entries.iter().map(|e| e.weight).sum();
        assert!((sum - 1.0).abs() < 1e-5);
        assert!((p.get_weight("a") - 0.25).abs() < 1e-5);
        assert!((p.get_weight("b") - 0.75).abs() < 1e-5);
    }

    #[test]
    fn scale_multiplies_weights() {
        let mut p = BlendProfile::new("test");
        p.add_entry("x", 0.4);
        p.add_entry("y", 0.6);
        p.scale(0.5);
        assert!((p.get_weight("x") - 0.2).abs() < 1e-6);
        assert!((p.get_weight("y") - 0.3).abs() < 1e-6);
    }

    #[test]
    fn clamp_weights_caps_at_one() {
        let mut p = BlendProfile::new("test");
        p.add_entry("over", 1.5);
        p.add_entry("under", -0.3);
        p.clamp_weights();
        assert!((p.get_weight("over") - 1.0).abs() < 1e-6);
        assert!((p.get_weight("under") - 0.0).abs() < 1e-6);
    }

    #[test]
    fn merge_sums_weights() {
        let mut a = BlendProfile::new("a");
        a.add_entry("smile", 0.4);
        a.add_entry("unique_a", 0.3);

        let mut b = BlendProfile::new("b");
        b.add_entry("smile", 0.7);
        b.add_entry("unique_b", 0.5);

        a.merge(&b);
        // smile: 0.4 + 0.7 = 1.1, clamped to 1.0
        assert!((a.get_weight("smile") - 1.0).abs() < 1e-6);
        // unique_a preserved
        assert!((a.get_weight("unique_a") - 0.3).abs() < 1e-6);
        // unique_b added from b
        assert!((a.get_weight("unique_b") - 0.5).abs() < 1e-6);
    }

    #[test]
    fn lerp_at_zero_equals_self() {
        let mut a = BlendProfile::new("a");
        a.add_entry("smile", 0.3);

        let mut b = BlendProfile::new("b");
        b.add_entry("smile", 0.9);

        let result = a.lerp(&b, 0.0);
        assert!((result.get_weight("smile") - 0.3).abs() < 1e-5);
    }

    #[test]
    fn lerp_at_one_equals_other() {
        let mut a = BlendProfile::new("a");
        a.add_entry("smile", 0.3);

        let mut b = BlendProfile::new("b");
        b.add_entry("smile", 0.9);

        let result = a.lerp(&b, 1.0);
        assert!((result.get_weight("smile") - 0.9).abs() < 1e-5);
    }

    #[test]
    fn affected_targets_unique() {
        let mut p = BlendProfile::new("test");
        p.add_entry("eye", 0.5);
        p.add_entry("mouth", 0.3);
        p.add_entry("eye", 0.2); // duplicate
        let targets = p.affected_targets();
        // "eye" should appear only once
        let eye_count = targets.iter().filter(|&&t| t == "eye").count();
        assert_eq!(eye_count, 1);
        assert!(targets.contains(&"mouth"));
    }

    #[test]
    fn to_weight_map_correct() {
        let mut p = BlendProfile::new("test");
        p.add_entry("brow", 0.4);
        p.add_entry("cheek", 0.7);
        let map = p.to_weight_map();
        assert!((map["brow"] - 0.4).abs() < 1e-6);
        assert!((map["cheek"] - 0.7).abs() < 1e-6);
    }

    #[test]
    fn library_add_and_get() {
        let mut lib = BlendProfileLibrary::new();
        assert!(lib.is_empty());
        let mut profile = BlendProfile::new("happy");
        profile.add_entry("smile", 0.8);
        lib.add(profile);
        assert_eq!(lib.len(), 1);
        let retrieved = lib.get("happy").expect("should succeed");
        assert_eq!(retrieved.name, "happy");
        assert!((retrieved.get_weight("smile") - 0.8).abs() < 1e-6);
        let removed = lib.remove("happy");
        assert!(removed.is_some());
        assert!(lib.is_empty());
    }

    #[test]
    fn library_profiles_for_target() {
        let mut lib = BlendProfileLibrary::new();

        let mut p1 = BlendProfile::new("happy");
        p1.add_entry("smile", 0.9);
        p1.add_entry("brow_raise", 0.5);
        lib.add(p1);

        let mut p2 = BlendProfile::new("sad");
        p2.add_entry("frown", 0.8);
        lib.add(p2);

        let mut p3 = BlendProfile::new("excited");
        p3.add_entry("smile", 0.6);
        lib.add(p3);

        let affecting_smile = lib.profiles_for_target("smile");
        assert_eq!(affecting_smile.len(), 2);

        let affecting_frown = lib.profiles_for_target("frown");
        assert_eq!(affecting_frown.len(), 1);

        let affecting_none = lib.profiles_for_target("nonexistent");
        assert!(affecting_none.is_empty());
    }

    #[test]
    fn with_description_and_params() {
        let p = BlendProfile::new("preset")
            .with_description("A test preset")
            .with_params(ParamState::default());
        assert_eq!(p.description, "A test preset");
        assert!(p.params.is_some());
    }

    #[test]
    fn library_names() {
        let mut lib = BlendProfileLibrary::default();
        lib.add(BlendProfile::new("alpha"));
        lib.add(BlendProfile::new("beta"));
        let mut names = lib.names();
        names.sort();
        assert_eq!(names, vec!["alpha", "beta"]);
    }
}
