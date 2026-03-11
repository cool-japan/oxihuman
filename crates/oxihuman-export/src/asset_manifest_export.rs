#![allow(dead_code)]
//! Export asset manifests.

/// Asset manifest export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AssetManifestExport {
    pub version: String,
    pub assets: Vec<String>,
    pub checksum: u32,
}

/// Export an asset manifest.
#[allow(dead_code)]
pub fn export_asset_manifest(version: &str, assets: &[&str]) -> AssetManifestExport {
    let asset_list: Vec<String> = assets.iter().map(|s| s.to_string()).collect();
    let checksum = asset_list.iter().fold(0u32, |acc, s| acc.wrapping_add(s.len() as u32));
    AssetManifestExport {
        version: version.to_string(),
        assets: asset_list,
        checksum,
    }
}

/// Return asset count.
#[allow(dead_code)]
pub fn manifest_asset_count(exp: &AssetManifestExport) -> usize {
    exp.assets.len()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn manifest_to_json(exp: &AssetManifestExport) -> String {
    let files: Vec<String> = exp.assets.iter().map(|a| format!("\"{}\"", a)).collect();
    format!(
        "{{\"version\":\"{}\",\"assets\":[{}],\"checksum\":{}}}",
        exp.version,
        files.join(","),
        exp.checksum
    )
}

/// Return version string.
#[allow(dead_code)]
pub fn manifest_version(exp: &AssetManifestExport) -> &str {
    &exp.version
}

/// Return checksum.
#[allow(dead_code)]
pub fn manifest_checksum(exp: &AssetManifestExport) -> u32 {
    exp.checksum
}

/// Return file list.
#[allow(dead_code)]
pub fn manifest_file_list(exp: &AssetManifestExport) -> &[String] {
    &exp.assets
}

/// Compute export size.
#[allow(dead_code)]
pub fn manifest_export_size(exp: &AssetManifestExport) -> usize {
    manifest_to_json(exp).len()
}

/// Validate manifest.
#[allow(dead_code)]
pub fn validate_manifest(exp: &AssetManifestExport) -> bool {
    !exp.version.is_empty() && !exp.assets.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_asset_manifest() {
        let e = export_asset_manifest("1.0", &["mesh.bin", "tex.png"]);
        assert_eq!(manifest_asset_count(&e), 2);
    }

    #[test]
    fn test_manifest_version() {
        let e = export_asset_manifest("2.1", &["a"]);
        assert_eq!(manifest_version(&e), "2.1");
    }

    #[test]
    fn test_manifest_to_json() {
        let e = export_asset_manifest("1.0", &["mesh.bin"]);
        let j = manifest_to_json(&e);
        assert!(j.contains("\"version\":\"1.0\""));
    }

    #[test]
    fn test_manifest_checksum() {
        let e = export_asset_manifest("1.0", &["abc"]);
        assert!(manifest_checksum(&e) > 0);
    }

    #[test]
    fn test_manifest_file_list() {
        let e = export_asset_manifest("1.0", &["a", "b"]);
        assert_eq!(manifest_file_list(&e).len(), 2);
    }

    #[test]
    fn test_manifest_export_size() {
        let e = export_asset_manifest("1.0", &["a"]);
        assert!(manifest_export_size(&e) > 0);
    }

    #[test]
    fn test_validate_manifest() {
        let e = export_asset_manifest("1.0", &["a"]);
        assert!(validate_manifest(&e));
    }

    #[test]
    fn test_validate_empty_assets() {
        let e = export_asset_manifest("1.0", &[]);
        assert!(!validate_manifest(&e));
    }

    #[test]
    fn test_validate_empty_version() {
        let e = export_asset_manifest("", &["a"]);
        assert!(!validate_manifest(&e));
    }

    #[test]
    fn test_empty_manifest() {
        let e = export_asset_manifest("1.0", &[]);
        assert_eq!(manifest_asset_count(&e), 0);
    }
}
