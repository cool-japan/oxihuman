//! Asset versioning system — tracks semantic version numbers and compatibility checks.

use std::collections::HashMap;

/// A semantic version with major, minor, and patch components.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

/// The result of a compatibility check between two asset versions.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VersionCompatibility {
    /// Versions are fully compatible (same major.minor or patch-only diff).
    Compatible,
    /// Available is older but can still be used with older assets (same major).
    BackwardsCompatible,
    /// Major version mismatch — incompatible.
    Incompatible,
}

/// A registry mapping asset names to their versions.
#[allow(dead_code)]
pub struct AssetVersionRegistry {
    pub entries: HashMap<String, AssetVersion>,
}

/// Creates a new `AssetVersion`.
#[allow(dead_code)]
pub fn new_asset_version(major: u32, minor: u32, patch: u32) -> AssetVersion {
    AssetVersion { major, minor, patch }
}

/// Formats an `AssetVersion` as `"major.minor.patch"`.
#[allow(dead_code)]
pub fn asset_version_to_string(v: &AssetVersion) -> String {
    format!("{}.{}.{}", v.major, v.minor, v.patch)
}

/// Parses a `"major.minor.patch"` string into an `AssetVersion`, or `None` on failure.
#[allow(dead_code)]
pub fn parse_asset_version(s: &str) -> Option<AssetVersion> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts[1].parse::<u32>().ok()?;
    let patch = parts[2].parse::<u32>().ok()?;
    Some(AssetVersion { major, minor, patch })
}

/// Checks compatibility: `required` is what the asset needs, `available` is what we have.
/// - Same major + available minor >= required minor → Compatible
/// - Same major + available minor < required minor → BackwardsCompatible
/// - Different major → Incompatible
#[allow(dead_code)]
pub fn check_compatibility(
    required: &AssetVersion,
    available: &AssetVersion,
) -> VersionCompatibility {
    if required.major != available.major {
        return VersionCompatibility::Incompatible;
    }
    if available.minor >= required.minor {
        VersionCompatibility::Compatible
    } else {
        VersionCompatibility::BackwardsCompatible
    }
}

/// Creates a new empty `AssetVersionRegistry`.
#[allow(dead_code)]
pub fn new_version_registry() -> AssetVersionRegistry {
    AssetVersionRegistry {
        entries: HashMap::new(),
    }
}

/// Registers or updates the version for a named asset.
#[allow(dead_code)]
pub fn register_asset_version(
    registry: &mut AssetVersionRegistry,
    name: &str,
    version: AssetVersion,
) {
    registry.entries.insert(name.to_string(), version);
}

/// Returns the version for the named asset, or `None` if not registered.
#[allow(dead_code)]
pub fn get_asset_version<'a>(
    registry: &'a AssetVersionRegistry,
    name: &str,
) -> Option<&'a AssetVersion> {
    registry.entries.get(name)
}

/// Returns true if version `a` is strictly newer than version `b`.
#[allow(dead_code)]
pub fn version_is_newer(a: &AssetVersion, b: &AssetVersion) -> bool {
    if a.major != b.major {
        return a.major > b.major;
    }
    if a.minor != b.minor {
        return a.minor > b.minor;
    }
    a.patch > b.patch
}

/// Returns the human-readable name for a `VersionCompatibility` value.
#[allow(dead_code)]
pub fn version_compatibility_name(compat: VersionCompatibility) -> &'static str {
    match compat {
        VersionCompatibility::Compatible => "Compatible",
        VersionCompatibility::BackwardsCompatible => "BackwardsCompatible",
        VersionCompatibility::Incompatible => "Incompatible",
    }
}

/// Returns the total number of assets registered in the registry.
#[allow(dead_code)]
pub fn registry_asset_count(registry: &AssetVersionRegistry) -> usize {
    registry.entries.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_asset_version() {
        let v = new_asset_version(1, 2, 3);
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_asset_version_to_string() {
        let v = new_asset_version(2, 0, 14);
        assert_eq!(asset_version_to_string(&v), "2.0.14");
    }

    #[test]
    fn test_parse_asset_version_valid() {
        let v = parse_asset_version("1.3.7").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 3);
        assert_eq!(v.patch, 7);
    }

    #[test]
    fn test_parse_asset_version_invalid() {
        assert!(parse_asset_version("1.2").is_none());
        assert!(parse_asset_version("abc").is_none());
        assert!(parse_asset_version("1.x.3").is_none());
        assert!(parse_asset_version("").is_none());
    }

    #[test]
    fn test_check_compatibility_compatible() {
        let required = new_asset_version(1, 2, 0);
        let available = new_asset_version(1, 3, 0);
        assert_eq!(
            check_compatibility(&required, &available),
            VersionCompatibility::Compatible
        );
    }

    #[test]
    fn test_check_compatibility_backwards() {
        let required = new_asset_version(1, 5, 0);
        let available = new_asset_version(1, 3, 0);
        assert_eq!(
            check_compatibility(&required, &available),
            VersionCompatibility::BackwardsCompatible
        );
    }

    #[test]
    fn test_check_compatibility_incompatible() {
        let required = new_asset_version(2, 0, 0);
        let available = new_asset_version(1, 9, 9);
        assert_eq!(
            check_compatibility(&required, &available),
            VersionCompatibility::Incompatible
        );
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut reg = new_version_registry();
        register_asset_version(&mut reg, "human_base", new_asset_version(1, 0, 0));
        let v = get_asset_version(&reg, "human_base").unwrap();
        assert_eq!(v.major, 1);
    }

    #[test]
    fn test_registry_get_missing() {
        let reg = new_version_registry();
        assert!(get_asset_version(&reg, "nope").is_none());
    }

    #[test]
    fn test_version_is_newer() {
        let a = new_asset_version(2, 0, 0);
        let b = new_asset_version(1, 9, 9);
        assert!(version_is_newer(&a, &b));
        assert!(!version_is_newer(&b, &a));

        let c = new_asset_version(1, 5, 0);
        let d = new_asset_version(1, 3, 0);
        assert!(version_is_newer(&c, &d));

        let e = new_asset_version(1, 0, 5);
        let f = new_asset_version(1, 0, 3);
        assert!(version_is_newer(&e, &f));

        // Equal versions
        assert!(!version_is_newer(&e, &e));
    }

    #[test]
    fn test_version_compatibility_name() {
        assert_eq!(
            version_compatibility_name(VersionCompatibility::Compatible),
            "Compatible"
        );
        assert_eq!(
            version_compatibility_name(VersionCompatibility::BackwardsCompatible),
            "BackwardsCompatible"
        );
        assert_eq!(
            version_compatibility_name(VersionCompatibility::Incompatible),
            "Incompatible"
        );
    }

    #[test]
    fn test_registry_asset_count() {
        let mut reg = new_version_registry();
        assert_eq!(registry_asset_count(&reg), 0);
        register_asset_version(&mut reg, "a", new_asset_version(1, 0, 0));
        register_asset_version(&mut reg, "b", new_asset_version(2, 0, 0));
        assert_eq!(registry_asset_count(&reg), 2);
        // Overwrite existing key
        register_asset_version(&mut reg, "a", new_asset_version(1, 1, 0));
        assert_eq!(registry_asset_count(&reg), 2);
    }

    #[test]
    fn test_parse_roundtrip() {
        let v = new_asset_version(3, 12, 99);
        let s = asset_version_to_string(&v);
        let parsed = parse_asset_version(&s).unwrap();
        assert_eq!(parsed, v);
    }
}
