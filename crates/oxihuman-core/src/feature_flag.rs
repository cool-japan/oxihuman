// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Feature flag registry with toggle support.

use std::collections::HashMap;

/// State of a single feature flag.
#[derive(Debug, Clone, PartialEq)]
pub enum FlagState {
    Enabled,
    Disabled,
    Experimental,
}

/// A single feature flag entry.
#[derive(Debug, Clone)]
pub struct FeatureFlagEntry {
    pub name: String,
    pub state: FlagState,
    pub description: String,
}

/// Registry for feature flags.
#[derive(Debug, Default)]
pub struct FeatureFlagRegistry {
    flags: HashMap<String, FeatureFlagEntry>,
}

impl FeatureFlagRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, name: &str, enabled: bool, description: &str) {
        let state = if enabled {
            FlagState::Enabled
        } else {
            FlagState::Disabled
        };
        self.flags.insert(
            name.to_string(),
            FeatureFlagEntry {
                name: name.to_string(),
                state,
                description: description.to_string(),
            },
        );
    }

    pub fn toggle(&mut self, name: &str) -> bool {
        if let Some(entry) = self.flags.get_mut(name) {
            entry.state = match entry.state {
                FlagState::Enabled => FlagState::Disabled,
                FlagState::Disabled | FlagState::Experimental => FlagState::Enabled,
            };
            true
        } else {
            false
        }
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.flags
            .get(name)
            .map(|e| e.state == FlagState::Enabled)
            .unwrap_or(false)
    }

    pub fn flag_count(&self) -> usize {
        self.flags.len()
    }

    pub fn all_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.flags.keys().cloned().collect();
        names.sort();
        names
    }
}

pub fn new_flag_registry() -> FeatureFlagRegistry {
    FeatureFlagRegistry::new()
}

pub fn register_feature(registry: &mut FeatureFlagRegistry, name: &str, enabled: bool, desc: &str) {
    registry.register(name, enabled, desc);
}

pub fn toggle_feature(registry: &mut FeatureFlagRegistry, name: &str) -> bool {
    registry.toggle(name)
}

pub fn is_feature_enabled(registry: &FeatureFlagRegistry, name: &str) -> bool {
    registry.is_enabled(name)
}

pub fn feature_flag_count(registry: &FeatureFlagRegistry) -> usize {
    registry.flag_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_check() {
        let mut reg = new_flag_registry();
        register_feature(&mut reg, "dark_mode", true, "Dark theme");
        assert!(is_feature_enabled(&reg, "dark_mode"));
    }

    #[test]
    fn test_disabled_flag() {
        let mut reg = new_flag_registry();
        register_feature(&mut reg, "beta", false, "Beta feature");
        assert!(!is_feature_enabled(&reg, "beta"));
    }

    #[test]
    fn test_toggle_on_to_off() {
        let mut reg = new_flag_registry();
        register_feature(&mut reg, "foo", true, "Foo");
        toggle_feature(&mut reg, "foo");
        assert!(!is_feature_enabled(&reg, "foo"));
    }

    #[test]
    fn test_toggle_off_to_on() {
        let mut reg = new_flag_registry();
        register_feature(&mut reg, "bar", false, "Bar");
        toggle_feature(&mut reg, "bar");
        assert!(is_feature_enabled(&reg, "bar"));
    }

    #[test]
    fn test_toggle_unknown_returns_false() {
        let mut reg = new_flag_registry();
        assert!(!toggle_feature(&mut reg, "nonexistent"));
    }

    #[test]
    fn test_flag_count() {
        let mut reg = new_flag_registry();
        register_feature(&mut reg, "a", true, "A");
        register_feature(&mut reg, "b", false, "B");
        assert_eq!(feature_flag_count(&reg), 2);
    }

    #[test]
    fn test_all_names_sorted() {
        let mut reg = new_flag_registry();
        register_feature(&mut reg, "zeta", true, "Z");
        register_feature(&mut reg, "alpha", true, "A");
        let names = reg.all_names();
        assert_eq!(names[0], "alpha");
    }

    #[test]
    fn test_unknown_flag_disabled() {
        let reg = new_flag_registry();
        assert!(!is_feature_enabled(&reg, "missing"));
    }

    #[test]
    fn test_experimental_state() {
        /* experimental flags toggle to enabled */
        let mut reg = new_flag_registry();
        reg.flags.insert(
            "exp".to_string(),
            FeatureFlagEntry {
                name: "exp".to_string(),
                state: FlagState::Experimental,
                description: "Experimental".to_string(),
            },
        );
        toggle_feature(&mut reg, "exp");
        assert!(is_feature_enabled(&reg, "exp"));
    }

    #[test]
    fn test_overwrite_flag() {
        /* registering same name twice overwrites */
        let mut reg = new_flag_registry();
        register_feature(&mut reg, "x", false, "off");
        register_feature(&mut reg, "x", true, "on");
        assert!(is_feature_enabled(&reg, "x"));
    }
}
