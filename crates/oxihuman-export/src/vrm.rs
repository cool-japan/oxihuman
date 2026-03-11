// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! VRM 1.0 avatar format — GLTF extension JSON builder.
//!
//! Produces the `VRMC_vrm` extension JSON nodes that are embedded into a
//! GLTF file to describe a VRM 1.0 avatar.

// ── types ─────────────────────────────────────────────────────────────────────

/// Who may use this avatar as their own avatar.
#[allow(dead_code)]
pub enum AvatarPermission {
    OnlyAuthor,
    OnlySeparatelyLicensedPerson,
    Everyone,
}

/// Commercial usage rights.
#[allow(dead_code)]
pub enum CommercialUsage {
    PersonalNonProfit,
    PersonalProfit,
    Corporation,
}

/// VRM 1.0 avatar metadata (VRMC_vrm `meta` block).
#[allow(dead_code)]
pub struct VrmMeta {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub license_url: String,
    pub avatar_permission: AvatarPermission,
    pub commercial_usage: CommercialUsage,
}

/// Humanoid bone node indices (into the GLTF nodes array).
#[allow(dead_code)]
pub struct VrmHumanoid {
    pub hips_node: u32,
    pub head_node: u32,
    pub left_hand_node: Option<u32>,
    pub right_hand_node: Option<u32>,
}

/// Top-level options for a VRM export.
#[allow(dead_code)]
pub struct VrmExportOptions {
    pub meta: VrmMeta,
    pub humanoid: VrmHumanoid,
    /// Spec version string, e.g. `"1.0"`.
    pub spec_version: String,
}

// ── string helpers ────────────────────────────────────────────────────────────

/// Map an [`AvatarPermission`] to its VRM 1.0 spec string.
#[allow(dead_code)]
pub fn avatar_permission_str(p: &AvatarPermission) -> &'static str {
    match p {
        AvatarPermission::OnlyAuthor => "onlyAuthor",
        AvatarPermission::OnlySeparatelyLicensedPerson => "onlySeparatelyLicensedPerson",
        AvatarPermission::Everyone => "everyone",
    }
}

/// Map a [`CommercialUsage`] to its VRM 1.0 spec string.
#[allow(dead_code)]
pub fn commercial_usage_str(c: &CommercialUsage) -> &'static str {
    match c {
        CommercialUsage::PersonalNonProfit => "personalNonProfit",
        CommercialUsage::PersonalProfit => "personalProfit",
        CommercialUsage::Corporation => "corporation",
    }
}

// ── JSON builders ─────────────────────────────────────────────────────────────

