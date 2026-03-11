// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde::{Deserialize, Serialize};

/// Blocked tag substrings for any policy profile.
const BLOCKED_TAGS: &[&str] = &["explicit", "sexual", "nudity", "adult"];

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
}
