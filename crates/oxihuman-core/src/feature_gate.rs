// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

pub struct FeatureGate {
    pub enabled: HashMap<String, bool>,
}

impl FeatureGate {
    pub fn new() -> Self {
        FeatureGate {
            enabled: HashMap::new(),
        }
    }
}

impl Default for FeatureGate {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_feature_gate() -> FeatureGate {
    FeatureGate::new()
}

pub fn gate_enable(g: &mut FeatureGate, feature: &str) {
    g.enabled.insert(feature.to_string(), true);
}

pub fn gate_disable(g: &mut FeatureGate, feature: &str) {
    g.enabled.insert(feature.to_string(), false);
}

pub fn gate_is_enabled(g: &FeatureGate, feature: &str) -> bool {
    *g.enabled.get(feature).unwrap_or(&false)
}

pub fn gate_toggle(g: &mut FeatureGate, feature: &str) {
    let current = *g.enabled.get(feature).unwrap_or(&false);
    g.enabled.insert(feature.to_string(), !current);
}

pub fn gate_feature_count(g: &FeatureGate) -> usize {
    g.enabled.len()
}

pub fn gate_clear(g: &mut FeatureGate) {
    g.enabled.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        /* new gate has no features */
        let g = new_feature_gate();
        assert_eq!(gate_feature_count(&g), 0);
    }

    #[test]
    fn test_enable_and_check() {
        /* enable then check returns true */
        let mut g = new_feature_gate();
        gate_enable(&mut g, "dark_mode");
        assert!(gate_is_enabled(&g, "dark_mode"));
    }

    #[test]
    fn test_disable() {
        /* disable sets feature to false */
        let mut g = new_feature_gate();
        gate_enable(&mut g, "beta");
        gate_disable(&mut g, "beta");
        assert!(!gate_is_enabled(&g, "beta"));
    }

    #[test]
    fn test_unknown_feature_disabled() {
        /* unknown feature is disabled by default */
        let g = new_feature_gate();
        assert!(!gate_is_enabled(&g, "unknown"));
    }

    #[test]
    fn test_toggle() {
        /* toggle flips state */
        let mut g = new_feature_gate();
        gate_toggle(&mut g, "x");
        assert!(gate_is_enabled(&g, "x"));
        gate_toggle(&mut g, "x");
        assert!(!gate_is_enabled(&g, "x"));
    }

    #[test]
    fn test_feature_count() {
        /* count includes all registered features */
        let mut g = new_feature_gate();
        gate_enable(&mut g, "a");
        gate_disable(&mut g, "b");
        assert_eq!(gate_feature_count(&g), 2);
    }

    #[test]
    fn test_clear() {
        /* clear removes all features */
        let mut g = new_feature_gate();
        gate_enable(&mut g, "x");
        gate_clear(&mut g);
        assert_eq!(gate_feature_count(&g), 0);
    }
}