/// Serialise [`VrmMeta`] to a JSON object string.
#[allow(dead_code)]
pub fn vrm_meta_to_json(meta: &VrmMeta) -> String {
    let authors_json: String = meta
        .authors
        .iter()
        .map(|a| format!("\"{}\"", a.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(",");

    format!(
        r#"{{"name":"{name}","version":"{ver}","authors":[{authors}],"licenseUrl":"{lic}","avatarPermission":"{ap}","commercialUsage":"{cu}"}}"#,
        name = meta.name.replace('"', "\\\""),
        ver = meta.version.replace('"', "\\\""),
        authors = authors_json,
        lic = meta.license_url.replace('"', "\\\""),
        ap = avatar_permission_str(&meta.avatar_permission),
        cu = commercial_usage_str(&meta.commercial_usage),
    )
}

/// Serialise [`VrmHumanoid`] to a JSON object string.
#[allow(dead_code)]
pub fn vrm_humanoid_to_json(h: &VrmHumanoid) -> String {
    let mut bones = format!(
        r#"{{"hips":{{"node":{}}},"head":{{"node":{}}}"#,
        h.hips_node, h.head_node
    );
    if let Some(lh) = h.left_hand_node {
        bones.push_str(&format!(r#","leftHand":{{"node":{}}}"#, lh));
    }
    if let Some(rh) = h.right_hand_node {
        bones.push_str(&format!(r#","rightHand":{{"node":{}}}"#, rh));
    }
    bones.push('}');

    format!(r#"{{"humanBones":{}}}"#, bones)
}

/// Build the full `VRMC_vrm` extension JSON string.
///
/// The returned string is suitable for embedding under
/// `extensions.VRMC_vrm` in a GLTF JSON file.
#[allow(dead_code)]
pub fn build_vrm_extensions_json(opts: &VrmExportOptions) -> String {
    format!(
        r#"{{"VRMC_vrm":{{"specVersion":"{sv}","meta":{meta},"humanoid":{hum}}}}}"#,
        sv = opts.spec_version.replace('"', "\\\""),
        meta = vrm_meta_to_json(&opts.meta),
        hum = vrm_humanoid_to_json(&opts.humanoid),
    )
}

// ── convenience ───────────────────────────────────────────────────────────────

/// Create a [`VrmMeta`] with sane CC-BY defaults.
#[allow(dead_code)]
pub fn default_vrm_meta(name: &str) -> VrmMeta {
    VrmMeta {
        name: name.to_string(),
        version: "1.0".to_string(),
        authors: vec!["OxiHuman".to_string()],
        license_url: "https://creativecommons.org/licenses/by/4.0/".to_string(),
        avatar_permission: AvatarPermission::Everyone,
        commercial_usage: CommercialUsage::PersonalProfit,
    }
}

/// Validate export options.  Returns `Err` if:
/// - `meta.name` is empty
/// - `meta.authors` is empty
/// - `hips_node == head_node` (they must differ)
#[allow(dead_code)]
pub fn validate_vrm_options(opts: &VrmExportOptions) -> Result<(), String> {
    if opts.meta.name.trim().is_empty() {
        return Err("VRM meta name must not be empty".to_string());
    }
    if opts.meta.authors.is_empty() {
        return Err("VRM meta must have at least one author".to_string());
    }
    if opts.humanoid.hips_node == opts.humanoid.head_node {
        return Err(format!(
            "hips_node and head_node must differ (both are {})",
            opts.humanoid.hips_node
        ));
    }
    Ok(())
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_opts() -> VrmExportOptions {
        VrmExportOptions {
            meta: default_vrm_meta("TestAvatar"),
            humanoid: VrmHumanoid {
                hips_node: 0,
                head_node: 5,
                left_hand_node: Some(10),
                right_hand_node: Some(11),
            },
            spec_version: "1.0".to_string(),
        }
    }

    // 1. build_vrm_extensions_json contains "VRMC_vrm"
    #[test]
    fn extensions_json_contains_vrmc_vrm() {
        let json = build_vrm_extensions_json(&default_opts());
        assert!(json.contains("VRMC_vrm"));
    }

    // 2. build_vrm_extensions_json contains specVersion
    #[test]
    fn extensions_json_has_spec_version() {
        let json = build_vrm_extensions_json(&default_opts());
        assert!(json.contains("specVersion"));
        assert!(json.contains("1.0"));
    }

    // 3. vrm_meta_to_json contains the name
    #[test]
    fn meta_json_has_name() {
        let meta = default_vrm_meta("MyHero");
        let json = vrm_meta_to_json(&meta);
        assert!(json.contains("MyHero"));
        assert!(json.contains("name"));
    }

    // 4. vrm_meta_to_json contains licenseUrl
    #[test]
    fn meta_json_has_license_url() {
        let meta = default_vrm_meta("X");
        let json = vrm_meta_to_json(&meta);
        assert!(json.contains("licenseUrl"));
        assert!(json.contains("creativecommons.org"));
    }

    // 5. vrm_humanoid_to_json contains hips node index
    #[test]
    fn humanoid_json_has_hips() {
        let h = VrmHumanoid {
            hips_node: 42,
            head_node: 7,
            left_hand_node: None,
            right_hand_node: None,
        };
        let json = vrm_humanoid_to_json(&h);
        assert!(json.contains("hips"));
        assert!(json.contains("42"));
    }

    // 6. vrm_humanoid_to_json contains head node index
    #[test]
    fn humanoid_json_has_head() {
        let h = VrmHumanoid {
            hips_node: 0,
            head_node: 99,
            left_hand_node: None,
            right_hand_node: None,
        };
        let json = vrm_humanoid_to_json(&h);
        assert!(json.contains("head"));
        assert!(json.contains("99"));
    }

    // 7. avatar_permission_str values
    #[test]
    fn avatar_permission_str_values() {
        assert_eq!(
            avatar_permission_str(&AvatarPermission::OnlyAuthor),
            "onlyAuthor"
        );
        assert_eq!(
            avatar_permission_str(&AvatarPermission::OnlySeparatelyLicensedPerson),
            "onlySeparatelyLicensedPerson"
        );
        assert_eq!(
            avatar_permission_str(&AvatarPermission::Everyone),
            "everyone"
        );
    }

    // 8. commercial_usage_str values
    #[test]
    fn commercial_usage_str_values() {
        assert_eq!(
            commercial_usage_str(&CommercialUsage::PersonalNonProfit),
            "personalNonProfit"
        );
        assert_eq!(
            commercial_usage_str(&CommercialUsage::PersonalProfit),
            "personalProfit"
        );
        assert_eq!(
            commercial_usage_str(&CommercialUsage::Corporation),
            "corporation"
        );
    }

    // 9. validate_vrm_options rejects empty name
    #[test]
    fn validate_rejects_empty_name() {
        let mut opts = default_opts();
        opts.meta.name = "  ".to_string();
        assert!(validate_vrm_options(&opts).is_err());
    }

    // 10. validate_vrm_options rejects no authors
    #[test]
    fn validate_rejects_empty_authors() {
        let mut opts = default_opts();
        opts.meta.authors.clear();
        assert!(validate_vrm_options(&opts).is_err());
    }

    // 11. validate_vrm_options rejects hips == head
    #[test]
    fn validate_rejects_same_hips_and_head() {
        let mut opts = default_opts();
        opts.humanoid.hips_node = 5;
        opts.humanoid.head_node = 5;
        assert!(validate_vrm_options(&opts).is_err());
    }

    // 12. validate_vrm_options passes valid options
    #[test]
    fn validate_passes_valid() {
        assert!(validate_vrm_options(&default_opts()).is_ok());
    }

    // 13. default_vrm_meta sets CC-BY license
    #[test]
    fn default_meta_cc_by_license() {
        let meta = default_vrm_meta("Test");
        assert!(meta.license_url.contains("creativecommons.org"));
    }

    // 14. optional hand nodes — None does not appear in JSON
    #[test]
    fn optional_hand_nodes_none() {
        let h = VrmHumanoid {
            hips_node: 0,
            head_node: 1,
            left_hand_node: None,
            right_hand_node: None,
        };
        let json = vrm_humanoid_to_json(&h);
        assert!(!json.contains("leftHand"));
        assert!(!json.contains("rightHand"));
    }

    // 15. optional hand nodes — Some appears in JSON
    #[test]
    fn optional_hand_nodes_some() {
        let h = VrmHumanoid {
            hips_node: 0,
            head_node: 1,
            left_hand_node: Some(20),
            right_hand_node: Some(21),
        };
        let json = vrm_humanoid_to_json(&h);
        assert!(json.contains("leftHand"));
        assert!(json.contains("20"));
        assert!(json.contains("rightHand"));
        assert!(json.contains("21"));
    }
}
