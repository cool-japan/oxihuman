// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

// The engine-level unit / safety constants live alongside the policy layer.
// Declared here (rather than in a top-level `mod` line) so the constant is
// available to the policy API and re-exported as `policy::AGE_ADULT_FLOOR_YR`.
#[path = "units.rs"]
pub mod units;
pub use units::AGE_ADULT_FLOOR_YR;

/// Blocked tag substrings for any policy profile.
const BLOCKED_TAGS: &[&str] = &["explicit", "sexual", "nudity", "adult"];

/// Numeric parameter names that carry an age (in years) and are therefore
/// subject to the adult age floor. Matched case-insensitively.
const AGE_PARAM_NAMES: &[&str] = &["age", "age_years", "ageyears"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PolicyProfile {
    /// Strictest — allowlist only, block all unlisted targets.
    Strict,
    /// Standard — block known bad tags, allow everything else.
    Standard,
}

#[derive(Debug, Clone)]
pub struct Policy {
    pub profile: PolicyProfile,
    pub allowlist: Vec<String>,
}

impl Policy {
    pub fn new(profile: PolicyProfile) -> Self {
        Policy {
            profile,
            allowlist: Vec::new(),
        }
    }

    pub fn with_allowlist(profile: PolicyProfile, allowlist: Vec<String>) -> Self {
        Policy { profile, allowlist }
    }

    /// Returns true if a target with the given name and tags is permitted.
    pub fn is_target_allowed(&self, name: &str, tags: &[&str]) -> bool {
        // Always block explicit content tags
        for tag in tags {
            let tag_lower = tag.to_lowercase();
            for blocked in BLOCKED_TAGS {
                if tag_lower.contains(blocked) {
                    return false;
                }
            }
        }
        // Also check the target name itself
        let name_lower = name.to_lowercase();
        for blocked in BLOCKED_TAGS {
            if name_lower.contains(blocked) {
                return false;
            }
        }

        match self.profile {
            PolicyProfile::Standard => true,
            PolicyProfile::Strict => self.allowlist.iter().any(|a| a == name),
        }
    }

    /// Return the inclusive lower bound this policy enforces on the numeric
    /// parameter `name`, if any.
    ///
    /// Both [`PolicyProfile::Standard`] and [`PolicyProfile::Strict`] enforce
    /// the adult age floor ([`AGE_ADULT_FLOOR_YR`]) on age parameters
    /// (`"age"`, `"age_years"`; matched case-insensitively). All other
    /// parameters are unconstrained and return `None`.
    ///
    /// This is the engine/core-level guarantee that mirrors the shipped
    /// pack's `age_floor_years` metadata. Third-party embedders can query it
    /// to clamp or reject age inputs before building a mesh.
    pub fn param_floor(&self, name: &str) -> Option<f32> {
        let name_lower = name.to_lowercase();
        if AGE_PARAM_NAMES.iter().any(|n| *n == name_lower) {
            return Some(AGE_ADULT_FLOOR_YR);
        }
        None
    }

    /// Returns `true` when `value` is permitted for the numeric parameter
    /// `name` under this policy.
    ///
    /// A value is allowed when the parameter has no floor, or when it is at or
    /// above the floor returned by [`Policy::param_floor`]. Age values below
    /// [`AGE_ADULT_FLOOR_YR`] are rejected under every profile.
    pub fn is_param_value_allowed(&self, name: &str, value: f32) -> bool {
        match self.param_floor(name) {
            Some(floor) => value >= floor,
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_allows_normal_targets() {
        let p = Policy::new(PolicyProfile::Standard);
        assert!(p.is_target_allowed("height", &["body", "shape"]));
        assert!(p.is_target_allowed("weight", &[]));
    }

    #[test]
    fn blocks_explicit_tags() {
        let p = Policy::new(PolicyProfile::Standard);
        assert!(!p.is_target_allowed("test", &["explicit"]));
        assert!(!p.is_target_allowed("test", &["sexual-content"]));
        assert!(!p.is_target_allowed("explicit-body", &[]));
    }

    #[test]
    fn strict_blocks_unlisted() {
        let p = Policy::with_allowlist(
            PolicyProfile::Strict,
            vec!["height".to_string(), "weight".to_string()],
        );
        assert!(p.is_target_allowed("height", &[]));
        assert!(!p.is_target_allowed("muscle", &[]));
    }

    #[test]
    fn age_floor_enforced_on_both_profiles() {
        for profile in [PolicyProfile::Standard, PolicyProfile::Strict] {
            let p = Policy::new(profile);
            assert_eq!(p.param_floor("age"), Some(AGE_ADULT_FLOOR_YR));
            assert_eq!(p.param_floor("Age"), Some(AGE_ADULT_FLOOR_YR));
            assert_eq!(p.param_floor("age_years"), Some(AGE_ADULT_FLOOR_YR));
        }
    }

    #[test]
    fn non_age_params_have_no_floor() {
        let p = Policy::new(PolicyProfile::Standard);
        assert_eq!(p.param_floor("height"), None);
        assert_eq!(p.param_floor("weight"), None);
        assert!(p.is_param_value_allowed("height", -5.0));
    }

    #[test]
    fn age_below_floor_rejected() {
        let p = Policy::new(PolicyProfile::Strict);
        assert!(!p.is_param_value_allowed("age", 17.9));
        assert!(!p.is_param_value_allowed("age", 5.0));
        assert!(!p.is_param_value_allowed("age", 0.0));
    }

    #[test]
    fn age_at_or_above_floor_allowed() {
        let p = Policy::new(PolicyProfile::Standard);
        assert!(p.is_param_value_allowed("age", 18.0));
        assert!(p.is_param_value_allowed("age", 25.0));
        assert!(p.is_param_value_allowed("age", 90.0));
    }
}
