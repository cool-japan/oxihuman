// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A/B testing configuration.

use std::collections::HashMap;

/// An A/B test variant definition.
#[derive(Debug, Clone)]
pub struct AbVariant {
    pub name: String,
    pub weight: f32,
}

/// A/B test configuration.
#[derive(Debug, Default)]
pub struct AbTestConfig {
    tests: HashMap<String, Vec<AbVariant>>,
}

impl AbTestConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_test(&mut self, test_name: &str, variants: Vec<AbVariant>) {
        self.tests.insert(test_name.to_string(), variants);
    }

    pub fn variant_count(&self, test_name: &str) -> usize {
        self.tests.get(test_name).map(|v| v.len()).unwrap_or(0)
    }

    pub fn total_weight(&self, test_name: &str) -> f32 {
        self.tests
            .get(test_name)
            .map(|v| v.iter().map(|vr| vr.weight).sum())
            .unwrap_or(0.0)
    }

    pub fn select_variant(&self, test_name: &str, seed: f32) -> Option<&str> {
        let variants = self.tests.get(test_name)?;
        let total = self.total_weight(test_name);
        if total <= 0.0 {
            return None;
        }
        let target = seed.rem_euclid(total);
        let mut cumulative = 0.0;
        for v in variants {
            cumulative += v.weight;
            if target < cumulative {
                return Some(&v.name);
            }
        }
        variants.last().map(|v| v.name.as_str())
    }

    pub fn test_count(&self) -> usize {
        self.tests.len()
    }

    pub fn test_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.tests.keys().cloned().collect();
        names.sort();
        names
    }
}

pub fn new_ab_test_config() -> AbTestConfig {
    AbTestConfig::new()
}

pub fn ab_add_test(cfg: &mut AbTestConfig, name: &str, variants: Vec<AbVariant>) {
    cfg.add_test(name, variants);
}

pub fn ab_select_variant<'a>(cfg: &'a AbTestConfig, test: &str, seed: f32) -> Option<&'a str> {
    cfg.select_variant(test, seed)
}

pub fn ab_variant_count(cfg: &AbTestConfig, test: &str) -> usize {
    cfg.variant_count(test)
}

pub fn ab_total_weight(cfg: &AbTestConfig, test: &str) -> f32 {
    cfg.total_weight(test)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_variants() -> Vec<AbVariant> {
        vec![
            AbVariant {
                name: "control".to_string(),
                weight: 50.0,
            },
            AbVariant {
                name: "treatment".to_string(),
                weight: 50.0,
            },
        ]
    }

    #[test]
    fn test_add_and_count() {
        let mut cfg = new_ab_test_config();
        ab_add_test(&mut cfg, "button_color", make_variants());
        assert_eq!(ab_variant_count(&cfg, "button_color"), 2);
    }

    #[test]
    fn test_total_weight() {
        let mut cfg = new_ab_test_config();
        ab_add_test(&mut cfg, "test1", make_variants());
        assert!((ab_total_weight(&cfg, "test1") - 100.0).abs() < 1e-5);
    }

    #[test]
    fn test_select_control() {
        /* seed=25 lands in first bucket (0..50) */
        let mut cfg = new_ab_test_config();
        ab_add_test(&mut cfg, "t", make_variants());
        let v = ab_select_variant(&cfg, "t", 25.0);
        assert_eq!(v, Some("control"));
    }

    #[test]
    fn test_select_treatment() {
        /* seed=75 lands in second bucket (50..100) */
        let mut cfg = new_ab_test_config();
        ab_add_test(&mut cfg, "t", make_variants());
        let v = ab_select_variant(&cfg, "t", 75.0);
        assert_eq!(v, Some("treatment"));
    }

    #[test]
    fn test_unknown_test_returns_none() {
        let cfg = new_ab_test_config();
        assert_eq!(ab_select_variant(&cfg, "missing", 0.5), None);
    }

    #[test]
    fn test_test_count() {
        let mut cfg = new_ab_test_config();
        ab_add_test(&mut cfg, "a", make_variants());
        ab_add_test(&mut cfg, "b", make_variants());
        assert_eq!(cfg.test_count(), 2);
    }

    #[test]
    fn test_test_names_sorted() {
        let mut cfg = new_ab_test_config();
        ab_add_test(&mut cfg, "z_test", make_variants());
        ab_add_test(&mut cfg, "a_test", make_variants());
        assert_eq!(cfg.test_names()[0], "a_test");
    }

    #[test]
    fn test_zero_weight_returns_none() {
        let mut cfg = new_ab_test_config();
        cfg.add_test(
            "empty_w",
            vec![AbVariant {
                name: "v".to_string(),
                weight: 0.0,
            }],
        );
        assert_eq!(ab_select_variant(&cfg, "empty_w", 0.5), None);
    }

    #[test]
    fn test_uneven_weights() {
        /* 90/10 split: seed=95 should land in treatment */
        let mut cfg = new_ab_test_config();
        cfg.add_test(
            "skewed",
            vec![
                AbVariant {
                    name: "ctrl".to_string(),
                    weight: 90.0,
                },
                AbVariant {
                    name: "treat".to_string(),
                    weight: 10.0,
                },
            ],
        );
        assert_eq!(ab_select_variant(&cfg, "skewed", 95.0), Some("treat"));
    }

    #[test]
    fn test_seed_wraps_via_rem_euclid() {
        /* seed=150 mod 100 = 50, should still land correctly */
        let mut cfg = new_ab_test_config();
        ab_add_test(&mut cfg, "t", make_variants());
        let v = ab_select_variant(&cfg, "t", 150.0);
        assert!(v.is_some());
    }
}
