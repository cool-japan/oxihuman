// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Plugin isolation sandbox with capability-based permission system.
//!
//! Plugins run in a [`PluginSandbox`] that limits which system resources they
//! may access.  A [`SandboxPolicy`] aggregates a set of allowed
//! [`PluginCapability`] values and can be applied to any sandbox.

#![allow(dead_code)]

use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Individual capabilities a plugin may be granted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum PluginCapability {
    /// Read access to host file system.
    FileRead,
    /// Write access to host file system.
    FileWrite,
    /// Network (TCP/UDP) access.
    Network,
    /// Arbitrary host-memory access.
    Memory,
    /// CPU-intensive background compute.
    Compute,
}

impl PluginCapability {
    /// Return the canonical string name for serialisation.
    pub fn as_str(self) -> &'static str {
        match self {
            PluginCapability::FileRead => "FileRead",
            PluginCapability::FileWrite => "FileWrite",
            PluginCapability::Network => "Network",
            PluginCapability::Memory => "Memory",
            PluginCapability::Compute => "Compute",
        }
    }

    /// All defined capabilities in a fixed order.
    pub fn all() -> &'static [PluginCapability] {
        &[
            PluginCapability::FileRead,
            PluginCapability::FileWrite,
            PluginCapability::Network,
            PluginCapability::Memory,
            PluginCapability::Compute,
        ]
    }
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// A policy describing which capabilities are permitted.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SandboxPolicy {
    /// Human-readable policy name.
    pub name: String,
    /// Allowed capabilities.
    allowed: HashSet<PluginCapability>,
}

/// Per-plugin sandbox instance tracking granted permissions.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PluginSandbox {
    /// Plugin identifier.
    pub plugin_id: String,
    /// Currently granted capabilities.
    granted: HashSet<PluginCapability>,
    /// Whether the sandbox is in restricted mode (deny-all override).
    restricted: bool,
}

// ---------------------------------------------------------------------------
// SandboxPolicy helpers
// ---------------------------------------------------------------------------

/// Create a new empty [`SandboxPolicy`] with the given name.
pub fn create_policy(name: &str) -> SandboxPolicy {
    SandboxPolicy {
        name: name.to_string(),
        allowed: HashSet::new(),
    }
}

/// Add a capability to a policy, returning the modified policy.
pub fn policy_allow(mut policy: SandboxPolicy, cap: PluginCapability) -> SandboxPolicy {
    policy.allowed.insert(cap);
    policy
}

/// Check whether a policy grants a specific capability.
pub fn policy_allows(policy: &SandboxPolicy, cap: PluginCapability) -> bool {
    policy.allowed.contains(&cap)
}

/// Merge two policies, returning a new policy with the union of allowed
/// capabilities.  The name is taken from `a`.
pub fn merge_policies(a: &SandboxPolicy, b: &SandboxPolicy) -> SandboxPolicy {
    let allowed = a.allowed.union(&b.allowed).copied().collect();
    SandboxPolicy {
        name: a.name.clone(),
        allowed,
    }
}

/// Serialise a policy to a simple JSON-like string.
pub fn policy_to_json(policy: &SandboxPolicy) -> String {
    let mut caps: Vec<&str> = policy.allowed.iter().map(|c| c.as_str()).collect();
    caps.sort_unstable();
    format!(
        r#"{{"name":"{}","allowed":[{}]}}"#,
        policy.name,
        caps.iter()
            .map(|s| format!(r#""{}""#, s))
            .collect::<Vec<_>>()
            .join(",")
    )
}

// ---------------------------------------------------------------------------
// PluginSandbox construction / mutation
// ---------------------------------------------------------------------------

/// Create a new empty (no capabilities) [`PluginSandbox`] for `plugin_id`.
pub fn new_plugin_sandbox(plugin_id: &str) -> PluginSandbox {
    PluginSandbox {
        plugin_id: plugin_id.to_string(),
        granted: HashSet::new(),
        restricted: false,
    }
}

/// Grant a single capability to the sandbox.
pub fn grant_capability(sandbox: &mut PluginSandbox, cap: PluginCapability) {
    sandbox.granted.insert(cap);
}

/// Revoke a single capability from the sandbox.
pub fn revoke_capability(sandbox: &mut PluginSandbox, cap: PluginCapability) {
    sandbox.granted.remove(&cap);
}

/// Check whether a sandbox currently holds a capability.
///
/// Returns `false` if the sandbox is in restricted mode, regardless of grants.
pub fn has_capability(sandbox: &PluginSandbox, cap: PluginCapability) -> bool {
    if sandbox.restricted {
        return false;
    }
    sandbox.granted.contains(&cap)
}

/// Apply a [`SandboxPolicy`] to the sandbox, replacing all granted capabilities.
pub fn apply_policy(sandbox: &mut PluginSandbox, policy: &SandboxPolicy) {
    sandbox.granted = policy.allowed.clone();
}

/// Serialise sandbox state to a JSON-like string.
pub fn sandbox_to_json(sandbox: &PluginSandbox) -> String {
    let mut caps: Vec<&str> = sandbox.granted.iter().map(|c| c.as_str()).collect();
    caps.sort_unstable();
    format!(
        r#"{{"plugin_id":"{}","restricted":{},"granted":[{}]}}"#,
        sandbox.plugin_id,
        sandbox.restricted,
        caps.iter()
            .map(|s| format!(r#""{}""#, s))
            .collect::<Vec<_>>()
            .join(",")
    )
}

/// Return a sorted list of capability names currently granted.
pub fn list_capabilities(sandbox: &PluginSandbox) -> Vec<&'static str> {
    let mut caps: Vec<&'static str> = sandbox.granted.iter().map(|c| c.as_str()).collect();
    caps.sort_unstable();
    caps
}

/// Return the number of currently granted capabilities.
pub fn capability_count(sandbox: &PluginSandbox) -> usize {
    sandbox.granted.len()
}

/// Remove all granted capabilities, keeping plugin identity.
pub fn reset_sandbox(sandbox: &mut PluginSandbox) {
    sandbox.granted.clear();
    sandbox.restricted = false;
}

/// Enable restricted mode – all capability checks return `false`.
pub fn is_restricted(sandbox: &PluginSandbox) -> bool {
    sandbox.restricted
}

/// Toggle the restricted flag on or off.
pub fn set_restricted(sandbox: &mut PluginSandbox, restricted: bool) {
    sandbox.restricted = restricted;
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // 1
    #[test]
    fn new_sandbox_has_no_capabilities() {
        let sb = new_plugin_sandbox("test");
        assert_eq!(capability_count(&sb), 0);
    }

    // 2
    #[test]
    fn grant_and_has_capability() {
        let mut sb = new_plugin_sandbox("p1");
        grant_capability(&mut sb, PluginCapability::FileRead);
        assert!(has_capability(&sb, PluginCapability::FileRead));
    }

    // 3
    #[test]
    fn revoke_removes_capability() {
        let mut sb = new_plugin_sandbox("p2");
        grant_capability(&mut sb, PluginCapability::Network);
        revoke_capability(&mut sb, PluginCapability::Network);
        assert!(!has_capability(&sb, PluginCapability::Network));
    }

    // 4
    #[test]
    fn has_capability_restricted_mode_returns_false() {
        let mut sb = new_plugin_sandbox("p3");
        grant_capability(&mut sb, PluginCapability::Compute);
        set_restricted(&mut sb, true);
        assert!(!has_capability(&sb, PluginCapability::Compute));
    }

    // 5
    #[test]
    fn unrestrict_restores_access() {
        let mut sb = new_plugin_sandbox("p4");
        grant_capability(&mut sb, PluginCapability::Memory);
        set_restricted(&mut sb, true);
        set_restricted(&mut sb, false);
        assert!(has_capability(&sb, PluginCapability::Memory));
    }

    // 6
    #[test]
    fn create_policy_is_empty() {
        let p = create_policy("empty");
        assert!(!policy_allows(&p, PluginCapability::FileRead));
    }

    // 7
    #[test]
    fn policy_allow_adds_capability() {
        let p = policy_allow(create_policy("r"), PluginCapability::FileRead);
        assert!(policy_allows(&p, PluginCapability::FileRead));
        assert!(!policy_allows(&p, PluginCapability::FileWrite));
    }

    // 8
    #[test]
    fn apply_policy_replaces_grants() {
        let mut sb = new_plugin_sandbox("p5");
        grant_capability(&mut sb, PluginCapability::Network);
        let pol = policy_allow(create_policy("safe"), PluginCapability::FileRead);
        apply_policy(&mut sb, &pol);
        assert!(!has_capability(&sb, PluginCapability::Network));
        assert!(has_capability(&sb, PluginCapability::FileRead));
    }

    // 9
    #[test]
    fn sandbox_to_json_contains_plugin_id() {
        let sb = new_plugin_sandbox("myplugin");
        let json = sandbox_to_json(&sb);
        assert!(json.contains("myplugin"));
    }

    // 10
    #[test]
    fn sandbox_to_json_reflects_grants() {
        let mut sb = new_plugin_sandbox("p6");
        grant_capability(&mut sb, PluginCapability::Compute);
        let json = sandbox_to_json(&sb);
        assert!(json.contains("Compute"));
    }

    // 11
    #[test]
    fn list_capabilities_sorted() {
        let mut sb = new_plugin_sandbox("p7");
        grant_capability(&mut sb, PluginCapability::Network);
        grant_capability(&mut sb, PluginCapability::FileRead);
        let caps = list_capabilities(&sb);
        let sorted = {
            let mut c = caps.clone();
            c.sort_unstable();
            c
        };
        assert_eq!(caps, sorted);
    }

    // 12
    #[test]
    fn capability_count_accurate() {
        let mut sb = new_plugin_sandbox("p8");
        grant_capability(&mut sb, PluginCapability::FileRead);
        grant_capability(&mut sb, PluginCapability::FileWrite);
        assert_eq!(capability_count(&sb), 2);
    }

    // 13
    #[test]
    fn reset_sandbox_clears_all() {
        let mut sb = new_plugin_sandbox("p9");
        grant_capability(&mut sb, PluginCapability::Memory);
        set_restricted(&mut sb, true);
        reset_sandbox(&mut sb);
        assert_eq!(capability_count(&sb), 0);
        assert!(!is_restricted(&sb));
    }

    // 14
    #[test]
    fn merge_policies_union() {
        let pa = policy_allow(create_policy("a"), PluginCapability::FileRead);
        let pb = policy_allow(create_policy("b"), PluginCapability::Network);
        let merged = merge_policies(&pa, &pb);
        assert!(policy_allows(&merged, PluginCapability::FileRead));
        assert!(policy_allows(&merged, PluginCapability::Network));
    }

    // 15
    #[test]
    fn merge_policies_name_from_a() {
        let pa = create_policy("alpha");
        let pb = create_policy("beta");
        let merged = merge_policies(&pa, &pb);
        assert_eq!(merged.name, "alpha");
    }

    // 16
    #[test]
    fn policy_to_json_contains_name() {
        let p = create_policy("mypol");
        let json = policy_to_json(&p);
        assert!(json.contains("mypol"));
    }

    // 17
    #[test]
    fn capability_all_has_five_entries() {
        assert_eq!(PluginCapability::all().len(), 5);
    }

    // 18
    #[test]
    fn grant_duplicate_does_not_increase_count() {
        let mut sb = new_plugin_sandbox("dup");
        grant_capability(&mut sb, PluginCapability::Compute);
        grant_capability(&mut sb, PluginCapability::Compute);
        assert_eq!(capability_count(&sb), 1);
    }

    // 19
    #[test]
    fn revoke_nonexistent_is_safe() {
        let mut sb = new_plugin_sandbox("safe");
        revoke_capability(&mut sb, PluginCapability::Memory); // should not panic
        assert_eq!(capability_count(&sb), 0);
    }

    // 20
    #[test]
    fn all_capability_names_are_unique() {
        let names: HashSet<&str> = PluginCapability::all().iter().map(|c| c.as_str()).collect();
        assert_eq!(names.len(), PluginCapability::all().len());
    }
}
